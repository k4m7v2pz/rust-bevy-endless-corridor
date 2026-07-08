//! 瓦片地图系统
//! - tile_map::GameMap: 程序化生成地图 (房间 + 走廊)
//! - tile_map::spawn_map_tiles: 生成瓦片 Sprite 实体
//! - 可行走检测与圆碰撞

use bevy::prelude::*;
use rand::Rng;

use crate::constants::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TileType {
    Wall,
    Room,
    Corridor,
}

impl Default for TileType {
    fn default() -> Self { TileType::Wall }
}

#[derive(Component)]
pub struct MapTile;

#[derive(Resource)]
pub struct GameMap {
    pub tiles: Vec<Vec<TileType>>,
    pub width_tiles: u32,
    pub height_tiles: u32,
    pub exit_position: Vec2,
    pub player_spawn: Vec2,
    pub hiding_spots: Vec<Vec2>,
    pub key_spots: Vec<Vec2>,
    pub monster_spawns: Vec<Vec2>,
}

impl GameMap {
    pub fn generate() -> Self {
        use crate::{WORLD_HEIGHT, WORLD_WIDTH};
        // 将世界限制为稍小一点的区域, 便于看到多个房间
        let world_w = WORLD_WIDTH * MAP_WORLD_SCALE;
        let world_h = WORLD_HEIGHT * MAP_WORLD_SCALE;
        let width_tiles = (world_w / TILE_SIZE) as u32;
        let height_tiles = (world_h / TILE_SIZE) as u32;

        let w = width_tiles as usize;
        let h = height_tiles as usize;
        let mut tiles = vec![vec![TileType::Wall; h]; w];

        let mut rng = rand::thread_rng();

        // 随机挖若干房间
        let mut rooms: Vec<(u32, u32, u32, u32)> = Vec::new();

        for _ in 0..ROOM_GENERATION_ATTEMPTS {
            let hw = rng.gen_range(ROOM_MIN_HALF_WIDTH..=ROOM_MAX_HALF_WIDTH);
            let hh = rng.gen_range(ROOM_MIN_HALF_HEIGHT..=ROOM_MAX_HALF_HEIGHT);
            let cx = rng.gen_range(hw + 2..(w as u32) - hw - 2);
            let cy = rng.gen_range(hh + 2..(h as u32) - hh - 2);

            let mut overlap = false;
            for &(ocx, ocy, ohw, ohh) in &rooms {
                let dx = (cx as i32 - ocx as i32).unsigned_abs();
                let dy = (cy as i32 - ocy as i32).unsigned_abs();
                if dx < hw + ohw + ROOM_MIN_GAP && dy < hh + ohh + ROOM_MIN_GAP {
                    overlap = true;
                    break;
                }
            }
            if overlap { continue; }

            for x in (cx - hw)..=(cx + hw) {
                for y in (cy - hh)..=(cy + hh) {
                    tiles[x as usize][y as usize] = TileType::Room;
                }
            }
            rooms.push((cx, cy, hw, hh));
        }

        if rooms.is_empty() {
            for x in 5..30 {
                for y in 5..30 {
                    if (x as usize) < w && (y as usize) < h {
                        tiles[x as usize][y as usize] = TileType::Room;
                    }
                }
            }
            rooms.push((20, 20, 10, 10));
        }

        // 连接房间
        for i in 1..rooms.len() {
            let (ax, ay, _, _) = rooms[i - 1];
            let (bx, by, _, _) = rooms[i];
            carve_corridor(&mut tiles, w, h, ax, ay, bx, by);
        }
        for _ in 0..rooms.len() / 3 {
            let a = rng.gen_range(0..rooms.len());
            let b = rng.gen_range(0..rooms.len());
            if a != b {
                let (ax, ay, _, _) = rooms[a];
                let (bx, by, _, _) = rooms[b];
                carve_corridor(&mut tiles, w, h, ax, ay, bx, by);
            }
        }

        // 确定关键点位置 (世界坐标)
        let center_room = rooms[rooms.len() / 2];
        let player_spawn = tile_center(center_room.0, center_room.1);

        let last_room = *rooms.last().unwrap();
        let exit_position = tile_center(last_room.0, last_room.1);

        let mut hiding_spots = Vec::new();
        let mut key_spots = Vec::new();
        let mut monster_spawns = Vec::new();

        for (i, &(cx, cy, _, _)) in rooms.iter().enumerate() {
            let center = tile_center(cx, cy);
            if i == rooms.len() / 2 { continue; } // 玩家房间不生成危险物
            hiding_spots.push(center + Vec2::new(-TILE_SIZE, -TILE_SIZE * 0.3));
            if i % 2 == 1 && key_spots.len() < KEYS_REQUIRED as usize {
                key_spots.push(center + Vec2::new(TILE_SIZE * 0.5, TILE_SIZE * 0.5));
            }
            if i % 3 == 2 {
                monster_spawns.push(center + Vec2::new(-TILE_SIZE * 0.8, TILE_SIZE * 0.7));
            }
        }

        // 保证至少 KEYS_REQUIRED 把钥匙
        while key_spots.len() < KEYS_REQUIRED as usize {
            let r = rooms[rng.gen_range(1..rooms.len())];
            key_spots.push(tile_center(r.0, r.1) + Vec2::new(TILE_SIZE, -TILE_SIZE));
        }
        // 保证至少 2 个怪物
        while monster_spawns.len() < 2 {
            let r = rooms[rng.gen_range(1..rooms.len())];
            monster_spawns.push(tile_center(r.0, r.1) + Vec2::new(-TILE_SIZE * 1.5, -TILE_SIZE * 1.5));
        }
        hiding_spots.truncate(MAX_HIDING_SPOTS);

        Self {
            tiles,
            width_tiles,
            height_tiles,
            exit_position,
            player_spawn,
            hiding_spots,
            key_spots,
            monster_spawns,
        }
    }

