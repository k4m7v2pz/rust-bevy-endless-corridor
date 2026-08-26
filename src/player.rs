//! 玩家系统: 移动 / 躲藏 / 手电筒

use bevy::prelude::*;

use crate::{GameMap, MainCamera, PlayerTag, HidingSpotTag, WORLD_HEIGHT, WORLD_WIDTH};
use crate::constants::*;

#[derive(Component)]
pub struct Player {
    pub radius: f32,
    pub speed: f32,
    pub is_hiding: bool,
    pub flashlight_angle: f32,
    pub flashlight_radius: f32,
}

impl Default for Player {
    fn default() -> Self {
        Self {
            radius: PLAYER_RADIUS,
            speed: PLAYER_SPEED,
            is_hiding: false,
            flashlight_angle: 0.0,
            flashlight_radius: PLAYER_FLASHLIGHT_RADIUS,
        }
    }
}

pub fn spawn_player(commands: &mut Commands, map: &GameMap) {
    let pos = map.player_spawn;
    commands.spawn((
        SpriteBundle {
            sprite: Sprite {
                color: Color::rgb(0.35, 0.8, 1.0),
                custom_size: Some(Vec2::new(PLAYER_SIZE, PLAYER_SIZE)),
                ..default()
            },
            transform: Transform::from_xyz(pos.x, pos.y, 2.0),
            ..default()
        },
        Player::default(),
        PlayerTag,
    ));
}

pub fn player_movement(
    keyboard: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    map: Res<GameMap>,
    mut q: Query<(&mut Transform, &mut Player)>,
) {
    let Ok((mut trans, mut _player)) = q.get_single_mut() else { return };

    let mut dir = Vec2::ZERO;
    if keyboard.pressed(KeyCode::KeyW) || keyboard.pressed(KeyCode::ArrowUp) {
        dir.y += 1.0;
    }
    if keyboard.pressed(KeyCode::KeyS) || keyboard.pressed(KeyCode::ArrowDown) {
        dir.y -= 1.0;
    }
    if keyboard.pressed(KeyCode::KeyA) || keyboard.pressed(KeyCode::ArrowLeft) {
        dir.x -= 1.0;
    }
    if keyboard.pressed(KeyCode::KeyD) || keyboard.pressed(KeyCode::ArrowRight) {
        dir.x += 1.0;
    }

    let speed_mod = if _player.is_hiding { PLAYER_HIDE_SPEED_MOD } else { 1.0 };
    let effective_speed = _player.speed * speed_mod * time.delta_seconds();

    if dir.length_squared() > 0.0 {
        dir = dir.normalize();
        let next = trans.translation.xy() + dir * effective_speed;
        let clamped = map.collide_circle(next, _player.radius);
        trans.translation.x = clamped.x.clamp(20.0, WORLD_WIDTH - 20.0);
        trans.translation.y = clamped.y.clamp(20.0, WORLD_HEIGHT - 20.0);
    }
}

pub fn player_hide_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut player_q: Query<(&Transform, &mut Player, &mut Sprite), Without<HidingSpotTag>>,
    hiding_q: Query<&Transform, (With<HidingSpotTag>, Without<Player>)>,
) {
    if keyboard.just_pressed(KeyCode::Space) {
        if let Ok((p_trans, mut player, mut sprite)) = player_q.get_single_mut() {
            let pp = p_trans.translation.xy();
            
            // 检查是否靠近躲藏点
            let near_hiding_spot = hiding_q.iter().any(|h_trans| {
                (h_trans.translation.xy() - pp).length() < PLAYER_HIDE_DISTANCE
            });
            
            if near_hiding_spot {
                player.is_hiding = !player.is_hiding;
                if player.is_hiding {
                    sprite.color = Color::rgba(0.25, 0.5, 0.8, 0.5);
                } else {
                    sprite.color = Color::rgb(0.35, 0.8, 1.0);
                }
            }
        }
    }
}

pub fn rotate_flashlight_to_mouse(
    windows: Query<&Window>,
    camera_q: Query<(&Camera, &GlobalTransform), With<MainCamera>>,
    mut player_q: Query<(&Transform, &mut Player)>,
) {
    let (camera, cam_trans) = camera_q.single();
    let Ok(window) = windows.get_single() else { return };
    let Some(cursor_screen) = window.cursor_position() else { return };
    let Some(world) = camera.viewport_to_world_2d(cam_trans, cursor_screen) else {
        return;
    };

    if let Ok((p_trans, mut player)) = player_q.get_single_mut() {
        let dx = world.x - p_trans.translation.x;
        let dy = world.y - p_trans.translation.y;
        if dx.abs() > 0.01 || dy.abs() > 0.01 {
            player.flashlight_angle = dy.atan2(dx);
        }
    }
}
