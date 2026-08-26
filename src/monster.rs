//! 怪物 AI: 巡逻 / 追逐 / 搜索

use bevy::prelude::*;
use bevy_state::state::NextState;
use rand::Rng;

use crate::{GameMap, GameState, MonsterTag, PlayerTag, WORLD_HEIGHT, WORLD_WIDTH, player::Player};
use crate::constants::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MonsterState {
    Patrolling,
    Chasing,
    Searching,
}

#[derive(Component)]
pub struct Monster {
    pub radius: f32,
    pub base_speed: f32,
    pub max_speed: f32,
    pub vision_radius: f32,
    pub hearing_radius: f32,
    pub state: MonsterState,
    pub target: Vec2,
    pub last_seen_player: Vec2,
    pub search_timer: f32,
    pub patrol_points: Vec<Vec2>,
    pub patrol_index: usize,
    pub search_points: Vec<Vec2>,
    pub search_index: usize,
}

impl Monster {
    pub fn new(spawn: Vec2) -> Self {
        let mut rng = rand::thread_rng();
        let mut patrol = Vec::with_capacity(MONSTER_PATROL_POINTS_COUNT);
        for _ in 0..MONSTER_PATROL_POINTS_COUNT {
            let ang = rng.gen_range(0.0..std::f32::consts::TAU);
            let dist = rng.gen_range(MONSTER_PATROL_MIN_DIST..MONSTER_PATROL_MAX_DIST);
            patrol.push(spawn + Vec2::new(ang.cos() * dist, ang.sin() * dist));
        }
        Self {
            radius: MONSTER_RADIUS,
            base_speed: MONSTER_BASE_SPEED,
            max_speed: MONSTER_MAX_SPEED,
            vision_radius: MONSTER_VISION_RADIUS,
            hearing_radius: MONSTER_HEARING_RADIUS,
            state: MonsterState::Patrolling,
            target: patrol[0],
            search_timer: 0.0,
            patrol_points: patrol,
            patrol_index: 0,
            last_seen_player: Vec2::ZERO,
            search_points: Vec::new(),
            search_index: 0,
        }
    }
    
    /// 根据玩家躲藏状态调整感知范围
    fn adjusted_senses(&self, player_hiding: bool) -> (f32, f32) {
        if player_hiding {
            (self.vision_radius * MONSTER_HIDE_VISION_MOD, self.hearing_radius * MONSTER_HIDE_HEARING_MOD)
        } else {
            (self.vision_radius, self.hearing_radius)
        }
    }
    
    /// 获取当前状态对应的速度
    fn current_speed(&self) -> f32 {
        match self.state {
            MonsterState::Chasing => self.max_speed,
            MonsterState::Searching => self.base_speed * MONSTER_SEARCH_SPEED_MOD,
            MonsterState::Patrolling => self.base_speed * MONSTER_PATROL_SPEED_MOD,
        }
    }
    
    /// 更新巡逻目标点
    fn update_patrol_target(&mut self, pos: Vec2) {
        let t = self.patrol_points[self.patrol_index % self.patrol_points.len()];
        if (t - pos).length() < 30.0 {
            self.patrol_index += 1;
        }
        self.target = self.patrol_points[self.patrol_index % self.patrol_points.len()];
    }
    
    /// 处理巡逻状态逻辑
    fn handle_patrolling(&mut self, player_pos: Vec2, can_see: bool, can_hear: bool) {
        if can_see {
            self.state = MonsterState::Chasing;
            self.last_seen_player = player_pos;
        } else if can_hear {
            self.state = MonsterState::Searching;
            self.search_timer = 0.0;
            self.search_points = generate_search_points(player_pos);
            self.search_index = 0;
        }
    }
    
    /// 处理追逐状态逻辑
    fn handle_chasing(&mut self, player_pos: Vec2, can_see: bool, dt: f32) {
        if can_see {
            self.last_seen_player = player_pos;
        } else {
            self.search_timer += dt;
            if self.search_timer > MONSTER_CHASE_TIMEOUT {
                self.state = MonsterState::Searching;
                self.search_timer = 0.0;
                self.search_points = generate_search_points(self.last_seen_player);
                self.search_index = 0;
            }
        }
        self.target = self.last_seen_player;
    }
    