    pub fn tile_at_world(&self, wx: f32, wy: f32) -> TileType {
        let tx = (wx / TILE_SIZE).floor() as i32;
        let ty = (wy / TILE_SIZE).floor() as i32;
        if tx < 0 || ty < 0 || tx >= self.width_tiles as i32 || ty >= self.height_tiles as i32 {
            return TileType::Wall;
        }
        self.tiles[tx as usize][ty as usize]
    }

    pub fn is_walkable(&self, wx: f32, wy: f32) -> bool {
        matches!(self.tile_at_world(wx, wy), TileType::Room | TileType::Corridor)
    }

    pub fn collide_circle(&self, pos: Vec2, radius: f32) -> Vec2 {
        let mut new_pos = pos;
        
        // 检查四个主要方向
        let checks = [
            (radius, 0.0),   // 右
            (-radius, 0.0),  // 左
            (0.0, radius),   // 上
            (0.0, -radius),  // 下
        ];
        
        for (dx, dy) in checks {
            if !self.is_walkable(new_pos.x + dx, new_pos.y + dy) {
                if dx.abs() > 0.0 {
                    new_pos.x = pos.x;
                }
                if dy.abs() > 0.0 {
                    new_pos.y = pos.y;
                }
            }
        }
        
        // 检查四个对角线方向（更精确的碰撞检测）
        let diagonal_checks = [
            (radius * 0.7, radius * 0.7),   // 右上
            (-radius * 0.7, radius * 0.7),  // 左上
            (radius * 0.7, -radius * 0.7),  // 右下
            (-radius * 0.7, -radius * 0.7), // 左下
        ];
        
        for (dx, dy) in diagonal_checks {
            if !self.is_walkable(new_pos.x + dx, new_pos.y + dy) {
                // 对角线碰撞时，尝试只在一个轴上移动
                if self.is_walkable(pos.x + dx, pos.y) {
                    new_pos.y = pos.y;
                } else if self.is_walkable(pos.x, pos.y + dy) {
                    new_pos.x = pos.x;
                } else {
                    // 两个轴都无法移动，保持原位置
                    new_pos = pos;
                }
            }
        }
        
        new_pos
    }
}

fn tile_center(tx: u32, ty: u32) -> Vec2 {
    Vec2::new(
        tx as f32 * TILE_SIZE + TILE_SIZE * 0.5,
        ty as f32 * TILE_SIZE + TILE_SIZE * 0.5,
    )
}

fn carve_corridor(tiles: &mut [Vec<TileType>], w: usize, h: usize, ax: u32, ay: u32, bx: u32, by: u32) {
    let mut x = ax as i32;
    let mut y = ay as i32;
    let tx = bx as i32;
    let ty = by as i32;
    let horizontal_first: bool = rand::thread_rng().gen();

    let step = |tiles: &mut [Vec<TileType>], x: i32, y: i32| {
        for dx in 0..2 {
            for dy in 0..2 {
                let cx = x + dx;
                let cy = y + dy;
                if cx >= 0 && cy >= 0 && (cx as usize) < w && (cy as usize) < h {
                    tiles[cx as usize][cy as usize] = TileType::Corridor;
                }
            }
        }
    };

    if horizontal_first {
        while x != tx { step(tiles, x, y); x += if x < tx { 1 } else { -1 }; }
        while y != ty { step(tiles, x, y); y += if y < ty { 1 } else { -1 }; }
    } else {
        while y != ty { step(tiles, x, y); y += if y < ty { 1 } else { -1 }; }
        while x != tx { step(tiles, x, y); x += if x < tx { 1 } else { -1 }; }
    }
    step(tiles, tx, ty);
}

/// 生成瓦片 Sprite 实体 (作为游戏世界背景)
pub fn spawn_map_tiles(commands: &mut Commands, map: &GameMap) {
    let tile_half = TILE_SIZE * 0.5;
    for tx in 0..map.width_tiles {
        for ty in 0..map.height_tiles {
            let tile = map.tiles[tx as usize][ty as usize];
            let color = match tile {
                TileType::Room => Color::rgb(0.09, 0.07, 0.06),
                TileType::Corridor => Color::rgb(0.06, 0.05, 0.045),
                TileType::Wall => continue,
            };
            let wx = tx as f32 * TILE_SIZE + tile_half;
            let wy = ty as f32 * TILE_SIZE + tile_half;
            commands.spawn((
                SpriteBundle {
                    sprite: Sprite {
                        color,
                        custom_size: Some(Vec2::new(TILE_SIZE, TILE_SIZE)),
                        ..default()
                    },
                    transform: Transform::from_xyz(wx, wy, 0.0),
                    ..default()
                },
                MapTile,
            ));
        }
    }
}
