//! Fog of War（战争迷雾）
//!
//! 迁移自 python_arcade `src/engine/map/exploration_map.py::FogOfWar`。
//! 与 `darkness.rs` 共存但语义不同：
//! - darkness overlay = 动态光照（手电筒照亮，离开即暗）
//! - Fog of War      = 探索记忆（走过的永久揭示，没走过的永远黑）
//!
//! z 层级：fog=2.5，在 darkness(3~4) 之下——意味着手电筒能刺穿迷雾
//! 照亮未探索区，符合"当前视野大于记忆"的设计。
//!
//! 渲染架构：fog 属游戏画面层，挂在 MainCamera，不挂 UiCamera。
//! 双窗口改造后，游戏画面窗口是纯净正方形，fog 在其中。

use bevy::prelude::*;
use bevy::math::primitives::Circle;
use bevy::sprite::MaterialMesh2dBundle;
use std::collections::HashSet;

use crate::{MainCamera, PlayerTag};
use crate::constants::{TILE_SIZE, WORLD_WIDTH, WORLD_HEIGHT};

// ---------- 资源: 已揭示的 tile 集合 ----------

/// 已揭示的 tile 坐标集合（永久记忆，随存档持久化）
///
/// 用 `HashSet` 快速查重；存档时转 `Vec<[i32;2]>` 序列化。
#[derive(Resource, Debug, Default, Clone)]
pub struct RevealedTiles {
    pub set: HashSet<[i32; 2]>,
}

impl RevealedTiles {
    pub fn reveal(&mut self, grid: [i32; 2]) {
        self.set.insert(grid);
    }

    pub fn is_revealed(&self, grid: &[i32; 2]) -> bool {
        self.set.contains(grid)
    }

    /// 序列化为可存档的 Vec
    pub fn to_vec(&self) -> Vec<[i32; 2]> {
        self.set.iter().copied().collect()
    }

    /// 从存档反序列化
    pub fn from_vec(vec: Vec<[i32; 2]>) -> Self {
        Self {
            set: vec.into_iter().collect(),
        }
    }
}

// ---------- 组件 ----------

/// 单个未揭示 tile 的黑色覆盖块
#[derive(Component, Debug)]
pub struct FogTile {
    pub grid: [i32; 2],
}

/// fog 覆盖层 z 值：在 darkness(3~4) 之下，手电筒可刺穿
const FOG_Z: f32 = 2.5;
/// 揭示半径（玩家周围 N 格）
const REVEAL_RADIUS: i32 = 5;
/// 黑色覆盖的不透明度（留一丝轮廓透出，非纯黑）
const FOG_ALPHA: f32 = 0.92;

// ---------- 系统: 揭示玩家周围 tile ----------

/// 每帧把玩家周围 REVEAL_RADIUS 格的 tile 加入已揭示集合
pub fn fog_reveal_around_player(
    player_q: Query<&Transform, With<PlayerTag>>,
    mut revealed: ResMut<RevealedTiles>,
) {
    let Ok(trans) = player_q.get_single() else { return };
    let px = (trans.translation.x / TILE_SIZE).floor() as i32;
    let py = (trans.translation.y / TILE_SIZE).floor() as i32;

    for dx in -REVEAL_RADIUS..=REVEAL_RADIUS {
        for dy in -REVEAL_RADIUS..=REVEAL_RADIUS {
            // 圆形揭示（距 hypot ≤ radius），而非方形
            if ((dx * dx + dy * dy) as f32).sqrt() <= REVEAL_RADIUS as f32 {
                revealed.reveal([px + dx, py + dy]);
            }
        }
    }
}

// ---------- 系统: 生成未揭示 tile 覆盖块 ----------

