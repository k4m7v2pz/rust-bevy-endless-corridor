//! 无尽回廊 - The Endless Corridor (Rust Bevy 重写)
//!
//! 系统列表 (主要模块):
//! - tile_map: 程序化生成地图 (房间 + 走廊)
//! - player: 玩家移动 / 躲藏 / 手电筒
//! - monster: 怪物 AI (巡逻 / 追逐 / 搜索)
//! - items: 钥匙收集 / 出口门 / 躲藏点
//! - darkness: 黑暗覆盖与光锥 (手电筒/怪物红光)
//! - perception: 恐惧 / 理智 / 幻觉系统
//! - save: 存档系统 (7位哈希ID + 双时间维度)
//! - game_ui: 开始 / 结束 / 胜利 / HUD
//! - trap: 陷阱与生存系统 (区域/交互陷阱、线索日志、死亡与检查点)

use bevy::prelude::*;
use bevy::window::WindowResolution;
use bevy::asset::load_internal_binary_asset;
use bevy::text::Font;
use bevy_state::app::AppExtStates;
use bevy_state::state::{OnEnter, OnExit, States, SubStates};
use bevy_state::condition::in_state;

mod constants;
mod tile_map;
mod player;
mod monster;
mod items;
mod darkness;
mod perception;
mod save;
mod game_ui;
mod dialogue;
mod endings;
mod looping_corridor;
mod environment;
mod warnings;
mod sound_cue;
mod trap;
mod debug;
mod narrative;
mod notification;
mod fog_of_war;

use tile_map::GameMap;
use constants::*;

// ---------- 游戏状态 ----------
#[derive(States, Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum GameState {
    #[default]
    StartScreen,
    SaveMenu,
    Playing,
    GameOver,
    Win,
}

/// 游戏内子状态：暂停覆盖层（保留世界、冻结逻辑）
#[derive(SubStates, Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[source(GameState = GameState::Playing)]
pub enum PlayPhase {
    #[default]
    Running,
    Paused,
}

/// 存档界面返回来源（多入口：主菜单 / 游戏中）
#[derive(Resource, Default)]
pub struct SaveReturnTo(pub SaveReturnOrigin);

#[derive(Default, Clone, Copy, PartialEq, Eq)]
pub enum SaveReturnOrigin {
    #[default]
    Title,
    Game,
}

// ---------- 全局资源 ----------
#[derive(Resource)]
pub struct WorldState {
    pub fear_level: f32,
    pub sanity: f32,
    pub keys_collected: u32,
    pub keys_total: u32,
    /// 死亡次数（对应 Python DeathState.death_count）
    pub death_count: u32,
    /// 最近死亡原因
    pub last_death_reason: String,
    /// 最近检查点（用于重生）
    pub last_checkpoint: Option<Vec2>,
}

impl Default for WorldState {
    fn default() -> Self {
        Self {
            fear_level: 0.0,
            sanity: SANITY_MAX,
            keys_collected: 0,
            keys_total: KEYS_REQUIRED,
            death_count: 0,
            last_death_reason: String::new(),
            last_checkpoint: None,
        }
    }
}

#[derive(Resource, Default)]
pub struct ScreenShake {
    pub intensity: f32,
}

/// 游戏计时器
#[derive(Resource, Default)]
pub struct GameTimer {
    pub seconds: f32,
}

/// 待加载的存档数据（从存档菜单进入游戏时使用）
#[derive(Resource, Default)]
pub struct PendingSaveLoad {
    pub save_id: Option<String>,
}

// ---------- 标记组件 (让 despawn_screen 能识别) ----------
#[derive(Component)]
pub struct PlayerTag;

#[derive(Component)]
pub struct MonsterTag;

#[derive(Component)]
pub struct ItemTag;

#[derive(Component)]
pub struct HidingSpotTag;

#[derive(Component)]
pub struct HudTag;

#[derive(Component)]
pub struct SaveMenuTag;

// 注意: HallucinationTag / DarknessOverlayTag 定义在各自模块中,
// 主文件里通过 re-export 暴露给其他子模块使用。
pub use perception::HallucinationTag;
pub use darkness::DarknessOverlayTag;

#[derive(Component)]
pub struct MainCamera;

#[derive(Component)]
pub struct UiCamera;

/// 注册打包进二进制的开源中文字体为默认字体。
///
/// 字体：思源黑体 Noto Sans CJK SC Regular（SIL OFL 1.1 开源许可），
/// 随仓库放在 `fonts/`。项目使用 `default-features = false`，Bevy 不内置任何
/// 字体；若不注册，全部 UI 文本（中文）将无法渲染（窗口一片黑）。
struct CjkFontPlugin;

