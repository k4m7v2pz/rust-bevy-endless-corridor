//! 剧本系统（AI 友好）
//!
//! 迁移自 python_arcade `src/game/scripts/script_loader.py`。
//! 设计目标：
//! - JSON 存储，2 空格缩进（便于 AI 工具与人类直接读写）
//! - 配套 JSON Schema（`configs/schema/script.schema.json`）做结构校验
//! - `metadata.ai_context` / `metadata.creator_hints` 字段保留，供外部 AI 工具消费
//!
//! 与 `dialogue.rs` 并存：
//! - 本模块是线性时间轴演出（旁白 / 对话 / 内心独白三轨，按 index 推进）
//! - `dialogue.rs` 是分支对话图（玩家选选项推进）
//! 两者可共存：演出用于旁白播报，对话用于 NPC 交互。

use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

// ---------- 数据结构 ----------

/// 单条轨道内容（旁白 / 对话 / 内心独白 共用）
///
/// 对应 Python `TrackContent` dataclass。字段保持语义一致。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrackContent {
    #[serde(default)]
    pub content: String,
    #[serde(default = "default_preset")]
    pub preset: String,
    #[serde(default = "default_position")]
    pub position: String,
    #[serde(default)]
    pub character: String,
    #[serde(default = "default_emotion")]
    pub emotion: String,
    #[serde(default = "default_opacity")]
    pub opacity: f32,
}

fn default_preset() -> String { "default".into() }
fn default_position() -> String { "center".into() }
fn default_emotion() -> String { "neutral".into() }
const fn default_opacity() -> f32 { 1.0 }

impl Default for TrackContent {
    fn default() -> Self {
        Self {
            content: String::new(),
            preset: default_preset(),
            position: default_position(),
            character: String::new(),
            emotion: default_emotion(),
            opacity: default_opacity(),
        }
    }
}

/// 事件元数据 —— AI 友好卖点
///
/// `ai_context`：给外部 AI 工具的真实状态提示（不展示给玩家）
/// `creator_hints`：创作者给 AI / 协者的设计意图说明
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct EventMetadata {
    #[serde(default)]
    pub ai_context: serde_json::Value,
    #[serde(default)]
    pub creator_hints: serde_json::Value,
    /// 额外自定义字段（宽松兼容，不破坏 Schema）
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

/// 单个剧本事件
///
/// 对应 Python `ScriptEvent` dataclass。
/// 三轨 narration / dialogue / inner 均可选，按时间轴顺序推进。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScriptEvent {
    pub id: String,
    #[serde(default = "default_trigger")]
    pub trigger: String,
    #[serde(default)]
    pub narration: Option<TrackContent>,
    #[serde(default)]
    pub dialogue: Option<TrackContent>,
    #[serde(default)]
    pub inner: Option<TrackContent>,
    #[serde(default)]
    pub metadata: Option<EventMetadata>,
}

fn default_trigger() -> String { "auto".into() }

impl Default for ScriptEvent {
    fn default() -> Self {
        Self {
            id: String::new(),
            trigger: default_trigger(),
            narration: None,
            dialogue: None,
            inner: None,
            metadata: None,
        }
    }
}

impl ScriptEvent {
    /// 取 AI 上下文（供 debug 命令或外部工具读取）
    pub fn get_ai_context(&self) -> Option<&serde_json::Value> {
        self.metadata.as_ref().map(|m| &m.ai_context)
    }

    /// 取创作者提示
    pub fn get_creator_hints(&self) -> Option<&serde_json::Value> {
        self.metadata.as_ref().map(|m| &m.creator_hints)
    }
}

// ---------- 剧本文件顶层结构 ----------

/// 剧本文件元信息
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ScriptMeta {
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub title: String,
    #[serde(default)]
    pub author: String,
    /// 目标剧本 schema 版本（与 `configs/schema/script.schema.json` 对齐）
    #[serde(default = "default_schema_version")]
    pub schema_version: String,
}

fn default_schema_version() -> String { "1.0.0".into() }

/// 整个剧本文件的反序列化目标
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScriptData {
    #[serde(default)]
    pub meta: ScriptMeta,
    pub timeline: Vec<ScriptEvent>,
}

// ---------- 加载器 ----------

/// 剧本加载结果
#[derive(Debug)]
pub struct LoadedScript {
    pub data: ScriptData,
    pub schema_path: Option<std::path::PathBuf>,
}

