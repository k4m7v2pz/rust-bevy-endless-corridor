//! 陷阱与生存系统
//!
//! 对应 Python `engine/map/trap_system.py`:
//! - 陷阱触发（区域 / 交互）
//! - 线索日志（Journal）
//! - 死亡状态（DeathState）与检查点重生
//!
//! 在 Rust 版中:
//! - 陷阱扣血映射为扣理智 + 增加恐惧 + 屏幕震动（本版无独立血量，理智归零即死亡）
//! - DeathState 作为 WorldState 字段而非独立类
//! - Journal 作为 Resource

use bevy::prelude::*;
use bevy_state::state::NextState;

use crate::constants::*;
use crate::player::Player;
use crate::{GameMap, GameState, PlayerTag, ScreenShake, WorldState};

// ---------- 陷阱实体 ----------

/// 陷阱种类
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrapKind {
    /// 区域陷阱：进入范围即触发
    Area,
    /// 交互陷阱：需主动按 E 触发
    Interaction,
}

/// 陷阱组件
#[derive(Component)]
pub struct Trap {
    pub trap_id: String,
    pub kind: TrapKind,
    /// 触发半径（世界单位）
    pub trigger_radius: f32,
    /// 伤害值（映射为理智扣减）
    pub damage: f32,
    /// 是否仍可触发
    pub active: bool,
    /// （交互陷阱）是否已被互动触发过
    pub triggered_by_interaction: bool,
    /// 触发后追加到日志的线索文本（可空）
    pub clue: Option<String>,
    /// 触发后是否直接致死（致命陷阱）
    pub lethal: bool,
    /// 描述
    pub description: String,
}

/// 陷阱标记组件（供 despawn_screen 识别）
#[derive(Component)]
pub struct TrapTag;

/// 在地图上生成陷阱实体
pub fn spawn_traps(commands: &mut Commands, map: &GameMap) {
    for spec in &map.trap_spots {
        let (color_inner, color_outer, size) = match spec.kind {
            TrapKind::Area => (
                Color::rgba(0.85, 0.12, 0.12, 0.35),
                Color::rgba(1.0, 0.3, 0.3, 0.18),
                spec.trigger_radius * 2.0,
            ),
            TrapKind::Interaction => (
                Color::rgba(0.85, 0.55, 0.12, 0.45),
                Color::rgba(1.0, 0.7, 0.25, 0.22),
                spec.trigger_radius * 2.0,
            ),
        };
        commands.spawn((
            SpriteBundle {
                sprite: Sprite {
                    color: color_outer,
                    custom_size: Some(Vec2::new(size, size)),
                    ..default()
                },
                transform: Transform::from_xyz(spec.pos.x, spec.pos.y, 1.2),
                ..default()
            },
            TrapTag,
        ));
        commands.spawn((
            SpriteBundle {
                sprite: Sprite {
                    color: color_inner,
                    custom_size: Some(Vec2::new(size * 0.55, size * 0.55)),
                    ..default()
                },
                transform: Transform::from_xyz(spec.pos.x, spec.pos.y, 1.25),
                ..default()
            },
            TrapTag,
            Trap {
                trap_id: spec.trap_id.clone(),
                kind: spec.kind,
                trigger_radius: spec.trigger_radius,
                damage: spec.damage,
                active: true,
                triggered_by_interaction: false,
                clue: spec.clue.clone(),
                lethal: spec.lethal,
                description: spec.description.clone(),
            },
        ));
    }
}

// ---------- 事件 ----------

/// 陷阱触发事件（对应 Python EventType.TRAP_TRIGGERED）
#[derive(Event, Debug, Clone)]
pub struct TrapTriggeredEvent {
    pub trap_id: String,
    pub kind: TrapKind,
    pub position: Vec2,
    pub damage: f32,
    pub lethal: bool,
    pub player_pos: Vec2,
}

// ---------- 线索日志 ----------

/// 单条线索条目
#[derive(Debug, Clone)]
pub struct JournalEntry {
    pub entry_id: String,
    pub title: String,
    pub content: String,
    pub discovered: bool,
}

/// 线索日志系统（对应 Python Journal）
#[derive(Resource, Default)]
pub struct Journal {
    pub entries: Vec<JournalEntry>,
}

impl Journal {
    pub fn add_entry(&mut self, entry_id: &str, title: &str, content: &str) {
        if self.entries.iter().any(|e| e.entry_id == entry_id) {
            return;
        }
        self.entries.push(JournalEntry {
            entry_id: entry_id.to_string(),
            title: title.to_string(),
            content: content.to_string(),
            discovered: false,
        });
    }

    pub fn discover_entry(&mut self, entry_id: &str) -> bool {
        for entry in &mut self.entries {
            if entry.entry_id == entry_id && !entry.discovered {
                entry.discovered = true;
                info!("线索已记录: {}", entry.title);
                return true;
            }
        }
        false
    }

    /// 已发现的线索数量
    pub fn discovered_count(&self) -> usize {
        self.entries.iter().filter(|e| e.discovered).count()
    }

    /// 直接以内容作为条目登记并标记发现（用于陷阱触发时追加线索）
    pub fn add_clue(&mut self, entry_id: &str, content: &str) {
        self.add_entry(entry_id, "线索", content);
        self.discover_entry(entry_id);
    }
}

/// 线索被发现事件（对应 Python EventType.CLUE_DISCOVERED）
#[derive(Event, Debug, Clone)]
pub struct ClueDiscoveredEvent {
    pub entry_id: String,
    pub title: String,
    pub content: String,
}

// ---------- 触发检测 ----------