impl Plugin for CjkFontPlugin {
    fn build(&self, app: &mut App) {
        load_internal_binary_asset!(
            app,
            Handle::default(),
            "../fonts/NotoSansCJKsc-Regular.otf",
            |bytes: &[u8], _path: String| -> Font {
                Font::try_from_bytes(bytes.to_vec()).expect("内置中文字体解析失败")
            }
        );
    }
}

// ---------- 主入口 ----------
fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "无尽回廊 - The Endless Corridor".into(),
                resolution: WindowResolution::new(WINDOW_WIDTH, WINDOW_HEIGHT),
                resizable: true,
                ..default()
            }),
            ..default()
        }))
        .add_plugins(CjkFontPlugin)
        .init_state::<GameState>()
        .add_sub_state::<PlayPhase>()
        .insert_resource(ClearColor(Color::srgb(0.02, 0.015, 0.015)))
        .insert_resource(WorldState::default())
        .insert_resource(ScreenShake::default())
        .insert_resource(GameTimer::default())
        .insert_resource(PendingSaveLoad::default())
        .insert_resource(SaveReturnTo::default())
        .add_plugins(dialogue::DialoguePlugin)
        .add_plugins(endings::EndingsPlugin::default())
        .add_plugins(environment::EnvironmentPlugin)
        .add_plugins(warnings::WarningPlugin::default())
        .add_plugins(sound_cue::SoundCuePlugin)
        .add_plugins(trap::TrapPlugin)
        .add_plugins(debug::DebugPlugin)
        .add_plugins(narrative::NarrativePlugin)
        .add_plugins(notification::NotificationPlugin)
        .add_plugins(fog_of_war::FogOfWarPlugin)
        // --- 启动 ---
        .add_systems(Startup, setup_camera)
        // --- 开始界面 ---
        .add_systems(OnEnter(GameState::StartScreen), game_ui::spawn_start_screen)
        .add_systems(
            Update,
            game_ui::start_screen_input.run_if(in_state(GameState::StartScreen)),
        )
        .add_systems(
            OnExit(GameState::StartScreen),
            game_ui::despawn_screen::<game_ui::StartScreenTag>,
        )
        // --- 存档界面 ---
        .add_systems(OnEnter(GameState::SaveMenu), game_ui::spawn_save_menu)
        .add_systems(
            Update,
            game_ui::save_menu_input.run_if(in_state(GameState::SaveMenu)),
        )
        .add_systems(
            OnExit(GameState::SaveMenu),
            game_ui::despawn_screen::<SaveMenuTag>,
        )
        // --- 进入 Playing ---
        .add_systems(
            OnEnter(GameState::Playing),
            (
                reset_world_state,
                reset_game_timer,
                spawn_game_world,
                darkness::setup_darkness_meshes,
                perception::setup_hallucination_assets,
                game_ui::spawn_hud,
                notification::setup_notification_ui,
                apply_pending_save_load,
            )
                .chain(),
        )
        .add_systems(
            Update,
            (
                update_game_timer,
                player::player_movement,
                player::player_hide_input,
                player::rotate_flashlight_to_mouse,
                monster::monster_ai,
                items::item_update,
                monster::check_player_monster_collision,
                perception::update_fear_and_sanity,
                perception::update_perception,
                perception::camera_follow,
                darkness::darkness_overlay,
                perception::draw_hallucinations,
                game_ui::update_hud_text,
                trap::trap_check_area,
                trap::trap_interaction_input,
                check_sanity_death,
                handle_save_input,
                handle_quick_load_input,
                esc_to_pause,
                letter_key_to_save,
            )
                .run_if(in_state(PlayPhase::Running)),
        )
        .add_systems(
            Update,
            (
                fog_of_war::fog_reveal_around_player,
                fog_of_war::fog_spawn_unrevealed,
                fog_of_war::fog_despawn_revealed,
            )
                .chain()
                .run_if(in_state(PlayPhase::Running)),
        )
        // --- 暂停覆盖层（ESC 菜单，世界保留）---
        .add_systems(OnEnter(PlayPhase::Paused), game_ui::spawn_pause_menu)
        .add_systems(
            Update,
            game_ui::pause_menu_input.run_if(in_state(PlayPhase::Paused)),
        )
        .add_systems(
            OnExit(PlayPhase::Paused),
            game_ui::despawn_screen::<game_ui::PauseMenuTag>,
        )
        .add_systems(
            OnExit(GameState::Playing),
            (
                game_ui::despawn_screen::<PlayerTag>,
                game_ui::despawn_screen::<MonsterTag>,
                game_ui::despawn_screen::<ItemTag>,
                game_ui::despawn_screen::<HidingSpotTag>,
                game_ui::despawn_screen::<HudTag>,
                game_ui::despawn_screen::<HallucinationTag>,
                game_ui::despawn_screen::<darkness::DarknessOverlayTag>,
                game_ui::despawn_screen::<tile_map::MapTile>,
                game_ui::despawn_screen::<trap::TrapTag>,
                game_ui::despawn_screen::<notification::NotificationRoot>,
                fog_of_war::despawn_all_fog,
            ),
        )
        // --- 游戏结束 ---
        .add_systems(OnEnter(GameState::GameOver), game_ui::spawn_game_over_screen)
        .add_systems(
            Update,
            game_ui::game_over_input.run_if(in_state(GameState::GameOver)),
        )
        .add_systems(
            OnExit(GameState::GameOver),
            game_ui::despawn_screen::<game_ui::GameOverScreenTag>,
        )
        // --- 胜利 ---
        .add_systems(OnEnter(GameState::Win), game_ui::spawn_win_screen)
        .add_systems(Update, game_ui::game_over_input.run_if(in_state(GameState::Win)))
        .add_systems(
            OnExit(GameState::Win),
            game_ui::despawn_screen::<game_ui::WinScreenTag>,
        )
        .run();
}

