//! UI: 开始界面 / 存档界面 / 游戏结束 / 胜利 / HUD

use bevy::prelude::*;
use bevy_state::state::NextState;

use crate::{
    GameState, HudTag, PlayPhase, SaveMenuTag, SaveReturnOrigin, SaveReturnTo, GameTimer,
    WorldState, PendingSaveLoad,
};
use crate::save::{SaveManager, SaveInfo, SLOT_NAME, SCENE_NAME};
use crate::trap::Journal;
use crate::constants::*;

#[derive(Component)]
pub struct StartScreenTag;

#[derive(Component)]
pub struct GameOverScreenTag;

#[derive(Component)]
pub struct WinScreenTag;

#[derive(Component)]
pub struct PauseMenuTag;

// --- 开始界面 ---
pub fn spawn_start_screen(mut commands: Commands) {
    let root = commands
        .spawn((
            NodeBundle {
                style: Style {
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    ..default()
                },
                background_color: Color::rgb(0.02, 0.02, 0.04).into(),
                ..default()
            },
            StartScreenTag,
        ))
        .id();

    let title = commands
        .spawn(
            TextBundle::from_section(
                "无尽回廊",
                TextStyle {
                    font_size: 96.0,
                    color: Color::rgb(0.95, 0.2, 0.25),
                    ..default()
                },
            )
            .with_style(Style {
                margin: UiRect::bottom(Val::Px(18.0)),
                ..default()
            }),
        )
        .id();

    let sub = commands
        .spawn(
            TextBundle::from_section(
                "The Endless Corridor - 一个关于恐惧与逃生的故事",
                TextStyle {
                    font_size: 24.0,
                    color: Color::rgb(0.9, 0.9, 0.95),
                    ..default()
                },
            )
            .with_style(Style {
                margin: UiRect::bottom(Val::Px(40.0)),
                ..default()
            }),
        )
        .id();

    let hints = [
        "WASD / 方向键: 移动",
        "鼠标: 手电筒方向",
        "空格: 躲藏 (需靠近蓝色躲藏点)",
        &format!("收集 {} 把钥匙, 找到绿色出口逃生", KEYS_REQUIRED),
        "F5: 快速存档  F9: 快速读档",
    ];
    let mut hint_ids = Vec::new();
    for h in hints {
        let id = commands
            .spawn(
                TextBundle::from_section(
                    h,
                    TextStyle {
                        font_size: 20.0,
                        color: Color::rgb(0.75, 0.8, 0.9),
                        ..default()
                    },
                )
                .with_style(Style {
                    margin: UiRect::vertical(Val::Px(4.0)),
                    ..default()
                }),
            )
            .id();
        hint_ids.push(id);
    }

    let start = commands
        .spawn(
            TextBundle::from_section(
                "按 ENTER 或 点击鼠标 开始游戏",
                TextStyle {
                    font_size: 30.0,
                    color: Color::rgb(1.0, 0.95, 0.4),
                    ..default()
                },
            )
            .with_style(Style {
                margin: UiRect::top(Val::Px(50.0)),
                ..default()
            }),
        )
        .id();
    
    let load = commands
        .spawn(
            TextBundle::from_section(
                "按 L 读取存档",
                TextStyle {
                    font_size: 22.0,
                    color: Color::rgb(0.6, 0.8, 1.0),
                    ..default()
                },
            )
            .with_style(Style {
                margin: UiRect::top(Val::Px(12.0)),
                ..default()
            }),
        )
        .id();

    let mut all = vec![title, sub];
    all.extend(hint_ids);
    all.push(start);
    all.push(load);
    commands.entity(root).push_children(&all);
}

pub fn start_screen_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    mouse: Res<ButtonInput<MouseButton>>,
    mut next_state: ResMut<NextState<GameState>>,
    mut ret: ResMut<SaveReturnTo>,
) {
    if keyboard.just_pressed(KeyCode::Enter)
        || mouse.just_pressed(MouseButton::Left)
    {
        next_state.set(GameState::Playing);
    }
    if keyboard.just_pressed(KeyCode::KeyL) {
        ret.0 = SaveReturnOrigin::Title;
        next_state.set(GameState::SaveMenu);
    }
}