/// 为视口内未揭示的 tile 生成黑色覆盖块
///
/// 视口裁剪：只处理相机可见范围内的 tile，避免对整个世界生成。
/// 查重：已存在的 FogTile 不重复 spawn。
pub fn fog_spawn_unrevealed(
    commands: Commands,
    revealed: Res<RevealedTiles>,
    camera_q: Query<&Transform, With<MainCamera>>,
    existing_q: Query<&FogTile>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    server: Res<AssetServer>,
) {
    let Ok(cam_trans) = camera_q.get_single() else { return };

    // 算视口覆盖的 tile 范围（假设视口 = WINDOW 尺寸，双窗口后游戏窗口尺寸）
    // 用 WORLD 尺寸做保守上限，实际视口裁剪由相机 projection 完成
    let half_view_w = WORLD_WIDTH * 0.5;
    let half_view_h = WORLD_HEIGHT * 0.5;

    let min_x = ((cam_trans.translation.x - half_view_w) / TILE_SIZE).floor() as i32;
    let max_x = ((cam_trans.translation.x + half_view_w) / TILE_SIZE).ceil() as i32;
    let min_y = ((cam_trans.translation.y - half_view_h) / TILE_SIZE).floor() as i32;
    let max_y = ((cam_trans.translation.y + half_view_h) / TILE_SIZE).ceil() as i32;

    // 收集已存在的 FogTile grid，避免重复 spawn
    let existing: HashSet<[i32; 2]> = existing_q.iter().map(|f| f.grid).collect();

    let mesh = meshes.add(Mesh::from(Circle { radius: TILE_SIZE * 0.5 }));
    let mat = materials.add(ColorMaterial::from(Color::srgba(0.0, 0.0, 0.0, FOG_ALPHA)));

    let mut cmds = commands;
    for gx in min_x..=max_x {
        for gy in min_y..=max_y {
            let grid = [gx, gy];
            if revealed.is_revealed(&grid) || existing.contains(&grid) {
                continue;
            }
            // 跳过世界范围外的 tile（地图不无限）
            let world_x = gx as f32 * TILE_SIZE;
            let world_y = gy as f32 * TILE_SIZE;
            if world_x < -TILE_SIZE || world_x > WORLD_WIDTH + TILE_SIZE {
                continue;
            }
            if world_y < -TILE_SIZE || world_y > WORLD_HEIGHT + TILE_SIZE {
                continue;
            }
            cmds.spawn((
                MaterialMesh2dBundle {
                    mesh: mesh.clone().into(),
                    material: mat.clone(),
                    transform: Transform::from_xyz(
                        world_x + TILE_SIZE * 0.5,
                        world_y + TILE_SIZE * 0.5,
                        FOG_Z,
                    ),
                    ..default()
                },
                FogTile { grid },
            ));
        }
    }
    // 静默 unused 警告（server 在双窗口改造后会用于加载 fog texture）
    let _ = &server;
}

// ---------- 系统: 清理刚被揭示的覆盖块 ----------

/// 当 tile 被揭示后，despawn 对应的 FogTile 覆盖块
pub fn fog_despawn_revealed(
    revealed: Res<RevealedTiles>,
    mut commands: Commands,
    fog_q: Query<(Entity, &FogTile)>,
) {
    for (entity, fog) in &fog_q {
        if revealed.is_revealed(&fog.grid) {
            commands.entity(entity).despawn();
        }
    }
}

// ---------- 系统: 切换场景时清空 fog ----------

/// OnExit(Playing) 时调用，despawn 所有 FogTile 并清空 RevealedTiles
///
/// 注意：清空 RevealedTiles 仅在新游戏时用；读档时应先恢复再 spawn。
/// 此系统只 despawn entity，RevealedTiles 的清空由 reset_world_state 处理。
pub fn despawn_all_fog(
    mut commands: Commands,
    fog_q: Query<Entity, With<FogTile>>,
) {
    for entity in &fog_q {
        commands.entity(entity).despawn();
    }
}

// ---------- Plugin ----------

pub struct FogOfWarPlugin;

impl Plugin for FogOfWarPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<RevealedTiles>();
    }
}

// ---------- 单元测试 ----------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reveal_and_check() {
        let mut r = RevealedTiles::default();
        r.reveal([1, 2]);
        r.reveal([3, 4]);
        assert!(r.is_revealed(&[1, 2]));
        assert!(r.is_revealed(&[3, 4]));
        assert!(!r.is_revealed(&[5, 6]));
        assert_eq!(r.set.len(), 2);
    }

    #[test]
    fn roundtrip_serde() {
        let mut r = RevealedTiles::default();
        r.reveal([1, 2]);
        r.reveal([-3, 0]);
        r.reveal([100, 200]);
        let vec = r.to_vec();
        let back = RevealedTiles::from_vec(vec);
        assert_eq!(back.set.len(), 3);
        assert!(back.is_revealed(&[1, 2]));
        assert!(back.is_revealed(&[-3, 0]));
        assert!(back.is_revealed(&[100, 200]));
    }

    #[test]
    fn reveal_radius_5_is_circular() {
        // 验证揭示半径 5 格是圆形（边角不应揭示）
        let mut r = RevealedTiles::default();
        // 模拟玩家在 (0, 0)
        for dx in -REVEAL_RADIUS..=REVEAL_RADIUS {
            for dy in -REVEAL_RADIUS..=REVEAL_RADIUS {
                if ((dx * dx + dy * dy) as f32).sqrt() <= REVEAL_RADIUS as f32 {
                    r.reveal([dx, dy]);
                }
            }
        }
        // (5, 5) 距 hypot ≈ 7.07 > 5，不应被揭示
        assert!(!r.is_revealed(&[5, 5]));
        // (5, 0) 距 = 5，应被揭示
        assert!(r.is_revealed(&[5, 0]));
        // (3, 4) 距 = 5，应被揭示
        assert!(r.is_revealed(&[3, 4]));
    }
}