// ---------- 启动系统: 摄像机 ----------
fn setup_camera(mut commands: Commands) {
    commands.spawn((
        Camera2dBundle {
            camera: Camera {
                order: 0,
                ..default()
            },
            transform: Transform::from_xyz(WORLD_WIDTH * 0.5, WORLD_HEIGHT * 0.5, CAMERA_Z),
            ..default()
        },
        MainCamera,
    ));

    commands.spawn((
        Camera2dBundle {
            camera: Camera {
                order: 2,
                clear_color: ClearColorConfig::None,
                ..default()
            },
            ..default()
        },
        UiCamera,
    ));
}

fn reset_world_state(mut state: ResMut<WorldState>) {
    *state = WorldState::default();
}

fn reset_game_timer(mut timer: ResMut<GameTimer>) {
    timer.seconds = 0.0;
}

/// 应用待加载的存档
fn apply_pending_save_load(
    mut pending: ResMut<PendingSaveLoad>,
    mut world_state: ResMut<WorldState>,
    mut timer: ResMut<GameTimer>,
    mut player_q: Query<&mut Transform, (With<PlayerTag>, Without<monster::Monster>)>,
    mut monster_q: Query<(&mut Transform, &mut monster::Monster), Without<PlayerTag>>,
    mut revealed: ResMut<fog_of_war::RevealedTiles>,
) {
    let Some(save_id) = pending.save_id.take() else { return };
    
    let manager = save::SaveManager::new("saves".into());
    match manager.load_save(&save_id) {
        Ok(save_data) => {
            let snap = &save_data.game_state;
            info!("加载存档: {} (时长: {})", save_id, save_data.game_duration);
            
            // 恢复世界状态
            world_state.fear_level = snap.fear_level;
            world_state.sanity = snap.sanity;
            world_state.keys_collected = snap.keys_collected;
            world_state.death_count = snap.death_count;
            world_state.last_death_reason = snap.last_death_reason.clone();
            world_state.last_checkpoint = snap.last_checkpoint.map(|[x, y]| Vec2::new(x, y));
            timer.seconds = snap.game_time_seconds;
            
            // 恢复玩家位置
            if let Ok(mut player_trans) = player_q.get_single_mut() {
                player_trans.translation.x = snap.player_x;
                player_trans.translation.y = snap.player_y;
            }
            
            // 恢复 Fog of War 探索记忆
            *revealed = fog_of_war::RevealedTiles::from_vec(snap.revealed_tiles.clone());

            // 恢复怪物位置（尽量匹配数量）
            let mut monster_iter = monster_q.iter_mut();
            for monster_snap in &snap.monsters {
                if let Some((mut m_trans, mut m)) = monster_iter.next() {
                    m_trans.translation.x = monster_snap.x;
                    m_trans.translation.y = monster_snap.y;
                    m.state = match monster_snap.state {
                        0 => monster::MonsterState::Patrolling,
                        1 => monster::MonsterState::Chasing,
                        _ => monster::MonsterState::Searching,
                    };
                }
            }
        }
        Err(e) => error!("加载存档失败: {}", e),
    }
}

fn update_game_timer(mut timer: ResMut<GameTimer>, time: Res<Time>) {
    timer.seconds += time.delta_seconds();
}

/// 存档快捷键处理 (F5保存, F9加载)
// ---------- 暂停 / 存档界面多入口 ----------
/// ESC 打开暂停菜单（调试控制台打开时 ESC 用于关闭控制台，不触发暂停）
fn esc_to_pause(
    keyboard: Res<ButtonInput<KeyCode>>,
    console: Res<debug::ConsoleState>,
    mut next_phase: ResMut<NextState<PlayPhase>>,
) {
    if keyboard.just_pressed(KeyCode::Escape) && !console.visible {
        next_phase.set(PlayPhase::Paused);
    }
}

