# 双窗口渲染架构 — 设计提案（待人类决策）

> 本文档是 Agent 整理的设计方案，**尚未实现**。供以后 Agent 读取并告知人类，由人类继续决策或设计。
>
> 起源：人类提出"为 UP 主之类的视频博主优化"——游戏画面渲染成正方形，UI 不渲染到游戏画面层，UP 主用 OBS 窗口捕获时能拿到纯净正方形画面，在 16:9 里自行配摄像头/状态栏。

---

## 一、目标

1. **游戏画面 = 正方形**：不受屏幕尺寸影响，手机竖屏刷视频、电脑看视频都贴切
2. **UI 与游戏画面分离**：UI 不渲染到游戏画面层，UP 主剪辑方形画面时无 UI 干扰
3. **OBS 窗口捕获友好**：游戏画面单独在一个窗口里，UI 在另一个窗口，UP 主可独立捕获、组合、裁剪

### 为什么是正方形

- 手机竖屏刷视频：正方形填满宽度，上下黑边最少
- 电脑看视频（16:9）：正方形画面 + 侧边放摄像头/状态栏/弹幕，布局自由
- 比例无关：正方形在各种比例的容器里都好摆

### 为什么 UI 单独一个窗口

- 传统做法是 UI 和游戏画面在同一窗口叠着渲染——OBS 捕获时 UI 也被捕获，UP 主要手动遮罩或裁剪
- UI 单独窗口后，UP 主捕获游戏窗口得到纯净画面，UI 窗口要不要捕获由 UP 主自己决定
- 叙事型游戏的观众不一定亲自玩，做成视频吸引人看更重要——UI 是给玩家用的，画面是给观众看的，分离两者符合这一定位

---

## 二、Bevy 0.14 双窗口技术方案

### 1. WindowPlugin 配两个窗口

```rust
// main.rs 的 main()
App::new()
.add_plugins(DefaultPlugins.set(WindowPlugin {
    primary_window: Some(Window {
        title: "无尽回廊 - 游戏画面".into(),
        resolution: WindowResolution::new(720.0, 720.0),  // 正方形
        resizable: true,
        ..default()
    }),
    exit_condition: ExitCondition::OnAllWindowsClosed,
    ..default()
}))
```

Bevy 0.14 的 `WindowPlugin` 默认只配一个主窗口。要开第二个窗口，需在 Startup 系统里用 `WindowCommands` 显式创建：

```rust
fn setup_ui_window(mut commands: Commands) {
    commands.spawn(Window {
        title: "无尽回廊 - UI".into(),
        resolution: WindowResolution::new(480.0, 720.0),  // 窄长条，放 HUD/通知/对话
        resizable: true,
        ..default()
    });
}
```

### 2. 每个窗口一个相机

```rust
fn setup_cameras(mut commands: Commands) {
    // 游戏画面相机 — 渲染到主窗口，正方形视口
    commands.spawn((
        Camera2dBundle {
            camera: Camera {
                order: 0,
                target: RenderTarget::Window(Entity::PLACEHOLDER),  // 主窗口 entity
                ..default()
            },
            transform: Transform::from_xyz(WORLD_WIDTH * 0.5, WORLD_HEIGHT * 0.5, CAMERA_Z),
            ..default()
        },
        MainCamera,
    ));

    // UI 相机 — 渲染到第二个窗口
    commands.spawn((
        Camera2dBundle {
            camera: Camera {
                order: 1,
                target: RenderTarget::Window(Entity::PLACEHOLDER),  // UI 窗口 entity
                ..default()
            },
            ..default()
        },
        UiCamera,
    ));
}
```

**难点**：0.14 的 `RenderTarget::Window` 要传窗口的 Entity，但窗口 entity 是 Startup 时动态创建的。需在 Startup 系统里先建窗口、拿到 entity、再建相机——或用 `RenderTarget::Window(WindowRef::Secondary)` 这种 0.14 提供的别名机制（需查证 0.14 API 是否支持）。

### 3. UI entity 挂 TargetCamera(UI 窗口相机)

现有所有 UI entity（`spawn_hud`、`spawn_start_screen`、`NotificationRoot`、`DebugPanel`、`ConsoleRoot`）都带 `TargetCamera(ui_camera_entity)`，确保渲染到 UI 窗口而非游戏窗口。

游戏画面层 entity（玩家、怪物、物品、地图、迷雾、darkness、幻觉）**不挂 TargetCamera**，渲染到默认主窗口。

---

## 三、影响清单

