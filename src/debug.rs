//! 调试控制台 + Debug HUD
//! 按 ~ (Backquote) 切换控制台，按 F3 切换 Debug HUD
//! 抄自 rust-bevy-terrain-feel/src/render/debug_ui.rs，命令分支适配 endless-corridor

use bevy::prelude::*;
use bevy::input::keyboard::{KeyCode, KeyboardInput};
use bevy::window::PrimaryWindow;
use std::collections::HashMap;

use crate::player::Player;
use crate::{UiCamera, WorldState, PlayerTag, MonsterTag, ItemTag};
use crate::constants::*;

// ---------- 资源: Debug HUD 状态 ----------

#[derive(Resource)]
pub struct DebugHudState {
    pub visible: bool,
    pub debug_info: HashMap<String, String>,
}

impl Default for DebugHudState {
    fn default() -> Self {
        Self {
            visible: false,
            debug_info: HashMap::new(),
        }
    }
}

impl DebugHudState {
    pub fn toggle(&mut self) {
        self.visible = !self.visible;
    }

    pub fn set_info(&mut self, key: &str, value: &str) {
        self.debug_info.insert(key.to_string(), value.to_string());
    }
}

// ---------- 资源: 控制台状态 ----------

#[derive(Resource)]
pub struct ConsoleState {
    pub visible: bool,
    pub input: String,
    pub history: Vec<String>,
    pub command_history: Vec<String>,
    pub command_history_index: isize,
    pub suppress_next_input: bool,
}

impl Default for ConsoleState {
    fn default() -> Self {
        let mut console = Self {
            visible: false,
            input: String::new(),
            history: Vec::new(),
            command_history: Vec::new(),
            command_history_index: -1,
            suppress_next_input: false,
        };
        console.push("=== Creator Console ===");
        console.push("Type 'help' for commands. Press ~ or ESC to close.");
        console
    }
}

impl ConsoleState {
    pub fn push(&mut self, line: impl Into<String>) {
        let line = line.into();
        for l in line.lines() {
            self.history.push(l.to_string());
            if self.history.len() > 200 {
                self.history.remove(0);
            }
        }
    }

    pub fn record_command(&mut self, cmd: &str) {
        if let Some(last) = self.command_history.last() {
            if last == cmd {
                return;
            }
        }
        self.command_history.push(cmd.to_string());
        if self.command_history.len() > 50 {
            self.command_history.remove(0);
        }
    }
}

// ---------- 组件: UI 标记 ----------

#[derive(Component)]
pub struct DebugHudText;

#[derive(Component)]
pub struct ConsoleRoot;

#[derive(Component)]
pub struct ConsoleHistoryText;

#[derive(Component)]
pub struct ConsoleInputText;

// ---------- 资源: 跨系统传递待执行命令 ----------

#[derive(Resource, Default)]
pub struct PendingCommand(pub Option<String>);

// ---------- UI 初始化 ----------

pub fn setup_debug_ui(mut commands: Commands, ui_camera_query: Query<Entity, With<UiCamera>>) {
    let Ok(ui_camera) = ui_camera_query.get_single() else { return };

    // ── Debug HUD (F3) — 左上角多行文字 ──
    commands.spawn((
        TextBundle {
            text: Text::from_section(
                "",
                TextStyle {
                    font_size: 13.0,
                    color: Color::srgba(0.9, 1.0, 0.9, 0.95),
                    ..default()
                },
            ),
            style: Style {
                position_type: PositionType::Absolute,
                top: Val::Px(10.0),
                left: Val::Px(10.0),
                ..default()
            },
            background_color: BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.55)),
            visibility: Visibility::Hidden,
            ..default()
        },
        TargetCamera(ui_camera),
        DebugHudText,
    ));

    // ── Console (~) — 屏幕底部 45% ──
    commands.spawn((
        NodeBundle {
            style: Style {
                position_type: PositionType::Absolute,
                bottom: Val::Px(0.0),
                left: Val::Px(0.0),
                width: Val::Percent(100.0),
                height: Val::Percent(45.0),
                flex_direction: FlexDirection::Column,
                padding: UiRect::all(Val::Px(8.0)),
                ..default()
            },
            background_color: BackgroundColor(Color::srgba(0.05, 0.05, 0.08, 0.85)),
            visibility: Visibility::Hidden,
            ..default()
        },
        TargetCamera(ui_camera),
        ConsoleRoot,
    ))
    .with_children(|parent| {
        // 历史输出区（上方，flex_grow 擑开）
        parent.spawn((
            TextBundle::from_section(
                "",
                TextStyle {
                    font_size: 14.0,
                    color: Color::srgb(0.85, 0.85, 0.85),
                    ..default()
                },
            )
            .with_style(Style {
                flex_grow: 1.0,
                ..default()
            }),
            ConsoleHistoryText,
        ));
        // 输入行（底部）
        parent.spawn((
            TextBundle::from_section(
                "> ",
                TextStyle {
                    font_size: 14.0,
                    color: Color::srgb(1.0, 1.0, 1.0),
                    ..default()
                },
            ),
            ConsoleInputText,
        ));
    });
}