/// 检查区域陷阱：玩家进入范围即触发（对应 TrapManager.check_area_traps）
pub fn trap_check_area(
    mut commands: Commands,
    mut state: ResMut<WorldState>,
    mut shake: ResMut<ScreenShake>,
    mut journal: ResMut<Journal>,
    mut evts: EventWriter<TrapTriggeredEvent>,
    mut clue_evts: EventWriter<ClueDiscoveredEvent>,
    mut next_state: ResMut<NextState<GameState>>,
    player_q: Query<(&Transform, &Player), With<PlayerTag>>,
    mut trap_q: Query<(Entity, &mut Trap, &Transform)>,
) {
    let Ok((p_trans, _player)) = player_q.get_single() else { return };
    let pp = p_trans.translation.xy();

    for (e, mut trap, t_trans) in trap_q.iter_mut() {
        if !trap.active {
            continue;
        }
        if trap.kind != TrapKind::Area {
            continue;
        }
        let ep = t_trans.translation.xy();
        if (ep - pp).length() <= trap.trigger_radius {
            trigger_trap(
                &mut commands,
                &mut state,
                &mut shake,
                &mut journal,
                &mut evts,
                &mut clue_evts,
                &mut next_state,
                e,
                &mut trap,
                ep,
                pp,
            );
        }
    }
}

/// 交互陷阱输入：按 E 与附近交互陷阱互动（对应 TrapManager.handle_interaction_with_trap）
pub fn trap_interaction_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut commands: Commands,
    mut state: ResMut<WorldState>,
    mut shake: ResMut<ScreenShake>,
    mut journal: ResMut<Journal>,
    mut evts: EventWriter<TrapTriggeredEvent>,
    mut clue_evts: EventWriter<ClueDiscoveredEvent>,
    mut next_state: ResMut<NextState<GameState>>,
    player_q: Query<&Transform, With<PlayerTag>>,
    mut trap_q: Query<(Entity, &mut Trap, &Transform)>,
) {
    if !keyboard.just_pressed(KeyCode::KeyE) {
        return;
    }
    let Ok(p_trans) = player_q.get_single() else { return };
    let pp = p_trans.translation.xy();

    // 找最近的交互陷阱
    let mut target: Option<(Entity, Vec2)> = None;
    let mut best_dist = TRAP_INTERACTION_DISTANCE;
    for (e, trap, t_trans) in trap_q.iter_mut() {
        if !trap.active || trap.kind != TrapKind::Interaction {
            continue;
        }
        let ep = t_trans.translation.xy();
        let d = (ep - pp).length();
        if d <= best_dist {
            best_dist = d;
            target = Some((e, ep));
        }
    }

    if let Some((e, ep)) = target {
        // 取到对应 Trap 的可变引用
        if let Some((_, mut trap, _)) = trap_q.iter_mut().find(|(te, _, _)| *te == e) {
            if trap.trigger_interaction() {
                trigger_trap(
                    &mut commands,
                    &mut state,
                    &mut shake,
                    &mut journal,
                    &mut evts,
                    &mut clue_evts,
                    &mut next_state,
                    e,
                    &mut trap,
                    ep,
                    pp,
                );
            }
        }
    }
}

impl Trap {
    /// 交互陷阱：标记为已互动触发（对应 InteractionTrap.trigger_interaction）
    fn trigger_interaction(&mut self) -> bool {
        if !self.active || self.triggered_by_interaction {
            return false;
        }
        self.triggered_by_interaction = true;
        true
    }
}

/// 触发陷阱的共用逻辑（对应 TrapManager.trigger_trap + 死亡处理）
fn trigger_trap(
    commands: &mut Commands,
    state: &mut WorldState,
    shake: &mut ScreenShake,
    journal: &mut Journal,
    evts: &mut EventWriter<TrapTriggeredEvent>,
    clue_evts: &mut EventWriter<ClueDiscoveredEvent>,
    next_state: &mut NextState<GameState>,
    entity: Entity,
    trap: &mut Trap,
    trap_pos: Vec2,
    player_pos: Vec2,
) {
    trap.active = false;

    evts.send(TrapTriggeredEvent {
        trap_id: trap.trap_id.clone(),
        kind: trap.kind,
        position: trap_pos,
        damage: trap.damage,
        lethal: trap.lethal,
        player_pos,
    });

    info!("陷阱已触发: {} ({})", trap.trap_id, trap.description);

    // 扣理智 + 增加恐惧 + 屏幕震动
    state.sanity = (state.sanity - trap.damage).max(0.0);
    state.fear_level = (state.fear_level + trap.damage * 0.5).min(FEAR_MAX);
    shake.intensity = (shake.intensity + trap.damage * 0.3).min(SCREEN_SHAKE_MAX_INTENSITY);

    // 追加线索到日志
    if let Some(clue) = trap.clue.clone() {
        let entry_id = format!("clue_{}", trap.trap_id);
        journal.add_clue(&entry_id, &clue);
        clue_evts.send(ClueDiscoveredEvent {
            entry_id,
            title: "线索".to_string(),
            content: clue,
        });
    }

    // 移除陷阱的可视实体
    commands.entity(entity).despawn();

    // 致命陷阱或理智归零 => 死亡
    if trap.lethal || state.sanity <= 0.0 {
        state.death_count += 1;
        state.last_death_reason = format!("触发了{}", trap.description);
        next_state.set(GameState::GameOver);
    } else {
        // 记录最近检查点（玩家当前位置）以便后续重生
        state.last_checkpoint = Some(player_pos);
    }
}

// ---------- 插件 ----------

/// 陷阱系统插件
pub struct TrapPlugin;

impl Plugin for TrapPlugin {
    fn build(&self, app: &mut App) {
        app
            .insert_resource(Journal::default())
            .add_event::<TrapTriggeredEvent>()
            .add_event::<ClueDiscoveredEvent>();
    }
}
