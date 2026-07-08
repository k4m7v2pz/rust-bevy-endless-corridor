//! 声音提示系统
//! 环境音效引导、空间音频计算、智能触发机制

use bevy::prelude::*;
use std::collections::HashMap;
use std::time::Duration;

// ---------- 声音类型 ----------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SoundCueType {
    Footstep,
    WaterDrip,
    Wind,
    Heartbeat,
    DoorCreak,
    MonsterGrowl,
    Whisper,
}

impl SoundCueType {
    pub fn label(&self) -> &'static str {
        match self {
            SoundCueType::Footstep => "footstep",
            SoundCueType::WaterDrip => "water_drip",
            SoundCueType::Wind => "wind",
            SoundCueType::Heartbeat => "heartbeat",
            SoundCueType::DoorCreak => "door_creak",
            SoundCueType::MonsterGrowl => "monster_growl",
            SoundCueType::Whisper => "whisper",
        }
    }

    pub fn max_distance(&self) -> f32 {
        match self {
            SoundCueType::Footstep => 200.0,
            SoundCueType::WaterDrip => 150.0,
            SoundCueType::Wind => 400.0,
            SoundCueType::Heartbeat => 100.0,
            SoundCueType::DoorCreak => 250.0,
            SoundCueType::MonsterGrowl => 350.0,
            SoundCueType::Whisper => 120.0,
        }
    }

    pub fn cooldown_secs(&self) -> f32 {
        match self {
            SoundCueType::Footstep => 0.4,
            SoundCueType::WaterDrip => 3.0,
            SoundCueType::Wind => 5.0,
            SoundCueType::Heartbeat => 1.5,
            SoundCueType::DoorCreak => 8.0,
            SoundCueType::MonsterGrowl => 4.0,
            SoundCueType::Whisper => 6.0,
        }
    }
}

// ---------- 声音源 ----------

#[derive(Component, Clone)]
pub struct SoundSource {
    pub cue_type: SoundCueType,
    pub position: Vec2,
    pub volume: f32,
    pub auto_trigger: bool,
    pub trigger_radius: f32,
}

impl Default for SoundSource {
    fn default() -> Self {
        Self {
            cue_type: SoundCueType::WaterDrip,
            position: Vec2::ZERO,
            volume: 1.0,
            auto_trigger: true,
            trigger_radius: 200.0,
        }
    }
}

// ---------- 资源 ----------

#[derive(Resource, Default)]
pub struct SoundCueManager {
    cooldowns: HashMap<SoundCueType, f32>,
}

impl SoundCueManager {
    pub fn can_play(&self, cue_type: SoundCueType) -> bool {
        self.cooldowns.get(&cue_type).copied().unwrap_or(0.0) <= 0.0
    }

    pub fn trigger(&mut self, cue_type: SoundCueType) {
        self.cooldowns.insert(cue_type, cue_type.cooldown_secs());
    }

    pub fn update(&mut self, dt: f32) {
        for (_k, v) in self.cooldowns.iter_mut() {
            *v = (*v - dt).max(0.0);
        }
    }

    /// 计算空间音量（基于距离衰减）
    pub fn spatial_volume(source_pos: Vec2, listener_pos: Vec2, cue_type: SoundCueType, base_vol: f32) -> f32 {
        let dist = source_pos.distance(listener_pos);
        let max = cue_type.max_distance();
        if dist >= max {
            return 0.0;
        }
        let ratio = 1.0 - dist / max;
        base_vol * ratio * ratio
    }

    /// 计算左右声道平衡
    pub fn spatial_pan(source_pos: Vec2, listener_pos: Vec2, screen_width: f32) -> f32 {
        let dx = source_pos.x - listener_pos.x;
        (dx / (screen_width * 0.5)).clamp(-1.0, 1.0)
    }
}

// ---------- 事件 ----------

#[derive(Event)]
pub struct PlaySoundCueEvent {
    pub cue_type: SoundCueType,
    pub position: Vec2,
    pub volume: f32,
}

// ---------- 插件 ----------

pub struct SoundCuePlugin;

impl Plugin for SoundCuePlugin {
    fn build(&self, app: &mut App) {
        app
            .init_resource::<SoundCueManager>()
            .add_event::<PlaySoundCueEvent>();
    }
}