// ---------- 切换系统 ----------

pub fn debug_hud_toggle_system(
    keys: Res<ButtonInput<KeyCode>>,
    mut hud_state: ResMut<DebugHudState>,
    mut query: Query<&mut Visibility, With<DebugHudText>>,
) {
    if keys.just_pressed(KeyCode::F3) {
        hud_state.toggle();
        for mut vis in &mut query {
            *vis = if hud_state.visible {
                Visibility::Visible
            } else {
                Visibility::Hidden
            };
        }
    }
}

pub fn console_toggle_system(
    keys: Res<ButtonInput<KeyCode>>,
    mut console: ResMut<ConsoleState>,
    mut query: Query<&mut Visibility, With<ConsoleRoot>>,
    mut window: Query<&mut bevy::window::Window, With<PrimaryWindow>>,
) {
    if keys.just_pressed(KeyCode::Backquote) {
        console.visible = !console.visible;
        if console.visible {
            console.input.clear();
            console.command_history_index = -1;
        }
        // 抑制下一次输入，避免 ~ 字符本身被打进控制台
        console.suppress_next_input = console.visible;
        for mut vis in &mut query {
            *vis = if console.visible {
                Visibility::Visible
            } else {
                Visibility::Hidden
            };
        }
        if let Ok(mut win) = window.get_single_mut() {
            win.cursor.grab_mode = bevy::window::CursorGrabMode::None;
        }
    }
}

// ---------- 输入系统 ----------

pub fn console_input_system(
    mut key_events: EventReader<KeyboardInput>,
    keys: Res<ButtonInput<KeyCode>>,
    mut console: ResMut<ConsoleState>,
) {
    if !console.visible {
        key_events.clear();
        return;
    }

    let suppress = console.suppress_next_input;
    console.suppress_next_input = false;

    let shift = keys.pressed(KeyCode::ShiftLeft) || keys.pressed(KeyCode::ShiftRight);

    for ev in key_events.read() {
        if !ev.state.is_pressed() {
            continue;
        }
        match ev.key_code {
            KeyCode::Backspace => {
                console.input.pop();
            }
            KeyCode::Enter | KeyCode::NumpadEnter => {
                // 由 console_dispatch_system 处理
            }
            KeyCode::Escape => {
                console.visible = false;
                console.input.clear();
                console.command_history_index = -1;
            }
            KeyCode::ArrowUp if !console.command_history.is_empty() => {
                if console.command_history_index == -1 {
                    console.command_history_index =
                        (console.command_history.len() - 1) as isize;
                } else {
                    console.command_history_index =
                        std::cmp::max(0, console.command_history_index - 1);
                }
                if let Some(cmd) = console
                    .command_history
                    .get(console.command_history_index as usize)
                {
                    console.input = cmd.clone();
                }
            }
            KeyCode::ArrowDown if console.command_history_index != -1 => {
                console.command_history_index += 1;
                if console.command_history_index >= console.command_history.len() as isize {
                    console.command_history_index = -1;
                    console.input.clear();
                } else if let Some(cmd) = console
                    .command_history
                    .get(console.command_history_index as usize)
                {
                    console.input = cmd.clone();
                }
            }
            k => {
                if suppress {
                    continue;
                }
                if let Some(c) = key_to_char(k, shift) {
                    console.input.push(c);
                }
            }
        }
    }
}