// --- 暂停菜单（ESC 覆盖层）---
pub fn spawn_pause_menu(mut commands: Commands) {
    let root = commands
        .spawn((
            NodeBundle {
                style: Style {
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    row_gap: Val::Px(20.0),
                    ..default()
                },
                background_color: Color::srgba(0.0, 0.0, 0.0, 0.78).into(),
                ..default()
            },
            PauseMenuTag,
        ))
        .id();

    let title = commands
        .spawn(TextBundle::from_section(
            "暂停",
            TextStyle {
                font_size: 60.0,
                color: Color::srgb(0.95, 0.95, 1.0),
                ..default()
            },
        ))
        .id();

    let hint1 = commands
        .spawn(TextBundle::from_section(
            "1 / ESC: 继续游戏",
            TextStyle {
                font_size: 26.0,
                color: Color::srgb(0.7, 0.9, 1.0),
                ..default()
            },
        ))
        .id();
    let hint2 = commands
        .spawn(TextBundle::from_section(
            "2 / S: 存档 · 读档",
            TextStyle {
                font_size: 26.0,
                color: Color::srgb(1.0, 0.9, 0.5),
                ..default()
            },
        ))
        .id();
    let hint3 = commands
        .spawn(TextBundle::from_section(
            "3 / Q: 返回主菜单",
            TextStyle {
                font_size: 26.0,
                color: Color::srgb(0.85, 0.75, 0.65),
                ..default()
            },
        ))
        .id();

    commands.entity(root).push_children(&[title, hint1, hint2, hint3]);
}

pub fn pause_menu_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut next_state: ResMut<NextState<GameState>>,
    mut next_phase: ResMut<NextState<PlayPhase>>,
    mut ret: ResMut<SaveReturnTo>,
) {
    if keyboard.just_pressed(KeyCode::Escape) || keyboard.just_pressed(KeyCode::Digit1) {
        next_phase.set(PlayPhase::Running);
    } else if keyboard.just_pressed(KeyCode::Digit2) || keyboard.just_pressed(KeyCode::KeyS) {
        ret.0 = SaveReturnOrigin::Game;
        next_state.set(GameState::SaveMenu);
    } else if keyboard.just_pressed(KeyCode::Digit3) || keyboard.just_pressed(KeyCode::KeyQ) {
        next_state.set(GameState::StartScreen);
    }
}

// --- 存档界面 ---
#[derive(Component)]
#[allow(dead_code)]
pub struct SaveRow(pub usize);

