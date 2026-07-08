//! 物品: 钥匙 (可收集) / 出口门 / 躲藏点

use bevy::prelude::*;
use bevy_state::state::NextState;

use crate::{GameMap, GameState, HidingSpotTag, ItemTag, WorldState, player::Player, PlayerTag};
use crate::constants::*;

#[derive(Component)]
pub enum ItemKind {
    Key,
    ExitDoor,
}

pub fn spawn_items_and_spots(commands: &mut Commands, map: &GameMap) {
    // 钥匙
    for &pos in &map.key_spots {
        commands.spawn((
            SpriteBundle {
                sprite: Sprite {
                    color: Color::rgb(1.0, 0.85, 0.2),
                    custom_size: Some(Vec2::new(KEY_SIZE, KEY_SIZE)),
                    ..default()
                },
                transform: Transform::from_xyz(pos.x, pos.y, 1.5),
                ..default()
            },
            ItemKind::Key,
            ItemTag,
        ));
        // 发光外圈
        commands.spawn((
            SpriteBundle {
                sprite: Sprite {
                    color: Color::rgba(1.0, 0.8, 0.2, 0.25),
                    custom_size: Some(Vec2::new(KEY_GLOW_SIZE, KEY_GLOW_SIZE)),
                    ..default()
                },
                transform: Transform::from_xyz(pos.x, pos.y, 1.4),
                ..default()
            },
            ItemTag,
        ));
    }

    // 出口门
    let pos = map.exit_position;
    commands.spawn((
        SpriteBundle {
            sprite: Sprite {
                color: Color::rgb(0.15, 1.0, 0.4),
                custom_size: Some(Vec2::new(EXIT_DOOR_SIZE, EXIT_DOOR_SIZE)),
                ..default()
            },
            transform: Transform::from_xyz(pos.x, pos.y, 1.5),
            ..default()
        },
        ItemKind::ExitDoor,
        ItemTag,
    ));
    commands.spawn((
        SpriteBundle {
            sprite: Sprite {
                color: Color::rgba(0.3, 1.0, 0.5, 0.3),
                custom_size: Some(Vec2::new(EXIT_DOOR_GLOW_SIZE, EXIT_DOOR_GLOW_SIZE)),
                ..default()
            },
            transform: Transform::from_xyz(pos.x, pos.y, 1.4),
            ..default()
        },
        ItemTag,
    ));

    // 躲藏点 (蓝色方块)
    for &pos in &map.hiding_spots {
        commands.spawn((
            SpriteBundle {
                sprite: Sprite {
                    color: Color::rgba(0.3, 0.6, 1.0, 0.35),
                    custom_size: Some(Vec2::new(HIDING_SPOT_SIZE, HIDING_SPOT_SIZE)),
                    ..default()
                },
                transform: Transform::from_xyz(pos.x, pos.y, 1.3),
                ..default()
            },
            HidingSpotTag,
        ));
    }
}

pub fn item_update(
    mut commands: Commands,
    mut state: ResMut<WorldState>,
    mut next_state: ResMut<NextState<GameState>>,
    player_q: Query<(&Transform, &Player), With<PlayerTag>>,
    items_q: Query<(Entity, &Transform, &ItemKind)>,
) {
    let Ok((p_trans, player)) = player_q.get_single() else { return };
    let pp = p_trans.translation.xy();
    let _ = player; // 使用 player 消除未使用警告

    for (e, trans, kind) in items_q.iter() {
        let ep = trans.translation.xy();
        match kind {
            ItemKind::Key => {
                if (ep - pp).length() < KEY_COLLECT_DISTANCE {
                    state.keys_collected += 1;
                    commands.entity(e).despawn();
                }
            }
            ItemKind::ExitDoor => {
                if state.keys_collected >= state.keys_total && (ep - pp).length() < EXIT_DOOR_ENTER_DISTANCE {
                    next_state.set(GameState::Win);
                }
            }
        }
    }
}
