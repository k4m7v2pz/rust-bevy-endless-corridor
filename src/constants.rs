//! 游戏常量配置 - 集中管理所有游戏参数

// === 窗口与世界 ===
pub const WINDOW_WIDTH: f32 = 1280.0;
pub const WINDOW_HEIGHT: f32 = 720.0;
pub const WORLD_WIDTH: f32 = WINDOW_WIDTH * 3.0;
pub const WORLD_HEIGHT: f32 = WINDOW_HEIGHT * 3.0;

// === 玩家 ===
pub const PLAYER_RADIUS: f32 = 14.0;
pub const PLAYER_SPEED: f32 = 260.0;
pub const PLAYER_HIDE_SPEED_MOD: f32 = 0.4;
pub const PLAYER_SIZE: f32 = 26.0;
pub const PLAYER_FLASHLIGHT_RADIUS: f32 = 360.0;
pub const PLAYER_FLASHLIGHT_HALF_ANGLE: f32 = std::f32::consts::PI / 4.0;
pub const PLAYER_HIDE_DISTANCE: f32 = 50.0; // 需要靠近躲藏点的距离

// === 怪物 ===
pub const MONSTER_RADIUS: f32 = 16.0;
pub const MONSTER_SIZE: f32 = 32.0;
pub const MONSTER_BASE_SPEED: f32 = 140.0;
pub const MONSTER_MAX_SPEED: f32 = 260.0;
pub const MONSTER_VISION_RADIUS: f32 = 340.0;
pub const MONSTER_HEARING_RADIUS: f32 = 420.0;
pub const MONSTER_HIDE_VISION_MOD: f32 = 0.12;
pub const MONSTER_HIDE_HEARING_MOD: f32 = 0.08;
pub const MONSTER_PATROL_SPEED_MOD: f32 = 0.6;
pub const MONSTER_SEARCH_SPEED_MOD: f32 = 0.8;
pub const MONSTER_SEARCH_TIMEOUT: f32 = 5.0;
pub const MONSTER_CHASE_TIMEOUT: f32 = 1.5;
pub const MONSTER_PATROL_POINTS_COUNT: usize = 4;
pub const MONSTER_PATROL_MIN_DIST: f32 = 150.0;
pub const MONSTER_PATROL_MAX_DIST: f32 = 280.0;
pub const MONSTER_SEARCH_POINTS_COUNT: usize = 6;
pub const MONSTER_SEARCH_MIN_DIST: f32 = 80.0;
pub const MONSTER_SEARCH_MAX_DIST: f32 = 180.0;
pub const MONSTER_COLLISION_RADIUS: f32 = 12.0;

// === 物品 ===
pub const KEY_SIZE: f32 = 22.0;
pub const KEY_GLOW_SIZE: f32 = 40.0;
pub const KEY_COLLECT_DISTANCE: f32 = 22.0;
pub const EXIT_DOOR_SIZE: f32 = 48.0;
pub const EXIT_DOOR_GLOW_SIZE: f32 = 72.0;
pub const EXIT_DOOR_ENTER_DISTANCE: f32 = 36.0;
pub const HIDING_SPOT_SIZE: f32 = 56.0;
pub const KEYS_REQUIRED: u32 = 3;

// === 地图 ===
pub const TILE_SIZE: f32 = 64.0;
pub const MAP_WORLD_SCALE: f32 = 0.7;
pub const ROOM_MIN_HALF_WIDTH: u32 = 4;
pub const ROOM_MAX_HALF_WIDTH: u32 = 7;
pub const ROOM_MIN_HALF_HEIGHT: u32 = 4;
pub const ROOM_MAX_HALF_HEIGHT: u32 = 7;
pub const ROOM_GENERATION_ATTEMPTS: usize = 30;
pub const ROOM_MIN_GAP: u32 = 3;
pub const MAX_HIDING_SPOTS: usize = 6;

// === 恐惧与理智 ===
pub const FEAR_RADIUS: f32 = 300.0;
pub const FEAR_INCREASE_RATE: f32 = 28.0;
pub const FEAR_DECREASE_RATE: f32 = 6.0;
pub const FEAR_MAX: f32 = 100.0;
pub const FEAR_SANITY_THRESHOLD: f32 = 60.0;
pub const SANITY_DECREASE_RATE: f32 = 4.0;
pub const SANITY_RECOVERY_HIDE_DIST: f32 = 250.0;
pub const SANITY_RECOVERY_HIDE_RATE: f32 = 15.0;
pub const SANITY_RECOVERY_FAR_DIST: f32 = 400.0;
pub const SANITY_RECOVERY_FAR_RATE: f32 = 3.0;
pub const SANITY_MAX: f32 = 100.0;
pub const SANITY_HALLUCINATION_THRESHOLD: f32 = 30.0;
pub const HALLUCINATION_MIN_DIST: f32 = 150.0;
pub const HALLUCINATION_MAX_DIST: f32 = 320.0;
pub const HALLUCINATION_MIN_LIFETIME: f32 = 1.0;
pub const HALLUCINATION_MAX_LIFETIME: f32 = 2.5;
pub const HALLUCINATION_MAX_CHANCE: f32 = 0.7;
pub const HALLUCINATION_SIZE: f32 = 22.0;
pub const SCREEN_SHAKE_MAX_INTENSITY: f32 = 10.0;

// === 黑暗 ===
pub const DARKNESS_AMBIENT_RADIUS: f32 = 180.0;
pub const DARKNESS_CONE_SEGMENTS: u32 = 32;
pub const DARKNESS_Z_AMBIENT: f32 = 3.0;
pub const DARKNESS_Z_FLASHLIGHT: f32 = 4.0;
pub const DARKNESS_Z_MONSTER: f32 = 3.5;

// === 相机 ===
pub const CAMERA_Z: f32 = 1000.0;
pub const CAMERA_FOLLOW_LERP: f32 = 0.12;