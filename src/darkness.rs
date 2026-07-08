//! 黑暗覆盖 + 手电筒锥形光晕 (通过自定义 Mesh + ColorMaterial 实现)

use bevy::prelude::*;
use bevy::render::mesh::{Indices, PrimitiveTopology};
use bevy::sprite::MaterialMesh2dBundle;

use crate::player::Player;
use crate::monster::{Monster, MonsterState};
use crate::PlayerTag;
use crate::constants::*;

#[derive(Component)]
pub struct DarknessOverlayTag;

#[derive(Component)]
pub struct AmbientLight;

#[derive(Component)]
pub struct FlashlightCone;

#[derive(Component)]
pub struct MonsterLight(Entity);

#[derive(Resource)]
pub struct DarknessMeshHandles {
    pub cone: Handle<Mesh>,
    pub circle: Handle<Mesh>,
    pub cone_mat: Handle<ColorMaterial>,
    pub circle_mat: Handle<ColorMaterial>,
    pub ambient_mat: Handle<ColorMaterial>,
}

pub fn setup_darkness_meshes(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let cone = meshes.add(build_fan_mesh(PLAYER_FLASHLIGHT_RADIUS, PLAYER_FLASHLIGHT_HALF_ANGLE, DARKNESS_CONE_SEGMENTS));
    let circle = meshes.add(Mesh::from(shape::Circle { radius: DARKNESS_AMBIENT_RADIUS, vertices: 32 }));

    let cone_mat = materials.add(ColorMaterial::from(Color::rgba(1.0, 0.95, 0.7, 0.28)));
    let circle_mat = materials.add(ColorMaterial::from(Color::rgba(1.0, 0.25, 0.15, 0.35)));
    let ambient_mat = materials.add(ColorMaterial::from(Color::rgba(0.6, 0.65, 0.8, 0.05)));

    // 创建持久的环境光实体
    commands.spawn((
        MaterialMesh2dBundle {
            mesh: circle.clone().into(),
            material: ambient_mat.clone(),
            transform: Transform::from_xyz(0.0, 0.0, DARKNESS_Z_AMBIENT),
            ..default()
        },
        DarknessOverlayTag,
        AmbientLight,
    ));

    // 创建持久的手电筒实体
    commands.spawn((
        MaterialMesh2dBundle {
            mesh: cone.clone().into(),
            material: cone_mat.clone(),
            transform: Transform::from_xyz(0.0, 0.0, DARKNESS_Z_FLASHLIGHT),
            ..default()
        },
        DarknessOverlayTag,
        FlashlightCone,
    ));

    commands.insert_resource(DarknessMeshHandles {
        cone,
        circle,
        cone_mat,
        circle_mat,
        ambient_mat,
    });
}

fn build_fan_mesh(radius: f32, half_angle: f32, segments: u32) -> Mesh {
    let mut positions: Vec<[f32; 3]> = Vec::with_capacity((segments + 2) as usize);
    positions.push([0.0, 0.0, 0.0]);
    for i in 0..=segments {
        let t = i as f32 / segments as f32;
        let ang = -half_angle + 2.0 * half_angle * t;
        positions.push([radius * ang.cos(), radius * ang.sin(), 0.0]);
    }
    let mut indices: Vec<u32> = Vec::with_capacity((segments * 3) as usize);
    for i in 1..=segments {
        indices.push(0);
        indices.push(i);
        indices.push(i + 1);
    }
    let uvs: Vec<[f32; 2]> = positions.iter().map(|_| [0.5, 0.5]).collect();
    let normals: Vec<[f32; 3]> = positions.iter().map(|_| [0.0, 0.0, 1.0]).collect();

    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList, default());
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.insert_indices(Indices::U32(indices));
    mesh
}

pub fn darkness_overlay(
    mut commands: Commands,
    handles: Option<Res<DarknessMeshHandles>>,
    player_q: Query<(&Transform, &Player), With<PlayerTag>>,
    monster_q: Query<(Entity, &Transform, &Monster)>,
    mut ambient_q: Query<&mut Transform, (With<AmbientLight>, Without<FlashlightCone>)>,
    mut flashlight_q: Query<&mut Transform, (With<FlashlightCone>, Without<AmbientLight>)>,
    mut monster_light_q: Query<(Entity, &mut Transform, &MonsterLight)>,
) {
    let Some(h) = handles.as_deref() else { return };
    let Ok((p_trans, player)) = player_q.get_single() else { return };
    let p = p_trans.translation.xy();

    // 更新环境光位置
    if let Ok(mut trans) = ambient_q.get_single_mut() {
        trans.translation.x = p.x;
        trans.translation.y = p.y;
    }

    // 更新手电筒位置和旋转
    if let Ok(mut trans) = flashlight_q.get_single_mut() {
        trans.translation.x = p.x;
        trans.translation.y = p.y;
        trans.rotation = Quat::from_rotation_z(player.flashlight_angle);
        
        // 根据躲藏状态调整可见性（通过 scale）
        if player.is_hiding {
            trans.scale = Vec3::ZERO;
        } else {
            trans.scale = Vec3::ONE;
        }
    }

    // 管理怪物光源
    let chasing_monsters: Vec<(Entity, Vec2)> = monster_q.iter()
        .filter(|(_, _, m)| m.state == MonsterState::Chasing)
        .map(|(e, t, _)| (e, t.translation.xy()))
        .collect();

    // 更新现有怪物光源
    for (light_entity, mut trans, monster_light) in monster_light_q.iter_mut() {
        // 检查对应的怪物是否还在追逐
        let still_chasing = chasing_monsters.iter()
            .find(|(m_entity, _)| *m_entity == monster_light.0);
        
        if let Some((_, pos)) = still_chasing {
            trans.translation.x = pos.x;
            trans.translation.y = pos.y;
        } else {
            // 怪物不再追逐，删除光源
            commands.entity(light_entity).despawn();
        }
    }

    // 为新的追逐怪物创建光源
    for (monster_entity, pos) in chasing_monsters {
        // 检查是否已经有光源
        let has_light = monster_light_q.iter()
            .any(|(_, _, ml)| ml.0 == monster_entity);
        
        if !has_light {
            commands.spawn((
                MaterialMesh2dBundle {
                    mesh: h.circle.clone().into(),
                    material: h.circle_mat.clone(),
                    transform: Transform::from_xyz(pos.x, pos.y, DARKNESS_Z_MONSTER),
                    ..default()
                },
                DarknessOverlayTag,
                MonsterLight(monster_entity),
            ));
        }
    }
}