fn key_to_char(k: KeyCode, shift: bool) -> Option<char> {
    use KeyCode::*;
    let c = match k {
        Space => ' ',
        Minus => if shift { '_' } else { '-' },
        Equals => if shift { '+' } else { '=' },
        LeftBracket => if shift { '{' } else { '[' },
        RightBracket => if shift { '}' } else { ']' },
        Semicolon => if shift { ':' } else { ';' },
        Apostrophe => if shift { '"' } else { '\'' },
        Comma => if shift { '<' } else { ',' },
        Period => if shift { '>' } else { '.' },
        Slash => if shift { '?' } else { '/' },
        Backslash => if shift { '|' } else { '\\' },
        Backquote => if shift { '~' } else { '`' },
        Digit0 => if shift { ')' } else { '0' },
        Digit1 => if shift { '!' } else { '1' },
        Digit2 => if shift { '@' } else { '2' },
        Digit3 => if shift { '#' } else { '3' },
        Digit4 => if shift { '$' } else { '4' },
        Digit5 => if shift { '%' } else { '5' },
        Digit6 => if shift { '^' } else { '6' },
        Digit7 => if shift { '&' } else { '7' },
        Digit8 => if shift { '*' } else { '8' },
        Digit9 => if shift { '(' } else { '9' },
        KeyA => if shift { 'A' } else { 'a' },
        KeyB => if shift { 'B' } else { 'b' },
        KeyC => if shift { 'C' } else { 'c' },
        KeyD => if shift { 'D' } else { 'd' },
        KeyE => if shift { 'E' } else { 'e' },
        KeyF => if shift { 'F' } else { 'f' },
        KeyG => if shift { 'G' } else { 'g' },
        KeyH => if shift { 'H' } else { 'h' },
        KeyI => if shift { 'I' } else { 'i' },
        KeyJ => if shift { 'J' } else { 'j' },
        KeyK => if shift { 'K' } else { 'k' },
        KeyL => if shift { 'L' } else { 'l' },
        KeyM => if shift { 'M' } else { 'm' },
        KeyN => if shift { 'N' } else { 'n' },
        KeyO => if shift { 'O' } else { 'o' },
        KeyP => if shift { 'P' } else { 'p' },
        KeyQ => if shift { 'Q' } else { 'q' },
        KeyR => if shift { 'R' } else { 'r' },
        KeyS => if shift { 'S' } else { 's' },
        KeyT => if shift { 'T' } else { 't' },
        KeyU => if shift { 'U' } else { 'u' },
        KeyV => if shift { 'V' } else { 'v' },
        KeyW => if shift { 'W' } else { 'w' },
        KeyX => if shift { 'X' } else { 'x' },
        KeyY => if shift { 'Y' } else { 'y' },
        KeyZ => if shift { 'Z' } else { 'z' },
        _ => return None,
    };
    Some(c)
}

// ---------- 命令派发 ----------

pub fn console_dispatch_system(
    keys: Res<ButtonInput<KeyCode>>,
    mut console: ResMut<ConsoleState>,
    mut pending: ResMut<PendingCommand>,
) {
    if console.visible && keys.just_pressed(KeyCode::Enter)
        || keys.just_pressed(KeyCode::NumpadEnter) && console.visible
    {
        let line = std::mem::take(&mut console.input);
        if !line.is_empty() {
            console.push(format!("> {}", line));
            console.record_command(&line);
            pending.0 = Some(line);
        }
    }
}

// ---------- 命令执行 ----------

