//! 对话与剧情系统
//! 实现分支对话（JSON配置）、变量驱动（全局状态）、条件判断等功能

use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

// ---------- 数据结构 ----------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DialogueOption {
    pub option_id: String,
    pub text: String,
    #[serde(default)]
    pub translate: bool,
    #[serde(default)]
    pub next_node: Option<String>,
    #[serde(default)]
    pub set_vars: HashMap<String, serde_json::Value>,
    #[serde(default)]
    pub condition: Option<Condition>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Condition {
    Equals { var: String, value: serde_json::Value },
    NotEquals { var: String, value: serde_json::Value },
    GreaterThan { var: String, value: f64 },
    LessThan { var: String, value: f64 },
    Exists { var: String, exists: bool },
    And { conditions: Vec<Condition> },
    Or { conditions: Vec<Condition> },
    Not { condition: Box<Condition> },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DialogueNode {
    pub node_id: String,
    pub text: String,
    #[serde(default)]
    pub translate: bool,
    #[serde(default)]
    pub options: Vec<DialogueOption>,
    #[serde(default)]
    pub set_vars: HashMap<String, serde_json::Value>,
    #[serde(default)]
    pub condition: Option<Condition>,
    #[serde(default)]
    pub fallback_node: Option<String>,
    #[serde(default)]
    pub is_ending: bool,
    #[serde(default)]
    pub ending_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DialogueData {
    pub dialogue_id: String,
    #[serde(default)]
    pub npc: String,
    pub nodes: Vec<DialogueNode>,
}

// ---------- 事件 ----------

#[derive(Event)]
pub struct DialogueStartEvent {
    pub dialogue_id: String,
}

#[derive(Event)]
pub struct DialogueEndEvent;

#[derive(Event)]
pub struct DialogueChoiceEvent {
    pub option_index: usize,
}

#[derive(Event)]
pub struct NarrativeFlagSetEvent {
    pub key: String,
    pub value: serde_json::Value,
}

// ---------- 资源 ----------

/// 剧情变量存储（全局叙事状态）
#[derive(Resource, Default)]
pub struct NarrativeVars {
    vars: HashMap<String, serde_json::Value>,
}

impl NarrativeVars {
    pub fn set(&mut self, key: &str, value: serde_json::Value) {
        self.vars.insert(key.to_string(), value);
    }

    pub fn get(&self, key: &str) -> Option<&serde_json::Value> {
        self.vars.get(key)
    }

    pub fn has(&self, key: &str) -> bool {
        self.vars.contains_key(key)
    }

    pub fn get_string(&self, key: &str) -> Option<String> {
        self.get(key).and_then(|v| v.as_str().map(|s| s.to_string()))
    }

    pub fn get_f64(&self, key: &str) -> Option<f64> {
        self.get(key).and_then(|v| v.as_f64())
    }

    pub fn get_bool(&self, key: &str) -> Option<bool> {
        self.get(key).and_then(|v| v.as_bool())
    }
}

/// 对话管理器资源
#[derive(Resource, Default)]
pub struct DialogueManager {
    dialogues: HashMap<String, DialogueData>,
    current_dialogue: Option<String>,
    current_node: Option<String>,
}

impl DialogueManager {
    pub fn new() -> Self {
        Self::default()
    }

    /// 从文件加载对话
    pub fn load_from_file<P: AsRef<Path>>(&mut self, dialogue_id: &str, path: P) -> Result<(), String> {
        let path = path.as_ref();
        if !path.exists() {
            return Err(format!("对话文件不存在: {:?}", path));
        }
        let json = fs::read_to_string(path).map_err(|e| format!("读取文件失败: {}", e))?;
        let data: DialogueData = serde_json::from_str(&json)
            .map_err(|e| format!("解析对话失败: {}", e))?;
        self.dialogues.insert(dialogue_id.to_string(), data);
        info!("加载对话: {}", dialogue_id);
        Ok(())
    }

    /// 从数据加载对话
    pub fn load_from_data(&mut self, data: DialogueData) {
        let id = data.dialogue_id.clone();
        self.dialogues.insert(id.clone(), data);
        info!("加载对话: {}", id);
    }

    /// 开始对话，返回第一个节点
    pub fn start_dialogue(&mut self, dialogue_id: &str, vars: &mut NarrativeVars) -> Option<DialogueNode> {
        let data = self.dialogues.get(dialogue_id)?.clone();
        self.current_dialogue = Some(dialogue_id.to_string());

        // 找 start 节点，没有就用第一个
        let start_node = data.nodes.iter()
            .find(|n| n.node_id == "start")
            .or_else(|| data.nodes.first())?
            .clone();

        let processed = self.process_node(start_node, vars, dialogue_id);
        processed
    }

    /// 选择选项，返回下一节点
    pub fn select_option(&mut self, option_index: usize, vars: &mut NarrativeVars) -> Option<DialogueNode> {
        let dialogue_id = self.current_dialogue.as_ref()?.clone();
        let node_id = self.current_node.as_ref()?.clone();
        let data = self.dialogues.get(&dialogue_id)?.clone();

        let current_node = data.nodes.iter().find(|n| n.node_id == *node_id)?;

        let option = current_node.options.get(option_index)?;

        // 应用选项的 set_vars
        for (k, v) in &option.set_vars {
            vars.set(k, v.clone());
        }

        let next_id = option.next_node.as_ref()?;
        let next_node = data.nodes.iter().find(|n| &n.node_id == next_id)?.clone();

        self.process_node(next_node, vars, &dialogue_id)
    }

    fn process_node(
        &mut self,
        node: DialogueNode,
        vars: &mut NarrativeVars,
        dialogue_id: &str,
    ) -> Option<DialogueNode> {
        // 检查条件
        if let Some(cond) = &node.condition {
            if !evaluate_condition(cond, vars) {
                if let Some(fallback) = &node.fallback_node {
                    let data = self.dialogues.get(dialogue_id)?;
                    if let Some(fb_node) = data.nodes.iter().find(|n| &n.node_id == fallback) {
                        return self.process_node(fb_node.clone(), vars, dialogue_id);
                    }
                }
                return None;
            }
        }

        // 应用 set_vars
        for (k, v) in &node.set_vars {
            vars.set(k, v.clone());
        }

        // 过滤可见选项
        let visible_options: Vec<DialogueOption> = node.options
            .iter()
            .filter(|opt| {
                opt.condition.as_ref()
                    .map(|c| evaluate_condition(c, vars))
                    .unwrap_or(true)
            })
            .cloned()
            .collect();

        self.current_node = Some(node.node_id.clone());

        Some(DialogueNode {
            options: visible_options,
            ..node
        })
    }

    pub fn end_dialogue(&mut self) {
        self.current_dialogue = None;
        self.current_node = None;
    }

    pub fn is_active(&self) -> bool {
        self.current_dialogue.is_some()
    }

    pub fn current_dialogue_id(&self) -> Option<&str> {
        self.current_dialogue.as_deref()
    }

    pub fn get_npc_name(&self, dialogue_id: &str) -> Option<String> {
        self.dialogues.get(dialogue_id).map(|d| d.npc.clone())
    }
}

/// 评估条件表达式
pub fn evaluate_condition(cond: &Condition, vars: &NarrativeVars) -> bool {
    match cond {
        Condition::Equals { var, value } => {
            vars.get(var).map(|v| v == value).unwrap_or(false)
        }
        Condition::NotEquals { var, value } => {
            vars.get(var).map(|v| v != value).unwrap_or(true)
        }
        Condition::GreaterThan { var, value } => {
            vars.get_f64(var).map(|v| v > *value).unwrap_or(false)
        }
        Condition::LessThan { var, value } => {
            vars.get_f64(var).map(|v| v < *value).unwrap_or(false)
        }
        Condition::Exists { var, exists } => {
            vars.has(var) == *exists
        }
        Condition::And { conditions } => {
            conditions.iter().all(|c| evaluate_condition(c, vars))
        }
        Condition::Or { conditions } => {
            conditions.iter().any(|c| evaluate_condition(c, vars))
        }
        Condition::Not { condition } => {
            !evaluate_condition(condition, vars)
        }
    }
}

// ---------- 系统 ----------

pub struct DialoguePlugin;

impl Plugin for DialoguePlugin {
    fn build(&self, app: &mut App) {
        app
            .init_resource::<DialogueManager>()
            .init_resource::<NarrativeVars>()
            .add_event::<DialogueStartEvent>()
            .add_event::<DialogueEndEvent>()
            .add_event::<DialogueChoiceEvent>()
            .add_event::<NarrativeFlagSetEvent>();
    }
}
