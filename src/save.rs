//! 存档系统
//! 
//! 核心特性:
//! - 7位小写哈希ID唯一标识
//! - 双时间维度（游戏内时长/现实时间）

use serde::{Deserialize, Serialize};
use sha2::{Sha256, Digest};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use chrono::{Local, Datelike, Timelike};
use bevy::prelude::*;

/// 存档数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SaveData {
    /// 存档唯一ID (7位小写十六进制)
    pub save_id: String,
    /// 游戏内累计时长 (格式: "Xh Ym")
    pub game_duration: String,
    /// 现实保存时间 (格式: "YYYY-MM-DD HH:mm")
    pub real_time: String,
    /// 游戏状态快照
    pub game_state: GameSnapshot,
}

/// 游戏状态快照
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameSnapshot {
    /// 玩家位置 X
    pub player_x: f32,
    /// 玩家位置 Y
    pub player_y: f32,
    /// 玩家恐惧等级
    pub fear_level: f32,
    /// 玩家理智值
    pub sanity: f32,
    /// 已收集钥匙数量
    pub keys_collected: u32,
    /// 怪物状态快照
    pub monsters: Vec<MonsterSnapshot>,
    /// 当前游戏时间（秒）
    pub game_time_seconds: f32,
    /// 玩家累计死亡次数
    #[serde(default)]
    pub death_count: u32,
    /// 最近死亡原因
    #[serde(default)]
    pub last_death_reason: String,
    /// 最近检查点（X, Y）
    #[serde(default)]
    pub last_checkpoint: Option<[f32; 2]>,
    /// 已揭示的 tile 坐标（Fog of War 探索记忆）
    #[serde(default)]
    pub revealed_tiles: Vec<[i32; 2]>,
}

/// 怪物状态快照
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonsterSnapshot {
    /// 怪物位置 X
    pub x: f32,
    /// 怪物位置 Y
    pub y: f32,
    /// 怪物状态 (0=巡逻, 1=追逐, 2=搜索)
    pub state: u8,
}

/// 存档管理器
pub struct SaveManager {
    /// 存档目录
    save_dir: PathBuf,
}

impl SaveManager {
    pub fn new(save_dir: PathBuf) -> Self {
        if !save_dir.exists() {
            fs::create_dir_all(&save_dir).ok();
        }
        Self { save_dir }
    }

    /// 创建存档
    pub fn create_save(&self, snapshot: GameSnapshot, duration_seconds: f32) -> Result<SaveData, String> {
        // 生成7位哈希ID
        let save_id = Self::generate_save_hash(&snapshot);
        
        // 格式化时长
        let hours = (duration_seconds / 3600.0) as u32;
        let minutes = ((duration_seconds % 3600.0) / 60.0) as u32;
        let game_duration = format!("{}h {}m", hours, minutes);
        
        // 现实时间
        let now = Local::now();
        let real_time = format!("{}-{:02}-{:02} {:02}:{:02}",
            now.year(), now.month(), now.day(), now.hour(), now.minute());
        
        let save_data = SaveData {
            save_id: save_id.clone(),
            game_duration,
            real_time,
            game_state: snapshot,
        };
        
        // 保存到文件
        let file_path = self.save_dir.join(format!("{}.json", save_id));
        let json = serde_json::to_string_pretty(&save_data)
            .map_err(|e| format!("序列化失败: {}", e))?;
        fs::write(&file_path, json)
            .map_err(|e| format!("写入文件失败: {}", e))?;
        
        info!("存档创建成功: {}", save_id);
        Ok(save_data)
    }

    /// 加载存档
    pub fn load_save(&self, save_id: &str) -> Result<SaveData, String> {
        let file_path = self.save_dir.join(format!("{}.json", save_id));
        if !file_path.exists() {
            return Err(format!("存档文件不存在: {}", save_id));
        }
        let json = fs::read_to_string(&file_path)
            .map_err(|e| format!("读取文件失败: {}", e))?;
        serde_json::from_str(&json)
            .map_err(|e| format!("解析存档失败: {}", e))
    }

    /// 删除存档
    pub fn delete_save(&self, save_id: &str) -> Result<(), String> {
        let file_path = self.save_dir.join(format!("{}.json", save_id));
        if file_path.exists() {
            fs::remove_file(&file_path)
                .map_err(|e| format!("删除存档失败: {}", e))?;
            info!("存档已删除: {}", save_id);
        }
        Ok(())
    }

    /// 列出所有存档
    pub fn list_saves(&self) -> Vec<SaveInfo> {
        let mut saves = Vec::new();
        if let Ok(entries) = fs::read_dir(&self.save_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().map_or(false, |e| e == "json") {
                    if let Ok(data) = fs::read_to_string(&path) {
                        if let Ok(save) = serde_json::from_str::<SaveData>(&data) {
                            saves.push(SaveInfo {
                                save_id: save.save_id,
                                game_duration: save.game_duration,
                                real_time: save.real_time,
                            });
                        }
                    }
                }
            }
        }
        // 按时间倒序排列
        saves.sort_by(|a, b| b.real_time.cmp(&a.real_time));
        saves
    }

    /// 生成7位哈希ID
    fn generate_save_hash(snapshot: &GameSnapshot) -> String {
        // 构建哈希数据
        let monster_data: String = snapshot.monsters.iter()
            .map(|m| format!("({},{},{})", m.x, m.y, m.state))
            .collect();
        
        let hash_data = format!(
            "{},{},{},{},{},{},{},{}",
            snapshot.player_x,
            snapshot.player_y,
            snapshot.fear_level,
            snapshot.sanity,
            snapshot.keys_collected,
            snapshot.death_count,
            snapshot.last_death_reason,
            monster_data
        );
        
        // 计算 SHA-256
        let mut hasher = Sha256::new();
        hasher.update(hash_data.as_bytes());
        let result = hasher.finalize();
        
        // 取前7位小写十六进制
        result[..7].iter()
            .map(|b| format!("{:02x}", b))
            .collect::<String>()
    }
}

/// 存档信息（用于列表显示）
#[derive(Debug, Clone)]
pub struct SaveInfo {
    pub save_id: String,
    pub game_duration: String,
    pub real_time: String,
}

/// 存档场景名称
pub const SCENE_NAME: &str = "无尽回廊";

/// 进度名（目前单存档槽，固定名称）
pub const SLOT_NAME: &str = "喜羊羊";

/// 从 WorldState 创建快照
pub fn create_snapshot_from_state(
    player_pos: Vec2,
    fear_level: f32,
    sanity: f32,
    keys_collected: u32,
    monster_positions: &[(Vec2, u8)], // (位置, 状态)
    game_time_seconds: f32,
    death_count: u32,
    last_death_reason: &str,
    last_checkpoint: Option<Vec2>,
    revealed_tiles: &[[i32; 2]],
) -> GameSnapshot {
    GameSnapshot {
        player_x: player_pos.x,
        player_y: player_pos.y,
        fear_level,
        sanity,
        keys_collected,
        monsters: monster_positions.iter().map(|(pos, state)| MonsterSnapshot {
            x: pos.x,
            y: pos.y,
            state: *state,
        }).collect(),
        game_time_seconds,
        death_count,
        last_death_reason: last_death_reason.to_string(),
        last_checkpoint: last_checkpoint.map(|p| [p.x, p.y]),
        revealed_tiles: revealed_tiles.to_vec(),
    }
}