pub fn console_execute_system(
    mut pending: ResMut<PendingCommand>,
    mut console: ResMut<ConsoleState>,
    mut hud: ResMut<DebugHudState>,
    mut player_query: Query<(&mut Transform, &mut Player), With<PlayerTag>>,
    mut world_state: ResMut<WorldState>,
    monster_query: Query<&Transform, With<MonsterTag>>,
    item_query: Query<(), With<ItemTag>>,
) {
    let Some(cmd_line) = pending.0.take() else { return };
    let parts: Vec<&str> = cmd_line.split_whitespace().collect();
    if parts.is_empty() {
        return;
    }

    let mut reply = |msg: String| console.push(msg);

    match parts[0] {
        "help" => {
            reply("Commands:".into());
            reply("  help                  show this".into());
            reply("  clear                clear console".into());
            reply("  quit                 hint to close window".into());
            reply("  fog on/off           toggle darkness overlay".into());
            reply("  door all             unlock all doors".into());
            reply("  speed <x>            set player speed multiplier".into());
            reply("  invisible on/off     toggle player visibility".into());
            reply("  sanity <value>       set player sanity (0-100)".into());
            reply("  fear <value>         set fear level (0-100)".into());
            reply("  truth on/off         toggle truth display in HUD".into());
            reply("  tp <x> <y>           teleport player".into());
            reply("  reset                reset world state".into());
        }
        "clear" => {
            console.history.clear();
        }
        "quit" => {
            reply("quit: use the window close button.".into());
        }
        "fog" => {
            if parts.len() != 2 {
                reply("usage: fog on/off".into());
            } else {
                match parts[1] {
                    "on" => {
                        hud.set_info("darkness", "enabled");
                        reply("Darkness overlay enabled.".into());
                    }
                    "off" => {
                        hud.set_info("darkness", "disabled");
                        reply("Darkness overlay disabled (cheat).".into());
                    }
                    other => reply(format!("fog: unknown action '{}' (on/off)", other)),
                }
            }
        }
        "door" => {
            if parts.len() != 2 || parts[1] != "all" {
                reply("usage: door all".into());
            } else {
                // 解锁出口门: 把 keys_collected 拉满
                world_state.keys_collected = world_state.keys_total;
                reply(format!(
                    "All doors unlocked (keys {}/{}).",
                    world_state.keys_collected, world_state.keys_total
                ));
            }
        }
        "speed" => {
            if parts.len() != 2 {
                reply("usage: speed <multiplier>".into());
            } else {
                match parts[1].parse::<f32>() {
                    Ok(mult) => {
                        let base = PLAYER_SPEED;
                        for (_, mut p) in player_query.iter_mut() {
                            p.speed = base * mult;
                        }
                        reply(format!("player.speed = {:.2} ({}x)", base * mult, mult));
                    }
                    Err(_) => reply(format!("speed: invalid value '{}'", parts[1])),
                }
            }
        }
        "invisible" => {
            if parts.len() != 2 {
                reply("usage: invisible on/off".into());
            } else {
                // 通过把 player 的 transform.scale 设为 0 来"隐身"
                // (endless-corridor 没有独立 color 字段，用 scale 控制可见性)
                match parts[1] {
                    "on" => {
                        for (mut t, _) in player_query.iter_mut() {
                            t.scale = Vec3::ZERO;
                        }
                        reply("Invisible mode ON.".into());
                    }
                    "off" => {
                        for (mut t, _) in player_query.iter_mut() {
                            t.scale = Vec3::ONE;
                        }
                        reply("Invisible mode OFF.".into());
                    }
                    other => reply(format!("invisible: unknown action '{}' (on/off)", other)),
                }
            }
        }
        "sanity" => {
            if parts.len() != 2 {
                reply("usage: sanity <value>".into());
            } else {
                match parts[1].parse::<f32>() {
                    Ok(v) => {
                        world_state.sanity = v.clamp(0.0, SANITY_MAX);
                        reply(format!("sanity = {:.1}", world_state.sanity));
                    }
                    Err(_) => reply(format!("sanity: invalid value '{}'", parts[1])),
                }
            }
        }
        "fear" => {
            if parts.len() != 2 {
                reply("usage: fear <value>".into());
            } else {
                match parts[1].parse::<f32>() {
                    Ok(v) => {
                        world_state.fear_level = v.clamp(0.0, FEAR_MAX);
                        reply(format!("fear = {:.1}", world_state.fear_level));
                    }
                    Err(_) => reply(format!("fear: invalid value '{}'", parts[1])),
                }
            }
        }
        "truth" => {
            if parts.len() != 2 {
                reply("usage: truth on/off".into());
            } else {
                match parts[1] {
                    "on" => {
                        hud.set_info("truth", "showing");
                        reply("Truth display ON.".into());
                    }
                    "off" => {
                        hud.set_info("truth", "hidden");
                        reply("Truth display OFF.".into());
                    }
                    other => reply(format!("truth: unknown action '{}' (on/off)", other)),
                }
            }
        }
        "tp" => {
            if parts.len() != 3 {
                reply("usage: tp <x> <y>".into());
            } else {
                match (parts[1].parse::<f32>(), parts[2].parse::<f32>()) {
                    (Ok(x), Ok(y)) => {
                        let x = x.clamp(0.0, WORLD_WIDTH);
                        let y = y.clamp(0.0, WORLD_HEIGHT);
                        for (mut t, _) in player_query.iter_mut() {
                            t.translation.x = x;
                            t.translation.y = y;
                        }
                        reply(format!("Teleported to ({}, {}).", x, y));
                    }
                    _ => reply("tp: invalid numbers".into()),
                }
            }
        }
        "reset" => {
            *world_state = WorldState::default();
            reply("World state reset to default.".into());
        }
        other => reply(format!("unknown command: {} (try 'help')", other)),
    }
}

