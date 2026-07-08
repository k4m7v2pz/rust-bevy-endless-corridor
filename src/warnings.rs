//! 玩家友好警告系统
//! 光敏性癫痫警告、惊吓提醒、健康忠告等

use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

// ---------- 数据结构 ----------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WarningItem {
    pub id: String,
    pub title: String,
    pub description: String,
    #[serde(default = "default_true")]
    pub enabled: bool,
}

fn default_true() -> bool { true }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthAdvice {
    #[serde(default = "default_true")]
    pub enable_china_mainland: bool,
    #[serde(default = "default_health_text")]
    pub text: String,
    #[serde(default)]
    pub position: String,
}

fn default_health_text() -> String {
    "抵制不良游戏，拒接盗版游戏，注意自我保护，谨防受骗上当。适度游戏益脑，沉迷游戏伤身，合理安排时间，享受健康生活。".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WarningGlobalSettings {
    #[serde(default = "default_true")]
    pub enable_warnings: bool,
    #[serde(default = "default_lang")]
    pub default_language: String,
}

fn default_lang() -> String { "zh_CN".to_string() }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WarningConfigData {
    #[serde(default)]
    pub global: WarningGlobalSettings,
    #[serde(default)]
    pub warnings: Vec<WarningItem>,
    #[serde(default)]
    pub health_advice: HealthAdvice,
}

impl Default for WarningGlobalSettings {
    fn default() -> Self {
        Self {
            enable_warnings: true,
            default_language: "zh_CN".to_string(),
        }
    }
}

impl Default for HealthAdvice {
    fn default() -> Self {
        Self {
            enable_china_mainland: true,
            text: default_health_text(),
            position: "after_warnings".to_string(),
        }
    }
}

impl Default for WarningConfigData {
    fn default() -> Self {
        Self {
            global: WarningGlobalSettings::default(),
            warnings: vec![
                WarningItem {
                    id: "epilepsy".to_string(),
                    title: "光敏性癫痫警告".to_string(),
                    description: "该游戏包含闪烁图像，可能引发光敏性癫痫，请有相关病史的玩家谨慎游玩。".to_string(),
                    enabled: true,
                },
                WarningItem {
                    id: "scare".to_string(),
                    title: "惊吓提醒".to_string(),
                    description: "游戏中包含突然出现的惊吓元素，可能会造成心理不适。".to_string(),
                    enabled: true,
                },
                WarningItem {
                    id: "bloody".to_string(),
                    title: "血腥暴力提醒".to_string(),
                    description: "游戏中包含血腥暴力元素，请根据个人承受能力游玩。".to_string(),
                    enabled: true,
                },
            ],
            health_advice: HealthAdvice::default(),
        }
    }
}

// ---------- 资源 ----------

#[derive(Resource)]
pub struct WarningConfig {
    pub config_path: PathBuf,
    pub data: WarningConfigData,
    pub dismissed: bool,
}

impl WarningConfig {
    pub fn new(config_path: impl Into<PathBuf>) -> Self {
        let config_path = config_path.into();
        let data = Self::load_or_default(&config_path);
        Self {
            config_path,
            data,
            dismissed: false,
        }
    }

    fn load_or_default(path: &PathBuf) -> WarningConfigData {
        if path.exists() {
            if let Ok(json) = fs::read_to_string(path) {
                if let Ok(data) = serde_json::from_str::<WarningConfigData>(&json) {
                    return data;
                }
            }
        }

        // 创建默认配置
        let default = WarningConfigData::default();
        if let Some(parent) = path.parent() {
            let _ = fs::create_dir_all(parent);
        }
        if let Ok(json) = serde_json::to_string_pretty(&default) {
            let _ = fs::write(path, json);
        }
        default
    }

    pub fn active_warnings(&self) -> Vec<&WarningItem> {
        self.data.warnings.iter().filter(|w| w.enabled).collect()
    }

    pub fn should_show_health_advice(&self) -> bool {
        self.data.health_advice.enable_china_mainland
    }

    pub fn health_advice_text(&self) -> &str {
        &self.data.health_advice.text
    }
}

// ---------- 组件 ----------

#[derive(Component)]
pub struct WarningScreenTag;

// ---------- 插件 ----------

pub struct WarningPlugin {
    pub config_path: PathBuf,
}

impl Default for WarningPlugin {
    fn default() -> Self {
        Self {
            config_path: PathBuf::from("configs/warning_config.json"),
        }
    }
}

impl Plugin for WarningPlugin {
    fn build(&self, app: &mut App) {
        let config = WarningConfig::new(self.config_path.clone());
        app.insert_resource(config);
    }
}