pub fn spawn_save_menu(mut commands: Commands) {
    let manager = SaveManager::new("saves".into());
    let saves = manager.list_saves();
    
    // 标题
    let root = commands
        .spawn((
            NodeBundle {
                style: Style {
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::FlexStart,
                    padding: UiRect::new(Val::Px(60.0), Val::Px(60.0), Val::Px(80.0), Val::Px(60.0)),
                    ..default()
                },
                background_color: Color::rgb(0.02, 0.02, 0.04).into(),
                ..default()
            },
            SaveMenuTag,
        ))
        .id();

    // "存档" 按钮样式标题
    let title = commands
        .spawn(
            NodeBundle {
                style: Style {
                    width: Val::Px(160.0),
                    height: Val::Px(80.0),
                    border: UiRect::all(Val::Px(2.0)),
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    margin: UiRect { bottom: Val::Px(60.0), left: Val::Px(20.0), ..default() },
                    ..default()
                },
                border_color: Color::rgb(0.6, 0.6, 0.7).into(),
                background_color: Color::rgba(0.05, 0.05, 0.08, 1.0).into(),
                ..default()
            }
        )
        .with_children(|parent| {
            parent.spawn(
                TextBundle::from_section(
                    "存档",
                    TextStyle {
                        font_size: 36.0,
                        color: Color::rgb(0.95, 0.95, 1.0),
                        ..default()
                    },
                )
            );
        })
        .id();

    // 表格容器
    let table = commands
        .spawn(
            NodeBundle {
                style: Style {
                    width: Val::Percent(80.0),
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Stretch,
                    ..default()
                },
                ..default()
            }
        )
        .id();

    // 表头
    let header = commands
        .spawn(
            NodeBundle {
                style: Style {
                    width: Val::Percent(100.0),
                    height: Val::Px(50.0),
                    flex_direction: FlexDirection::Row,
                    border: UiRect::all(Val::Px(2.0)),
                    margin: UiRect::bottom(Val::Px(2.0)),
                    ..default()
                },
                border_color: Color::rgb(0.5, 0.5, 0.6).into(),
                background_color: Color::rgba(0.08, 0.08, 0.12, 1.0).into(),
                ..default()
            }
        )
        .with_children(|parent| {
            // 第一列: 玩家/进度名
            parent.spawn(
                NodeBundle {
                    style: Style {
                        flex_grow: 1.0,
                        border: UiRect::right(Val::Px(2.0)),
                        align_items: AlignItems::Center,
                        justify_content: JustifyContent::Center,
                        ..default()
                    },
                    border_color: Color::rgb(0.5, 0.5, 0.6).into(),
                    ..default()
                }
            ).with_children(|p| {
                p.spawn(TextBundle::from_section(
                    "进度",
                    TextStyle { font_size: 22.0, color: Color::rgb(0.9, 0.9, 1.0), ..default() },
                ));
            });
            // 第二列: 地图/场景
            parent.spawn(
                NodeBundle {
                    style: Style {
                        flex_grow: 1.0,
                        border: UiRect::right(Val::Px(2.0)),
                        align_items: AlignItems::Center,
                        justify_content: JustifyContent::Center,
                        ..default()
                    },
                    border_color: Color::rgb(0.5, 0.5, 0.6).into(),
                    ..default()
                }
            ).with_children(|p| {
                p.spawn(TextBundle::from_section(
                    "场景",
                    TextStyle { font_size: 22.0, color: Color::rgb(0.9, 0.9, 1.0), ..default() },
                ));
            });
            // 第三列: 时间
            parent.spawn(
                NodeBundle {
                    style: Style {
                        flex_grow: 1.0,
                        border: UiRect::right(Val::Px(2.0)),
                        align_items: AlignItems::Center,
                        justify_content: JustifyContent::Center,
                        ..default()
                    },
                    border_color: Color::rgb(0.5, 0.5, 0.6).into(),
                    ..default()
                }
            ).with_children(|p| {
                p.spawn(TextBundle::from_section(
                    "游戏时间",
                    TextStyle { font_size: 22.0, color: Color::rgb(0.9, 0.9, 1.0), ..default() },
                ));
            });
            // 第四列: 存档ID
            parent.spawn(
                NodeBundle {
                    style: Style {
                        flex_grow: 1.0,
                        align_items: AlignItems::Center,
                        justify_content: JustifyContent::Center,
                        ..default()
                    },
                    ..default()
                }
            ).with_children(|p| {
                p.spawn(TextBundle::from_section(
                    "存档ID",
                    TextStyle { font_size: 22.0, color: Color::rgb(0.9, 0.9, 1.0), ..default() },
                ));
            });
        })
        .id();

    // 存档行
    let mut row_ids = Vec::new();
    if saves.is_empty() {
        let empty = commands
            .spawn(
                NodeBundle {
                    style: Style {
                        width: Val::Percent(100.0),
                        height: Val::Px(60.0),
                        align_items: AlignItems::Center,
                        justify_content: JustifyContent::Center,
                        ..default()
                    },
                    ..default()
                }
            )
            .with_children(|p| {
                p.spawn(TextBundle::from_section(
                    "暂无存档",
                    TextStyle { font_size: 22.0, color: Color::rgb(0.5, 0.5, 0.6), ..default() },
                ));
            })
            .id();
        row_ids.push(empty);
    } else {
        for (idx, save) in saves.iter().enumerate() {
            let row = spawn_save_row(&mut commands, idx, save);
            row_ids.push(row);
        }
    }

    // 返回提示
    let back_hint = commands
        .spawn(
            TextBundle::from_section(
                "按 ESC 返回 / 按 回车 或 点击 读取选中存档",
                TextStyle {
                    font_size: 20.0,
                    color: Color::rgb(0.6, 0.7, 0.9),
                    ..default()
                },
            )
            .with_style(Style {
                margin: UiRect::top(Val::Px(40.0)),
                ..default()
            }),
        )
        .id();

    commands.entity(table).push_children(&[header]);
    commands.entity(table).push_children(&row_ids);
    commands.entity(root).push_children(&[title, table, back_hint]);
}