// ---------- Debug HUD 更新 ----------

pub fn debug_hud_update_system(
    hud_state: Res<DebugHudState>,
    diagnostics: Res<bevy::diagnostic::DiagnosticsStore>,
    world_state: Res<WorldState>,
    player_query: Query<(&Transform, &Player), With<PlayerTag>>,
    monster_query: Query<&Transform, With<MonsterTag>>,
    item_query: Query<(), With<ItemTag>>,
    entities: Query<Entity>,
    mut text_query: Query<&mut Text, With<DebugHudText>>,
    mut panel_query: Query<&mut Visibility, With<DebugHudText>>,
) {
    for mut vis in &mut panel_query {
        *vis = if hud_state.visible {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }
    if !hud_state.visible {
        return;
    }
    let Ok(mut text) = text_query.get_single_mut() else { return };

    let fps = diagnostics
        .get(&bevy::diagnostic::FrameTimeDiagnosticsPlugin::FPS)
        .and_then(|d| d.smoothed())
        .map(|v| v as i32)
        .unwrap_or(0);

    let entity_count = entities.iter().count();
    let monster_count = monster_query.iter().count();
    let item_count = item_query.iter().count();

    let mut lines = vec![
        format!("endless_corridor debug  [F3 hide]  [~ console]"),
        format!(
            "FPS: {}  entities: {}  monsters: {}  items: {}",
            fps, entity_count, monster_count, item_count
        ),
        format!(
            "world: {}x{}  tile: {}px",
            WORLD_WIDTH, WORLD_HEIGHT, TILE_SIZE
        ),
        format!(
            "sanity: {:.1}/{:.0}  fear: {:.1}/{:.0}  keys: {}/{}",
            world_state.sanity,
            SANITY_MAX,
            world_state.fear_level,
            FEAR_MAX,
            world_state.keys_collected,
            world_state.keys_total
        ),
        format!(
            "deaths: {}  last: {}",
            world_state.death_count, world_state.last_death_reason
        ),
        String::new(),
    ];

    if let Ok((trans, player)) = player_query.get_single() {
        lines.push(format!(
            "player pos: ({:.1}, {:.1})  hiding: {}  speed: {:.0}",
            trans.translation.x,
            trans.translation.y,
            player.is_hiding,
            player.speed
        ));
        lines.push(format!(
            "flashlight: angle={:.2}  radius={:.0}",
            player.flashlight_angle, player.flashlight_radius
        ));
    }

    // 附加 debug_info 条目（按字母序）
    let mut entries: Vec<String> = hud_state
        .debug_info
        .iter()
        .map(|(k, v)| format!("{}: {}", k, v))
        .collect();
    entries.sort();
    if !entries.is_empty() {
        lines.push(String::new());
        lines.extend(entries);
    }

    text.sections[0].value = lines.join("\n");
}

// ---------- 控制台渲染 ----------

pub fn console_render_system(
    console: Res<ConsoleState>,
    mut panel: Query<&mut Visibility, With<ConsoleRoot>>,
    mut history: Query<&mut Text, With<ConsoleHistoryText>>,
    mut input: Query<&mut Text, With<ConsoleInputText>>,
) {
    for mut vis in &mut panel {
        *vis = if console.visible {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }
    if !console.visible {
        return;
    }
    if let Ok(mut hist) = history.get_single_mut() {
        hist.sections[0].value = console.history.join("\n");
    }
    if let Ok(mut inp) = input.get_single_mut() {
        inp.sections[0].value = format!("> {}", console.input);
    }
}

// ---------- Plugin ----------

pub struct DebugPlugin;

impl Plugin for DebugPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<DebugHudState>()
            .init_resource::<ConsoleState>()
            .init_resource::<PendingCommand>()
            .add_systems(Startup, setup_debug_ui)
            .add_systems(
                Update,
                (
                    debug_hud_toggle_system,
                    console_toggle_system,
                    console_input_system,
                    console_dispatch_system,
                    console_execute_system,
                    debug_hud_update_system,
                    console_render_system,
                )
                    .chain(),
            );
    }
}
