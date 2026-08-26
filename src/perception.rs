//! 恐惧 / 理智 & 幻觉系统

use bevy::prelude::*;
use bevy::math::primitives::Circle;
use bevy::sprite::MaterialMesh2dBundle;
use rand::Rng;

use crate::player::Player;
use crate::{MainCamera, MonsterTag, PlayerTag, ScreenShake, WorldState};
use crate::constants::*;

#[derive(Component)]
pub struct HallucinationTag;

#[derive(Component)]
pub struct Hallucination {
    pub lifetime: f32,
}

#[derive(Resource)]
pub struct HallucinationAssets {
    pub mesh: Handle<Mesh>,
    pub material: Handle<ColorMaterial>,
}

pub fn setup_hallucination_assets(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let mesh = meshes.add(Mesh::from(Circle { radius: HALLUCINATION_SIZE }));
    let material = materials.add(ColorMaterial::from(Color::rgba(0.7, 0.6, 0.9, 0.35)));
    commands.insert_resource(HallucinationAssets { mesh, material });
}

pub fn update_fear_and_sanity(
    time: Res<Time>,
    mut state: ResMut<WorldState>,
    mut shake: ResMut<ScreenShake>,
    player_q: Query<(&Transform, &Player), (With<PlayerTag>, Without<MonsterTag>)>,
    monster_q: Query<&Transform, (With<MonsterTag>, Without<PlayerTag>)>,
) {
    let Ok((p_trans, player)) = player_q.get_single() else { return };
    let pp = p_trans.translation.xy();
    let dt = time.delta_seconds();

    let mut nearest = f32::INFINITY;
    for m_trans in monster_q.iter() {
        let d = (m_trans.translation.xy() - pp).length();
        if d < nearest { nearest = d; }
    }

    if nearest < FEAR_RADIUS {
        let inc = ((FEAR_RADIUS - nearest) / FEAR_RADIUS) * FEAR_INCREASE_RATE * dt;
        state.fear_level = (state.fear_level + inc).min(FEAR_MAX);
    } else {
        state.fear_level = (state.fear_level - FEAR_DECREASE_RATE * dt).max(0.0);
    }

    if state.fear_level > FEAR_SANITY_THRESHOLD && !player.is_hiding {
        state.sanity = (state.sanity - SANITY_DECREASE_RATE * dt).max(0.0);
    } else if player.is_hiding && nearest > SANITY_RECOVERY_HIDE_DIST {
        state.sanity = (state.sanity + SANITY_RECOVERY_HIDE_RATE * dt).min(SANITY_MAX);
    } else if nearest > SANITY_RECOVERY_FAR_DIST {
        state.sanity = (state.sanity + SANITY_RECOVERY_FAR_RATE * dt).min(SANITY_MAX);
    }

    shake.intensity = (state.fear_level / FEAR_MAX) * SCREEN_SHAKE_MAX_INTENSITY;
}

pub fn update_perception(
    mut commands: Commands,
    time: Res<Time>,
    state: Res<WorldState>,
    assets: Option<Res<HallucinationAssets>>,
    player_q: Query<&Transform, With<PlayerTag>>,
    mut existing: Query<(Entity, &mut Hallucination)>,
) {
    let dt = time.delta_seconds();
    for (e, mut h) in existing.iter_mut() {
        h.lifetime -= dt;
        if h.lifetime <= 0.0 {
            commands.entity(e).despawn();
        }
    }

    let Some(ha) = assets.as_deref() else { return };
    let Ok(p_trans) = player_q.get_single() else { return };
    let pp = p_trans.translation.xy();

    if state.sanity > SANITY_HALLUCINATION_THRESHOLD { return; }
    let intensity = (SANITY_HALLUCINATION_THRESHOLD - state.sanity) / SANITY_HALLUCINATION_THRESHOLD; // 0..1
    let chance = intensity.clamp(0.0, 1.0) * HALLUCINATION_MAX_CHANCE * dt;
    if rand::thread_rng().gen::<f32>() < chance {
        let mut rng = rand::thread_rng();
        let ang = rng.gen_range(0.0..std::f32::consts::TAU);
        let dist = rng.gen_range(HALLUCINATION_MIN_DIST..HALLUCINATION_MAX_DIST);
        let spawn = pp + Vec2::new(ang.cos() * dist, ang.sin() * dist);
        let lifetime = rng.gen_range(HALLUCINATION_MIN_LIFETIME..HALLUCINATION_MAX_LIFETIME);
        commands.spawn((
            MaterialMesh2dBundle {
                mesh: ha.mesh.clone().into(),
                material: ha.material.clone(),
                transform: Transform::from_xyz(spawn.x, spawn.y, 6.0)
                    .with_scale(Vec3::new(1.2 + intensity, 1.2 + intensity, 1.0)),
                ..default()
            },
            Hallucination { lifetime },
            HallucinationTag,
        ));
    }
}

pub fn draw_hallucinations(mut q: Query<(&Hallucination, &mut Transform)>) {
    for (h, mut trans) in q.iter_mut() {
        let s = (h.lifetime / HALLUCINATION_MAX_LIFETIME).clamp(0.0, 1.0) * 1.6;
        trans.scale = Vec3::new(s.max(0.1), s.max(0.1), 1.0);
    }
}

pub fn camera_follow(
    shake: Res<ScreenShake>,
    mut cam_q: Query<&mut Transform, With<MainCamera>>,
    player_q: Query<&Transform, (With<Player>, Without<MainCamera>)>,
) {
    let Ok(mut cam) = cam_q.get_single_mut() else { return };
    let Ok(p) = player_q.get_single() else { return };

    let mut target = p.translation;
    target.z = CAMERA_Z;

    if shake.intensity > 0.05 {
        let mut rng = rand::thread_rng();
        target.x += rng.gen_range(-shake.intensity..shake.intensity);
        target.y += rng.gen_range(-shake.intensity..shake.intensity);
    }

    let cur = cam.translation;
    let new = Vec3::new(
        cur.x + (target.x - cur.x) * CAMERA_FOLLOW_LERP,
        cur.y + (target.y - cur.y) * CAMERA_FOLLOW_LERP,
        target.z,
    );
    cam.translation = new;
}