fn spawn_save_row(commands: &mut Commands, index: usize, save: &SaveInfo) -> Entity {
    commands
        .spawn((
            NodeBundle {
                style: Style {
                    width: Val::Percent(100.0),
                    height: Val::Px(45.0),
                    flex_direction: FlexDirection::Row,
                    border: UiRect::all(Val::Px(2.0)),
                    margin: UiRect::bottom(Val::Px(2.0)),
                    align_items: AlignItems::Center,
                    ..default()
                },
                border_color: Color::rgb(0.4, 0.4, 0.5).into(),
                background_color: Color::rgba(0.05, 0.05, 0.08, 1.0).into(),
                ..default()
            },
            SaveRow(index),
        ))
        .with_children(|parent| {
            // 进度名
            parent.spawn(
                NodeBundle {
                    style: Style {
                        flex_grow: 1.0,
                        border: UiRect::right(Val::Px(2.0)),
                        height: Val::Percent(100.0),
                        align_items: AlignItems::Center,
                        justify_content: JustifyContent::Center,
                        ..default()
                    },
                    border_color: Color::rgb(0.4, 0.4, 0.5).into(),
                    ..default()
                }
            ).with_children(|p| {
                p.spawn(TextBundle::from_section(
                    SLOT_NAME,
                    TextStyle { font_size: 20.0, color: Color::rgb(0.85, 0.9, 1.0), ..default() },
                ));
            });
            // 场景
            parent.spawn(
                NodeBundle {
                    style: Style {
                        flex_grow: 1.0,
                        border: UiRect::right(Val::Px(2.0)),
                        height: Val::Percent(100.0),
                        align_items: AlignItems::Center,
                        justify_content: JustifyContent::Center,
                        ..default()
                    },
                    border_color: Color::rgb(0.4, 0.4, 0.5).into(),
                    ..default()
                }
            ).with_children(|p| {
                p.spawn(TextBundle::from_section(
                    SCENE_NAME,
                    TextStyle { font_size: 20.0, color: Color::rgb(0.85, 0.9, 1.0), ..default() },
                ));
            });
            // 时间
            parent.spawn(
                NodeBundle {
                    style: Style {
                        flex_grow: 1.0,
                        border: UiRect::right(Val::Px(2.0)),
                        height: Val::Percent(100.0),
                        align_items: AlignItems::Center,
                        justify_content: JustifyContent::Center,
                        ..default()
                    },
                    border_color: Color::rgb(0.4, 0.4, 0.5).into(),
                    ..default()
                }
            ).with_children(|p| {
                p.spawn(TextBundle::from_section(
                    &save.game_duration,
                    TextStyle { font_size: 20.0, color: Color::rgb(0.85, 0.9, 1.0), ..default() },
                ));
            });
            // 存档ID
            parent.spawn(
                NodeBundle {
                    style: Style {
                        flex_grow: 1.0,
                        height: Val::Percent(100.0),
                        align_items: AlignItems::Center,
                        justify_content: JustifyContent::Center,
                        ..default()
                    },
                    ..default()
                }
            ).with_children(|p| {
                p.spawn(TextBundle::from_section(
                    &save.save_id,
                    TextStyle { font_size: 18.0, color: Color::rgb(0.5, 0.8, 1.0), ..default() },
                ));
            });
        })
        .id()
}

pub fn save_menu_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut next_state: ResMut<NextState<GameState>>,
    mut pending_load: ResMut<PendingSaveLoad>,
    ret: Res<SaveReturnTo>,
) {
    if keyboard.just_pressed(KeyCode::Escape) {
        match ret.0 {
            SaveReturnOrigin::Title => next_state.set(GameState::StartScreen),
            SaveReturnOrigin::Game => next_state.set(GameState::Playing),
        }
    }
    
    // 回车加载第一个存档
    if keyboard.just_pressed(KeyCode::Enter) {
        let manager = SaveManager::new("saves".into());
        let saves = manager.list_saves();
        if let Some(first) = saves.first() {
            pending_load.save_id = Some(first.save_id.clone());
            next_state.set(GameState::Playing);
        }
    }
}

// --- 对话界面 ---
#[derive(Component)]
pub struct DialogueUiTag;

#[derive(Component)]
pub struct SpeakerNameTag;

#[derive(Component)]
pub struct DialogueTextTag;

#[derive(Component)]
pub struct DialogueOptionTag(pub usize);