/// 加载并校验剧本文件
///
/// 校验流程（对应 Python `ScriptLoader._validate`）：
/// 1. 读 JSON 文件 → 反序列化
/// 2. 检查 `meta` 与 `meta.id` 存在
/// 3. 检查 `timeline` 存在且为数组
/// 4. 每个 timeline 元素必须有 `id`
/// 5. 若提供 `schema_path`，做 JSON Schema 结构校验（见下方说明）
///
/// JSON Schema 校验说明：本仓库不引入 `jsonschema` 重依赖（会拉一大堆 crate），
/// 改用轻量结构校验 + Schema 文件供外部工具（编辑器 / AI）静态检查。
/// 如需运行时严格校验，可在 `configs/schema/` 放 schema 后用外部工具预检。
pub fn load_script_file(path: &Path, schema_path: Option<&Path>) -> Result<LoadedScript, String> {
    if !path.exists() {
        return Err(format!("剧本文件不存在: {:?}", path));
    }
    let json = std::fs::read_to_string(path).map_err(|e| format!("读取文件失败: {}", e))?;
    let data: ScriptData = serde_json::from_str(&json)
        .map_err(|e| format!("解析剧本失败: {}", e))?;

    // 结构校验（对应 Python _validate）
    let mut errors: Vec<String> = Vec::new();
    if data.meta.id.is_empty() {
        errors.push("meta.id 缺失或为空".into());
    }
    if data.timeline.is_empty() {
        errors.push("timeline 为空或缺失".into());
    } else {
        for (i, ev) in data.timeline.iter().enumerate() {
            if ev.id.is_empty() {
                errors.push(format!("timeline[{}].id 缺失", i));
            }
        }
    }
    if !errors.is_empty() {
        return Err(format!("剧本校验失败:\n{}", errors.join("\n")));
    }

    Ok(LoadedScript {
        data,
        schema_path: schema_path.map(|p| p.to_path_buf()),
    })
}

/// 把剧本数据写回 JSON 文件，2 空格缩进（AI 友好）
pub fn save_script_file(path: &Path, data: &ScriptData) -> Result<(), String> {
    let json = serde_json::to_string_pretty(data)
        .map_err(|e| format!("序列化剧本失败: {}", e))?;
    // serde_json::to_string_pretty 默认 2 空格缩进，符合 AI 友好要求
    std::fs::write(path, json).map_err(|e| format!("写入文件失败: {}", e))?;
    Ok(())
}

// ---------- Bevy 资源: 时间轴播放器 ----------

/// 当前正在播放的剧本时间轴
///
/// 对应 Python `ScriptPlayer`。作为 Bevy Resource 存放，
/// 由 `narrative_advance_system` 按 trigger 推进。
#[derive(Resource, Debug)]
pub struct ScriptPlayer {
    pub script: ScriptData,
    pub current_index: usize,
    pub is_playing: bool,
}

impl ScriptPlayer {
    pub fn new(script: ScriptData) -> Self {
        Self {
            script,
            current_index: 0,
            is_playing: false,
        }
    }

    /// 当前事件
    pub fn current_event(&self) -> Option<&ScriptEvent> {
        self.script.timeline.get(self.current_index)
    }

    /// 推进到下一个事件，返回是否还有后续
    pub fn advance(&mut self) -> bool {
        if self.current_index + 1 < self.script.timeline.len() {
            self.current_index += 1;
            true
        } else {
            false
        }
    }

    /// 跳转到指定 id 的事件
    pub fn jump_to(&mut self, event_id: &str) -> bool {
        if let Some(i) = self.script.timeline.iter().position(|e| e.id == event_id) {
            self.current_index = i;
            true
        } else {
            false
        }
    }

    pub fn reset(&mut self) {
        self.current_index = 0;
        self.is_playing = false;
    }

    /// 进度 (current, total)
    pub fn progress(&self) -> (usize, usize) {
        (self.current_index + 1, self.script.timeline.len())
    }
}

// ---------- 事件 ----------

/// 时间轴推进到新事件时发出（供 UI / 通知系统订阅）
#[derive(Event, Debug, Clone)]
pub struct NarrativeAdvanceEvent {
    pub event_id: String,
    pub trigger: String,
}

/// 时间轴播放完毕
#[derive(Event, Debug, Clone, Default)]
pub struct NarrativeFinishedEvent;

// ---------- Plugin ----------