/// 字母键 L 直达存档界面（多入口之一：游戏中直达）
fn letter_key_to_save(
    keyboard: Res<ButtonInput<KeyCode>>,
    console: Res<debug::ConsoleState>,
    mut next_state: ResMut<NextState<GameState>>,
    mut ret: ResMut<SaveReturnTo>,
) {
    if keyboard.just_pressed(KeyCode::KeyL) && !console.visible {
        ret.0 = SaveReturnOrigin::Game;
        next_state.set(GameState::SaveMenu);
    }
}

fn handle_save_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    world_state: Res<WorldState>,
    timer: Res<GameTimer>,
    player_q: Query<&Transform, (With<PlayerTag>, Without<monster::Monster>)>,
    monster_q: Query<(&Transform, &monster::Monster), Without<PlayerTag>>,
    revealed: Res<fog_of_war::RevealedTiles>,
) {
    // F5 保存
    if keyboard.just_pressed(KeyCode::F5) {
        let player_pos = player_q.single().translation.xy();
        let monster_states: Vec<_> = monster_q.iter()
            .map(|(t, m)| {
                let state = match m.state {
                    monster::MonsterState::Patrolling => 0,
                    monster::MonsterState::Chasing => 1,
                    monster::MonsterState::Searching => 2,
                };
                (t.translation.xy(), state)
            })
            .collect();
        
        let snapshot = save::create_snapshot_from_state(
            player_pos,
            world_state.fear_level,
            world_state.sanity,
            world_state.keys_collected,
            &monster_states,
            timer.seconds,
            world_state.death_count,
            &world_state.last_death_reason,
            world_state.last_checkpoint,
            &revealed.to_vec()[..],
        );
        
        let manager = save::SaveManager::new("saves".into());
        match manager.create_save(snapshot, timer.seconds) {
            Ok(save_data) => info!("存档已保存: {} (时长: {})", save_data.save_id, save_data.game_duration),
            Err(e) => error!("存档失败: {}", e),
        }
    }
}

/// 快速读档 (F9)
fn handle_quick_load_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut world_state: ResMut<WorldState>,
    mut timer: ResMut<GameTimer>,
    mut player_q: Query<&mut Transform, (With<PlayerTag>, Without<monster::Monster>)>,
    mut monster_q: Query<(&mut Transform, &mut monster::Monster), Without<PlayerTag>>,
) {
    if keyboard.just_pressed(KeyCode::F9) {
        let manager = save::SaveManager::new("saves".into());
        let saves = manager.list_saves();
        if let Some(latest) = saves.first() {
            match manager.load_save(&latest.save_id) {
                Ok(save_data) => {
                    let snap = &save_data.game_state;
                    info!("加载存档: {} (时长: {})", save_data.save_id, save_data.game_duration);
                    
                    world_state.fear_level = snap.fear_level;
                    world_state.sanity = snap.sanity;
                    world_state.keys_collected = snap.keys_collected;
                    world_state.death_count = snap.death_count;
                    world_state.last_death_reason = snap.last_death_reason.clone();
                    world_state.last_checkpoint = snap.last_checkpoint.map(|[x, y]| Vec2::new(x, y));
                    timer.seconds = snap.game_time_seconds;
                    
                    if let Ok(mut p) = player_q.get_single_mut() {
                        p.translation.x = snap.player_x;
                        p.translation.y = snap.player_y;
                    }
                    
                    let mut iter = monster_q.iter_mut();
                    for m_snap in &snap.monsters {
                        if let Some((mut t, mut m)) = iter.next() {
                            t.translation.x = m_snap.x;
                            t.translation.y = m_snap.y;
                            m.state = match m_snap.state {
                                0 => monster::MonsterState::Patrolling,
                                1 => monster::MonsterState::Chasing,
                                _ => monster::MonsterState::Searching,
                            };
                        }
                    }
                }
                Err(e) => error!("加载失败: {}", e),
            }
        }
    }
}

fn spawn_game_world(mut commands: Commands) {
    let map = GameMap::generate();

    tile_map::spawn_map_tiles(&mut commands, &map);

    player::spawn_player(&mut commands, &map);
    monster::spawn_monsters(&mut commands, &map);
    items::spawn_items_and_spots(&mut commands, &map);
    trap::spawn_traps(&mut commands, &map);

    commands.insert_resource(map);
}

/// 理智归零触发死亡（陷阱扣理智的兜底死亡路径）
fn check_sanity_death(
    state: Res<WorldState>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    if state.sanity <= 0.0 {
        next_state.set(GameState::GameOver);
    }
}