pub fn spawn_dialogue_ui(mut commands: Commands) {
    let root = commands
        .spawn((
            NodeBundle {
                style: Style {
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    position_type: PositionType::Absolute,
                    left: Val::Px(0.0),
                    top: Val::Px(0.0),
                    flex_direction: FlexDirection::Column,
                    justify_content: JustifyContent::FlexEnd,
                    align_items: AlignItems::Center,
                    padding: UiRect::all(Val::Px(40.0)),
                    ..default()
                },
                z_index: ZIndex::Global(100),
                ..default()
            },
            DialogueUiTag,
        ))
        .id();

    let box_node = commands
        .spawn(
            NodeBundle {
                style: Style {
                    width: Val::Percent(80.0),
                    border: UiRect::all(Val::Px(3.0)),
                    padding: UiRect::all(Val::Px(20.0)),
                    flex_direction: FlexDirection::Column,
                    ..default()
                },
                background_color: Color::rgba(0.02, 0.02, 0.05, 0.95).into(),
                border_color: Color::rgb(0.6, 0.6, 0.8).into(),
                ..default()
            }
        )
        .with_children(|p| {
            // 说话者名称
            p.spawn((
                TextBundle::from_section(
                    "",
                    TextStyle {
                        font_size: 24.0,
                        color: Color::rgb(1.0, 0.9, 0.5),
                        ..default()
                    },
                )
                .with_style(Style {
                    margin: UiRect::bottom(Val::Px(10.0)),
                    ..default()
                }),
                SpeakerNameTag,
            ));

            // 对话文本
            p.spawn((
                TextBundle::from_section(
                    "",
                    TextStyle {
                        font_size: 22.0,
                        color: Color::rgb(0.95, 0.95, 1.0),
                        ..default()
                    },
                ),
                DialogueTextTag,
            ));
        })
        .id();

    commands.entity(root).push_children(&[box_node]);
}

pub fn despawn_dialogue_ui(mut commands: Commands, q: Query<Entity, With<DialogueUiTag>>) {
    for e in &q {
        commands.entity(e).despawn_recursive();
    }
}

// --- 游戏结束 ---
pub fn spawn_game_over_screen(mut commands: Commands) {
    let root = commands
        .spawn((
            NodeBundle {
                style: Style {
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    ..default()
                },
                background_color: Color::rgba(0.08, 0.0, 0.0, 0.92).into(),
                ..default()
            },
            GameOverScreenTag,
        ))
        .id();

    let t = commands.spawn(TextBundle::from_section(
        "游戏结束",
        TextStyle { font_size: 96.0, color: Color::rgb(1.0, 0.2, 0.2), ..default() },
    )).id();
    let s = commands.spawn(
        TextBundle::from_section(
            "按 R 或 点击鼠标 重新开始",
            TextStyle { font_size: 28.0, color: Color::rgb(0.95, 0.95, 0.95), ..default() },
        )
        .with_style(Style { margin: UiRect::top(Val::Px(30.0)), ..default() }),
    ).id();

    commands.entity(root).push_children(&[t, s]);
}

// --- 胜利 ---
pub fn spawn_win_screen(mut commands: Commands) {
    let root = commands
        .spawn((
            NodeBundle {
                style: Style {
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    ..default()
                },
                background_color: Color::rgba(0.0, 0.1, 0.02, 0.92).into(),
                ..default()
            },
            WinScreenTag,
        ))
        .id();

    let t = commands.spawn(TextBundle::from_section(
        "你逃出了无尽回廊!",
        TextStyle { font_size: 72.0, color: Color::rgb(0.25, 1.0, 0.5), ..default() },
    )).id();
    let s = commands.spawn(
        TextBundle::from_section(
            "按 R 或 点击鼠标 再挑战一次",
            TextStyle { font_size: 28.0, color: Color::rgb(0.95, 0.95, 0.95), ..default() },
        )
        .with_style(Style { margin: UiRect::top(Val::Px(30.0)), ..default() }),
    ).id();

    commands.entity(root).push_children(&[t, s]);
}

pub fn game_over_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    mouse: Res<ButtonInput<MouseButton>>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    if keyboard.just_pressed(KeyCode::KeyR) || mouse.just_pressed(MouseButton::Left) {
        next_state.set(GameState::Playing);
    }
}

