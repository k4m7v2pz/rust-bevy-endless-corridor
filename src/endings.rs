//! 结局系统
//! 实现结局解锁记录、进度统计和持久化

use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

// ---------- 数据结构 ----------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EndingDefinition {
    pub ending_id: String,
    pub name: String,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EndingRecord {
    pub ending_id: String,
    pub name: String,
    pub description: String,
    #[serde(default)]
    pub unlocked: bool,
    #[serde(default)]
    pub unlocked_at: Option<String>,
    #[serde(default)]
    pub play_count: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct EndingsData {
    definitions: Vec<EndingDefinition>,
    records: Vec<EndingRecord>,
}

// ---------- 事件 ----------

#[derive(Event)]
pub struct EndingUnlockedEvent {
    pub ending_id: String,
    pub name: String,
    pub description: String,
    pub first_unlock: bool,
}

// ---------- 资源 ----------

#[derive(Resource)]
pub struct EndingManager {
    save_dir: PathBuf,
    definitions: HashMap<String, EndingDefinition>,
    records: HashMap<String, EndingRecord>,
}

impl EndingManager {
    pub fn new(save_dir: impl Into<PathBuf>) -> Self {
        let save_dir = save_dir.into();
        let mut mgr = Self {
            save_dir,
            definitions: HashMap::new(),
            records: HashMap::new(),
        };
        mgr.load();
        mgr
    }

    pub fn register_ending(&mut self, ending_id: &str, name: &str, description: &str) {
        let def = EndingDefinition {
            ending_id: ending_id.to_string(),
            name: name.to_string(),
            description: description.to_string(),
        };
        self.definitions.insert(ending_id.to_string(), def.clone());

        if !self.records.contains_key(ending_id) {
            self.records.insert(ending_id.to_string(), EndingRecord {
                ending_id: ending_id.to_string(),
                name: name.to_string(),
                description: description.to_string(),
                unlocked: false,
                unlocked_at: None,
                play_count: 0,
            });
        }

        self.save();
        info!("注册结局: {} ({})", name, ending_id);
    }

    pub fn unlock_ending(&mut self, ending_id: &str) -> bool {
        let definition = if let Some(def) = self.definitions.get(ending_id) {
            def.clone()
        } else {
            warn!("尝试解锁未知结局: {}", ending_id);
            return false;
        };

        let record = self.records.entry(ending_id.to_string())
            .or_insert_with(|| EndingRecord {
                ending_id: ending_id.to_string(),
                name: definition.name.clone(),
                description: definition.description.clone(),
                unlocked: false,
                unlocked_at: None,
                play_count: 0,
            });

        let first = !record.unlocked;

        if first {
            record.unlocked = true;
            record.unlocked_at = Some(chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string());
            info!("新结局解锁! {}", record.name);
        }

        record.play_count += 1;
        self.save();

        first
    }

    pub fn is_unlocked(&self, ending_id: &str) -> bool {
        self.records.get(ending_id).map(|r| r.unlocked).unwrap_or(false)
    }

    pub fn get_all_records(&self) -> Vec<&EndingRecord> {
        let mut list: Vec<_> = self.records.values().collect();
        list.sort_by(|a, b| a.ending_id.cmp(&b.ending_id));
        list
    }

    pub fn unlocked_count(&self) -> usize {
        self.records.values().filter(|r| r.unlocked).count()
    }

    pub fn total_count(&self) -> usize {
        self.definitions.len()
    }

    pub fn progress_text(&self) -> String {
        let u = self.unlocked_count();
        let t = self.total_count();
        let p = if t > 0 { u as f32 / t as f32 * 100.0 } else { 0.0 };
        format!("{}/{} ({:.1}%)", u, t, p)
    }

    fn load(&mut self) {
        let path = self.save_dir.join("endings.json");
        if !path.exists() {
            return;
        }

        if let Ok(json) = fs::read_to_string(&path) {
            if let Ok(data) = serde_json::from_str::<EndingsData>(&json) {
                for def in data.definitions {
                    self.definitions.insert(def.ending_id.clone(), def);
                }
                for rec in data.records {
                    self.records.insert(rec.ending_id.clone(), rec);
                }
                info!("加载结局数据: {} 个记录", self.records.len());
            }
        }
    }

    fn save(&self) {
        if let Err(e) = fs::create_dir_all(&self.save_dir) {
            error!("创建保存目录失败: {}", e);
            return;
        }

        let data = EndingsData {
            definitions: self.definitions.values().cloned().collect(),
            records: self.records.values().cloned().collect(),
        };

        let path = self.save_dir.join("endings.json");
        match serde_json::to_string_pretty(&data) {
            Ok(json) => {
                if let Err(e) = fs::write(&path, json) {
                    error!("保存结局数据失败: {}", e);
                }
            }
            Err(e) => error!("序列化结局数据失败: {}", e),
        }
    }
}

// ---------- 插件 ----------

pub struct EndingsPlugin {
    pub save_dir: PathBuf,
}

impl Default for EndingsPlugin {
    fn default() -> Self {
        Self {
            save_dir: PathBuf::from("saves"),
        }
    }
}

impl Plugin for EndingsPlugin {
    fn build(&self, app: &mut App) {
        let mut manager = EndingManager::new(self.save_dir.clone());

        // 注册默认结局
        manager.register_ending("normal_ending", "普通结局", "经历了一段精彩的旅程后，你回到了家乡。");
        manager.register_ending("true_ending", "真结局", "你收集了所有的圣物，并获得了人们的认可，成为了传奇。");
        manager.register_ending("tragic_ending", "悲剧结局", "在冒险中经历了太多次失败，但你的勇气永远被铭记。");
        manager.register_ending("secret_ending", "隐藏结局", "你发现了不为人知的秘密，开启了全新的冒险篇章。");

        app
            .insert_resource(manager)
            .add_event::<EndingUnlockedEvent>();
    }
}
