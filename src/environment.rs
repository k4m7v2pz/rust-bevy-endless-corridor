//! 环境特效系统
//! 支持多种覆盖层效果：战争迷雾、水下、毒气、魔法光环、下雨等

use bevy::prelude::*;
use std::collections::HashSet;
use std::time::Duration;

// ---------- 效果类型 ----------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EnvironmentEffect {
    FogOfWar,
    Underwater,
    Poison,
    MagicAura,
    Rain,
    Wind,
}

impl EnvironmentEffect {
    pub fn color(&self) -> Color {
        match self {
            EnvironmentEffect::FogOfWar => Color::rgba(0.3, 0.3, 0.35, 0.5),
            EnvironmentEffect::Underwater => Color::rgba(0.1, 0.3, 0.6, 0.4),
            EnvironmentEffect::Poison => Color::rgba(0.2, 0.6, 0.2, 0.4),
            EnvironmentEffect::MagicAura => Color::rgba(0.5, 0.2, 0.7, 0.3),
            EnvironmentEffect::Rain => Color::rgba(0.3, 0.4, 0.6, 0.2),
            EnvironmentEffect::Wind => Color::rgba(0.5, 0.55, 0.6, 0.1),
        }
    }
}

// ---------- 资源 ----------

#[derive(Resource, Default)]
pub struct EnvironmentEffects {
    active: HashSet<EnvironmentEffect>,
    pub rain_intensity: f32,
    pub wind_direction: f32,
    pub wind_speed: f32,
}

impl EnvironmentEffects {
    pub fn enable(&mut self, effect: EnvironmentEffect) {
        self.active.insert(effect);
    }

    pub fn disable(&mut self, effect: EnvironmentEffect) {
        self.active.remove(&effect);
    }

    pub fn is_active(&self, effect: EnvironmentEffect) -> bool {
        self.active.contains(&effect)
    }

    pub fn active_effects(&self) -> &HashSet<EnvironmentEffect> {
        &self.active
    }

    /// 计算所有活跃效果的混合颜色
    pub fn blended_color(&self) -> Color {
        let mut r = 0.0;
        let mut g = 0.0;
        let mut b = 0.0;
        let mut a = 0.0;
        let count = self.active.len() as f32;

        if count == 0.0 {
            return Color::rgba(0.0, 0.0, 0.0, 0.0);
        }

        for effect in &self.active {
            let c = effect.color();
            r += c.r();
            g += c.g();
            b += c.b();
            a += c.a();
        }

        Color::rgba(r / count, g / count, b / count, a / count)
    }
}

// ---------- 组件 ----------

/// 环境覆盖层标记
#[derive(Component)]
pub struct EnvOverlayTag;

/// 雨滴粒子
#[derive(Component)]
pub struct Raindrop {
    pub speed: f32,
}

// ---------- 事件 ----------

#[derive(Event)]
pub struct EnvironmentEffectToggle {
    pub effect: EnvironmentEffect,
    pub enable: bool,
}

// ---------- 系统 ----------

pub struct EnvironmentPlugin;

impl Plugin for EnvironmentPlugin {
    fn build(&self, app: &mut App) {
        app
            .init_resource::<EnvironmentEffects>()
            .add_event::<EnvironmentEffectToggle>();
    }
}

/// 生成雨滴（供外部调用）
pub fn spawn_raindrops(commands: &mut Commands, count: u32, window_w: f32, window_h: f32) {
    let mut rng = rand::thread_rng();
    for _ in 0..count {
        let x = rand::Rng::gen_range(&mut rng, 0.0..window_w);
        let y = rand::Rng::gen_range(&mut rng, 0.0..window_h);
        let speed = rand::Rng::gen_range(&mut rng, 300.0..500.0);

        commands.spawn((
            SpriteBundle {
                sprite: Sprite {
                    color: Color::rgba(0.6, 0.7, 0.9, 0.6),
                    custom_size: Some(Vec2::new(2.0, 12.0)),
                    ..default()
                },
                transform: Transform::from_xyz(x, y, 50.0),
                ..default()
            },
            Raindrop { speed },
        ));
    }
}

/// 更新雨滴位置
pub fn update_rain(
    mut commands: Commands,
    mut raindrops: Query<(Entity, &mut Transform, &Raindrop)>,
    effects: Res<EnvironmentEffects>,
    time: Res<Time>,
    window_q: Query<&Window>,
) {
    if !effects.is_active(EnvironmentEffect::Rain) {
        return;
    }

    let Ok(window) = window_q.get_single() else { return };
    let w_h = window.height();

    for (entity, mut trans, drop) in &mut raindrops {
        trans.translation.y -= drop.speed * time.delta_seconds();

        if trans.translation.y < -20.0 {
            let mut rng = rand::thread_rng();
            trans.translation.y = w_h + 20.0;
            trans.translation.x = rand::Rng::gen_range(&mut rng, 0.0..window.width());
        }
    }
}