// --- HUD: 顶部状态条 ---
pub fn spawn_hud(mut commands: Commands) {
    let root = commands
        .spawn((
            NodeBundle {
                style: Style {
                    width: Val::Percent(100.0),
                    height: Val::Auto,
                    flex_direction: FlexDirection::Row,
                    align_items: AlignItems::FlexStart,
                    justify_content: JustifyContent::SpaceBetween,
                    padding: UiRect::all(Val::Px(12.0)),
                    ..default()
                },
                background_color: Color::rgba(0.0, 0.0, 0.0, 0.35).into(),
                ..default()
            },
            HudTag,
        ))
        .id();

    // 左侧: 钥匙、理智、恐惧
    let stats = commands
        .spawn((
            NodeBundle {
                style: Style {
                    flex_direction: FlexDirection::Row,
                    column_gap: Val::Px(20.0),
                    ..default()
                },
                ..default()
            },
        ))
        .id();
    
    let keys = commands
        .spawn(TextBundle::from_sections([
            TextSection::new(
                "钥匙: ",
                TextStyle { font_size: 22.0, color: Color::rgb(1.0, 0.9, 0.35), ..default() },
            ),
            TextSection::new(
                "0/3",
                TextStyle { font_size: 22.0, color: Color::WHITE, ..default() },
            ),
        ]))
        .id();

    let sanity = commands
        .spawn(TextBundle::from_sections([
            TextSection::new(
                "理智: ",
                TextStyle { font_size: 22.0, color: Color::rgb(0.6, 0.85, 1.0), ..default() },
            ),
            TextSection::new(
                "100",
                TextStyle { font_size: 22.0, color: Color::WHITE, ..default() },
            ),
        ]))
        .id();

    let fear = commands
        .spawn(TextBundle::from_sections([
            TextSection::new(
                "恐惧: ",
                TextStyle { font_size: 22.0, color: Color::rgb(1.0, 0.4, 0.4), ..default() },
            ),
            TextSection::new(
                "0",
                TextStyle { font_size: 22.0, color: Color::WHITE, ..default() },
            ),
        ]))
        .id();

    let clues = commands
        .spawn(TextBundle::from_sections([
            TextSection::new(
                "线索: ",
                TextStyle { font_size: 22.0, color: Color::rgb(0.85, 0.65, 0.25), ..default() },
            ),
            TextSection::new(
                "0",
                TextStyle { font_size: 22.0, color: Color::WHITE, ..default() },
            ),
        ]))
        .id();

    commands.entity(stats).push_children(&[keys, sanity, fear, clues]);

    // 右侧: 游戏时间、快捷键提示
    let right_panel = commands
        .spawn((
            NodeBundle {
                style: Style {
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::FlexEnd,
                    row_gap: Val::Px(4.0),
                    ..default()
                },
                ..default()
            },
        ))
        .id();
    
    let time = commands
        .spawn(TextBundle::from_sections([
            TextSection::new(
                "时间: ",
                TextStyle { font_size: 22.0, color: Color::rgb(0.8, 0.8, 0.8), ..default() },
            ),
            TextSection::new(
                "0h 0m",
                TextStyle { font_size: 22.0, color: Color::WHITE, ..default() },
            ),
        ]))
        .id();
    
    let hints = commands
        .spawn(TextBundle::from_section(
            "F5存档 F9读档  E互动",
            TextStyle { font_size: 16.0, color: Color::rgb(0.5, 0.5, 0.5), ..default() },
        ))
        .id();

    commands.entity(right_panel).push_children(&[time, hints]);

    commands.entity(root).push_children(&[stats, right_panel]);
}

pub fn update_hud_text(
    state: Res<WorldState>,
    journal: Res<Journal>,
    timer: Res<GameTimer>,
    mut q: Query<&mut Text>,
) {
    // 遍历所有文本组件
    for mut text in &mut q {
        for section in &mut text.sections {
            if section.value.starts_with("钥匙: ") {
                section.value = format!("钥匙: {}/{}", state.keys_collected, state.keys_total.max(1));
            } else if section.value.starts_with("理智: ") {
                section.value = format!("理智: {:.0}", state.sanity);
            } else if section.value.starts_with("恐惧: ") {
                section.value = format!("恐惧: {:.0}", state.fear_level);
            } else if section.value.starts_with("线索: ") {
                section.value = format!("线索: {}", journal.discovered_count());
            } else if section.value.starts_with("时间: ") {
                let hours = (timer.seconds / 3600.0) as u32;
                let minutes = ((timer.seconds % 3600.0) / 60.0) as u32;
                section.value = format!("时间: {}h {}m", hours, minutes);
            }
        }
    }
}

// --- 通用: 清理带指定 tag 的实体 ---
pub fn despawn_screen<T: Component>(mut commands: Commands, q: Query<Entity, With<T>>) {
    for e in q.iter() {
        commands.entity(e).despawn_recursive();
    }
}