    /// 处理搜索状态逻辑
    fn handle_searching(&mut self, pos: Vec2, player_pos: Vec2, can_see: bool, dt: f32) {
        if can_see {
            self.state = MonsterState::Chasing;
            self.last_seen_player = player_pos;
            self.search_timer = 0.0;
        } else {
            self.search_timer += dt;
            if self.search_timer > MONSTER_SEARCH_TIMEOUT {
                self.state = MonsterState::Patrolling;
            } else if !self.search_points.is_empty() {
                let t = self.search_points[self.search_index % self.search_points.len()];
                if (t - pos).length() < 30.0 {
                    self.search_index += 1;
                }
                self.target = t;
            }
        }
    }
}

pub fn spawn_monsters(commands: &mut Commands, map: &GameMap) {
    for &pos in &map.monster_spawns {
        commands.spawn((
            SpriteBundle {
                sprite: Sprite {
                    color: Color::rgb(1.0, 0.95, 0.3),
                    custom_size: Some(Vec2::new(MONSTER_SIZE, MONSTER_SIZE)),
                    ..default()
                },
                transform: Transform::from_xyz(pos.x, pos.y, 2.0),
                ..default()
            },
            Monster::new(pos),
            MonsterTag,
        ));
    }
}

pub fn monster_ai(
    time: Res<Time>,
    map: Res<GameMap>,
    player_q: Query<(&Transform, &Player), (With<PlayerTag>, Without<Monster>)>,
    mut monster_q: Query<(&mut Transform, &mut Monster, &mut Sprite), Without<PlayerTag>>,
) {
    let Ok((p_trans, player)) = player_q.get_single() else { return };
    let player_pos = p_trans.translation.xy();
    let dt = time.delta_seconds();

    for (mut m_trans, mut m, mut sprite) in monster_q.iter_mut() {
        let pos = m_trans.translation.xy();
        let dist = (player_pos - pos).length();

        let (vision, hearing) = m.adjusted_senses(player.is_hiding);
        let can_see = dist < vision;
        let can_hear = dist < hearing;

        // 状态机处理
        match m.state {
            MonsterState::Patrolling => m.handle_patrolling(player_pos, can_see, can_hear),
            MonsterState::Chasing => m.handle_chasing(player_pos, can_see, dt),
            MonsterState::Searching => m.handle_searching(pos, player_pos, can_see, dt),
        }

        // 巡逻状态下的目标更新
        if matches!(m.state, MonsterState::Patrolling) {
            m.update_patrol_target(pos);
        }

        // 移动
        let speed = m.current_speed();
        let dir_to_target = (m.target - pos).normalize_or_zero();
        let next = pos + dir_to_target * speed * dt;
        let clamped = map.collide_circle(next, m.radius);
        m_trans.translation.x = clamped.x.clamp(20.0, WORLD_WIDTH - 20.0);
        m_trans.translation.y = clamped.y.clamp(20.0, WORLD_HEIGHT - 20.0);

        // 颜色更新
        sprite.color = match m.state {
            MonsterState::Chasing => Color::rgb(1.0, 0.25, 0.2),
            MonsterState::Searching => Color::rgb(1.0, 0.7, 0.2),
            MonsterState::Patrolling => Color::rgb(1.0, 0.95, 0.3),
        };
    }
}

fn generate_search_points(center: Vec2) -> Vec<Vec2> {
    let mut pts = Vec::with_capacity(MONSTER_SEARCH_POINTS_COUNT);
    let mut rng = rand::thread_rng();
    for i in 0..MONSTER_SEARCH_POINTS_COUNT {
        let ang = (i as f32) * std::f32::consts::TAU / (MONSTER_SEARCH_POINTS_COUNT as f32) + rng.gen_range(-0.2..0.2);
        let dist = rng.gen_range(MONSTER_SEARCH_MIN_DIST..MONSTER_SEARCH_MAX_DIST);
        pts.push(center + Vec2::new(ang.cos() * dist, ang.sin() * dist));
    }
    pts
}

pub fn check_player_monster_collision(
    mut next_state: ResMut<NextState<GameState>>,
    player_q: Query<(&Transform, &Player), Without<MonsterTag>>,
    monster_q: Query<&Transform, (With<MonsterTag>, Without<Player>)>,
) {
    let Ok((p_trans, player)) = player_q.get_single() else { return };
    if player.is_hiding { return; }
    let pp = p_trans.translation.xy();
    for m_trans in monster_q.iter() {
        let mp = m_trans.translation.xy();
        if (mp - pp).length() < player.radius + MONSTER_COLLISION_RADIUS {
            next_state.set(GameState::GameOver);
            return;
        }
    }
}