| 模块 | 改动 | 风格 |
|---|---|---|
| `main.rs` | WindowPlugin 配置、setup_ui_window、setup_cameras 双相机 | 中 |
| `game_ui.rs` | spawn_hud / spawn_start_screen / spawn_save_menu / spawn_game_over / spawn_win 的 TargetCamera 指向 UI 相机 | 小（改 entity 引用） |
| `debug.rs` | setup_debug_ui 的 TargetCamera | 小 |
| `notification.rs` | setup_notification_ui 的 TargetCamera | 小 |
| `dialogue.rs` | 对话框 UI 的 TargetCamera | 小 |
| `constants.rs` | 游戏窗口尺寸改正方形（720x720），UI 窗口尺寸新增（480x720） | 小 |
| `perception.rs` | camera_follow 要适配正方形视口（相机移动范围变了） | 小 |
| `save.rs` | 无影响（存档不涉及窗口配置） | — |

---

## 四、待人类决策的点

### D1. 游戏窗口正方形尺寸

- **720x720**：手机竖屏友好，但电脑上看可能偏小
- **960x960**：电脑上更舒适，但手机捕获后缩放损失
- **可调整**：玩家可自行拉伸，但正方形比例锁死

建议：**720x720 锁定**，UP 主后期放大无质量损失（像素艺术风）。

### D2. UI 窗口的形态

- **窄长条 480x720**：HUD/通知/对话竖排，放游戏窗口右侧
- **与游戏窗口同尺寸 720x720**：UI 自由布局，但 OBS 排列时占地大
- **可隐藏**：玩家不录视频时可关闭 UI �窗口，纯画面沉浸

建议：**480x720 竖排**，可选关闭。

### D3. 正方形视口与世界地图的适配

现有世界是 `WORLD_WIDTH x WORLD_HEIGHT = 1280*3 x 720*3 = 3840 x 2160`（宽长方形）。
正方形视口（720x720）意味着玩家只看到世界的一小块——相机跟随要调整：

- **方案 A**：相机正方形视口，跟随玩家移动，玩家始终居中——视野变小，恐怖感增强
- **方案 B**：世界改成正方形生成（`WORLD_WIDTH = WORLD_HEIGHT`），相机不跟随，固定全局——地图变小，但整体可见
- **方案 C**：世界保持长方形，相机正方形视口跟随，但渲染时做 letterbox 黑边填充——画面纯净正方形，但牺牲部分渲染区域

建议：**方案 A**——相机跟随、玩家居中、视野变小符合恐怖游戏调性（手电筒只照前方，迷雾只揭示周围 5 格，视野本就不大）。

### D4. 鼠标输入坐标转换

`rotate_flashlight_to_mouse` 现在从主窗口取鼠标坐标。双窗口后鼠标可能在 UI 窗口上——需确认鼠标坐标取的是哪个窗口的，且转换到游戏世界坐标时用游戏窗口的相机投影。

建议：**显式取游戏窗口的 CursorPos**，UI 窗口的鼠标事件不影响手电筒方向。

### D5. 是否要支持单窗口回退

有些玩家不录视频，希望 UI 叠在画面上（传统方式）。是否要一个配置开关：

- `--single-window`：UI 叠在游戏窗口（回退到现状）
- 默认：双窗口

建议：**加配置开关**，默认双窗口，但保留单窗口回退路径。

---

## 五、实现顺序建议（供以后 Agent 执行）

1. 先在 `/tmp` 写最小双窗口用例验证 0.14 API（`RenderTarget::Window` 怎么拿动态 entity）
2. 改 `constants.rs`：加 `GAME_WINDOW_SIZE`/`UI_WINDOW_SIZE` 常量
3. 改 `main.rs`：WindowPlugin 配主窗口 + Startup 建 UI 窗口 + 双相机
4. 扫所有 UI spawn 函数，统一 `TargetCamera(UI_CAMERA_ENTITY)`（用一个 Resource 存 UI 相机 entity）
5. `cargo check` + 手动运行验证 OBS 能识别两个窗口
6. 改 `perception.rs::camera_follow` 适配正方形视口
7. 改 `player.rs::rotate_flashlight_to_mouse` 适配双窗口鼠标坐标
8. （可选）加 `--single-window` CLI 开关

---

## 六、相关 AGENTS.md 条目

实现后应在 `AGENTS.md` 附录追加：

```
### E. 渲染架构
- 双窗口: 游戏画面窗口（正方形 720x720）+ UI 窗口（480x720）
- 为 UP 主优化: OBS 窗口捕获友好，游戏画面纯净无 UI
- --single-window 开关回退到传统单窗口叠 UI
- 叙事型游戏定位: 画面给观众看，UI 给玩家用，分离两者
```

---

## 七、参考

- Bevy 0.14 多窗口：需查证 `RenderTarget::Window` 在 0.14 下如何引用动态创建的窗口 entity
- OBS 窗口捕获：UP 主在 OBS 里"添加来源 → 窗口捕获"选对应窗口即可
- RPG Maker 对比：古老，UI 和画面绑死，无双窗口概念——本项目对 UP 主更友好
