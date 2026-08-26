# 无尽回廊（The Endless Corridor）

一个用 **Rust + Bevy** 重建的 2D 生存恐怖无尽走廊游戏。从 Python（pygame/arcade）版本迁移而来，无商用意图、纯开源分享。

## 运行

```bash
cargo run
```

- Bevy 锁定 **0.14**（更高版本在 macOS 上会黑屏），`default-features = false`。
- 中文 UI 使用内嵌的思源黑体（Noto Sans CJK SC），无需额外安装字体。
- 美术走 `.png` 文件，不使用运行时过程化 sprite/tile。

## 操作

| 按键 | 功能 |
|---|---|
| 方向键 / WASD | 移动 |
| `ESC` | 暂停菜单（继续 / 存档·读档 / 返回主菜单） |
| `L` | 开始界面 / 游戏中直接进入存档界面 |
| `回车` | 开始界面进入游戏 |

存档界面有多个入口：开始界面按 `L`、暂停菜单按 `2`/`S`、游戏中按 `L`。

## 项目结构

- `src/main.rs` — 状态机（StartScreen / SaveMenu / Playing / GameOver / Win）+ `PlayPhase` 子状态（Running / Paused）+ 字体插件
- `src/game_ui.rs` — 各界面（标题、暂停菜单、存档界面）
- `src/player.rs` / `monster.rs` / `items.rs` / `trap.rs` — 玩家、怪物、物品、陷阱
- `src/perception.rs` / `darkness.rs` / `fog_of_war.rs` / `environment.rs` — 感知、黑暗、战争迷雾、环境（雨效）
- `src/looping_corridor.rs` / `tile_map.rs` — 无尽走廊循环与地图
- `src/save.rs` — 存档系统
- `src/narrative.rs` / `dialogue.rs` / `endings.rs` — 剧情、对话、结局
- `src/debug.rs` / `notification.rs` / `sound_cue.rs` — 调试控制台、通知、音效
- `docs/` — 设计文档（含 `license_strategy.md` 授权策略说明）

## 相关仓库

| 仓库 | 说明 |
|---|---|
| `assets-endless-corridor` | 美术 / 音效 / 音频素材（git 子模块） |
| `rust-bevy-narrative-path-craft` | 引擎（从本游戏示例中抽离，纯文档骨架） |

## 许可证

- 代码：**木兰宽松许可证第 2 版（Mulan PSL v2）+ Unlicense** 双许可，任选其一，见 [`LICENSE`](./LICENSE) 与 [`UNLICENSE`](./UNLICENSE)。
- 署名见 [`CREDITS`](./CREDITS)；授权策略与署名思想见 [`docs/license_strategy.md`](./docs/license_strategy.md)。
- 素材（`assets/` 子模块）单独按 **Mulan OWL BY-PL v1** 授权，见素材仓库。