/// 剧本系统插件
///
/// 注册资源与系统。剧本加载由 `load_script_file` 显式调用，
/// 加载完成后把 `ScriptPlayer` insert 到 world 即可开始播放。
pub struct NarrativePlugin;

impl Plugin for NarrativePlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<NarrativeAdvanceEvent>()
            .add_event::<NarrativeFinishedEvent>()
            .add_systems(Update, narrative_advance_system);
    }
}

/// 时间轴推进系统
///
/// 当 `ScriptPlayer` 资源存在且 `is_playing` 为真时，按 trigger=="auto" 自动推进。
/// 其他 trigger 值（如 "on_enter_room"）应由外部系统调用 `ScriptPlayer::advance`。
///
/// 推进节流：每事件至少停留 1 帧，避免单帧刷完整个时间轴。
/// 严格节流由各事件的"持续时间"决定（本版未实现 per-event duration，
/// 如需可在 `TrackContent` 加 `duration` 字段后扩展）。
fn narrative_advance_system(
    mut player: ResMut<ScriptPlayer>,
    mut advance_evts: EventWriter<NarrativeAdvanceEvent>,
    mut finished_evts: EventWriter<NarrativeFinishedEvent>,
) {
    if !player.is_playing {
        return;
    }
    // 发当前事件
    if let Some(ev) = player.current_event().cloned() {
        advance_evts.send(NarrativeAdvanceEvent {
            event_id: ev.id.clone(),
            trigger: ev.trigger.clone(),
        });
    }
    // auto 触发则推进；其他 trigger 留给外部系统
    let is_auto = player
        .current_event()
        .map(|e| e.trigger == "auto")
        .unwrap_or(false);
    if is_auto {
        if !player.advance() {
            player.is_playing = false;
            finished_evts.send(NarrativeFinishedEvent);
        }
    }
}

// ---------- 单元测试 ----------

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_script() -> ScriptData {
        ScriptData {
            meta: ScriptMeta {
                id: "test".into(),
                title: "测试剧本".into(),
                author: " tester".into(),
                schema_version: "1.0.0".into(),
            },
            timeline: vec![
                ScriptEvent {
                    id: "ev1".into(),
                    trigger: "auto".into(),
                    narration: Some(TrackContent {
                        content: "旁白文本".into(),
                        ..default()
                    }),
                    dialogue: None,
                    inner: None,
                    metadata: Some(EventMetadata {
                        ai_context: serde_json::json!({"true_state": "hidden"}),
                        creator_hints: serde_json::json!({"intent": "atmosphere"}),
                        extra: HashMap::new(),
                    }),
                },
                ScriptEvent {
                    id: "ev2".into(),
                    trigger: "on_enter".into(),
                    ..Default::default()
                },
            ],
        }
    }

    #[test]
    fn script_roundtrip_preserves_2space_indent() {
        let script = sample_script();
        let json = serde_json::to_string_pretty(&script).unwrap();
        // 确认 2 空格缩进（serde_json 默认）
        assert!(json.contains("\n  \"meta\""), "应为 2 空格缩进");
        // 往返不丢字段
        let back: ScriptData = serde_json::from_str(&json).unwrap();
        assert_eq!(back.timeline.len(), 2);
        assert_eq!(back.timeline[0].id, "ev1");
    }

    #[test]
    fn script_player_advance_and_jump() {
        let mut p = ScriptPlayer::new(sample_script());
        assert_eq!(p.progress(), (1, 2));
        assert!(p.advance());
        assert_eq!(p.progress(), (2, 2));
        assert!(!p.advance()); // 到尾
        p.jump_to("ev1");
        assert_eq!(p.current_index, 0);
    }

    #[test]
    fn ai_context_accessible() {
        let script = sample_script();
        let ctx = script.timeline[0].get_ai_context();
        assert!(ctx.is_some());
        assert_eq!(ctx.unwrap()["true_state"], "hidden");
    }

    #[test]
    fn validate_rejects_empty_timeline() {
        let bad = ScriptData {
            meta: ScriptMeta::default(),
            timeline: vec![],
        };
        let json = serde_json::to_string(&bad).unwrap();
        let tmp = std::env::temp_dir().join("ec_test_script.json");
        std::fs::write(&tmp, json).unwrap();
        let err = load_script_file(&tmp, None).unwrap_err();
        assert!(err.contains("timeline 为空"));
        std::fs::remove_file(&tmp).ok();
    }
}
