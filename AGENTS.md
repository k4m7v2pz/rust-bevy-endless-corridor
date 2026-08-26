# AGENTS.md — Agent 协作规约

本文件是本仓库的 Agent 协作约定。**Agent 在与人类的协作方式发生变动时，必须自动编辑本文件以反映现状。** 人类也可随时手动修订。

---

## 一、脱敏与公开仓库政策

> **大部分仓库都是公开开源仓库，提交到 git 的内容默认会被互联网可见。**

### 1. 提交内容脱敏

凡进入 git 的内容（代码、文档、commit message、注释、配置）必须满足：

- **不得包含** 个人邮箱、真实姓名、私钥、token、密码、私人服务器地址、代理端口（如本机 socks 7890）、内部 IP。
- **不得包含** 未公开的私人仓库地址。公开开源仓库地址可保留。
- **commit message** 里不要嵌入远端 URL、不要嵌入用户私人邮箱；trailer 统一用 `Co-Authored-By: Doubao <noreply@doubao.com>`（署名如实标注实际 AI Agent 名字）。
- **文档里** 若要举例远端、邮箱、端口，用占位符（`<example@example.com>`、`<proxy-port>`、`<your-remote>`）。

### 2. 用 .gitignore 忽略不该进库的本地数据

Agent 在提交前必须核对暂存区，下列内容**不得入库**，应写入 `.gitignore`：

| 类别 | 示例 | 理由 |
|---|---|---|
| 运行时生成的存档 | `saves/*.json` | 玩家个人进度，非项目数据 |
| 本地缓存 / 构建产物 | `target/`、`.DS_Store`、`*.log` | 体积大、机器相关 |
| 私人笔记 / 草稿 | `notes/private.md`、`scratch/` | 个人用，非项目交付 |
| 第三方克隆参考仓 | `reference/<其他仓>` | 旁路参考，非本仓代码 |
| IDE 本地配置 | `.idea/`、`.vscode/`（除非团队共享） | 机器相关 |

### 3. 提交前核对流程

Agent 在执行 `git commit` 前必须：

1. `git status --short` + `git diff --cached --name-only` 列暂存区
2. 肉眼扫一遍：有无 `saves/`、私人邮箱、token、本地绝对路径泄漏
3. 若有误网，`git restore --staged <file>` 摘出，必要时加进 `.gitignore`
4. 确认无泄漏再 commit

---

## 二、Agent 自动提交与推送

### 1. 何时自动提交

当人类明确要求"提交并推送"、"你来处理提交"等时，Agent 可直接执行 `git add` → `git commit` → `git push`，无需每步停下问人。

### 2. commit message 规范

- 首行：`<type>: <概要>`，type 用 `feat` / `fix` / `docs` / `refactor` / `chore` / `test`
- 空行后正文：要点列表，说明做了什么、为什么
- 末尾 trailer（空行隔开）：
  ```
  Co-Authored-By: Doubao <noreply@doubao.com>
  ```
- 用 `git commit -m "$(cat <<'EOF' ... EOF)"` heredoc 保空行；`--amend` / `revert` 不加 trailer

### 3. 推送前确认

- 推送前 `git log -1 --format='%B'` 校验 message 完整（trailer 不应裸成首行）
- 推送目标分支默认当前分支（`git push origin <current>`），不擅自改远端或新建分支
- 推送失败不重试同一命令，先读错误（权限 / 非快进 / 拒接）再修

---

## 三、协作方式自维护

**触发条件**：Agent 与人类的协作方式发生变动时，例如：

- 人类指定了新的代理或网络配置（本机 socks 7890 等）→ 不要写进 git，但要在本文件"附录"里记协作约束
- 人类偏好变更（如"美术一律走 .png 文件不要运行时生成"）→ 在"附录"里记设计原则
- 新的自动行为约定（如"测试必须过才能提交"）→ 在本文件里记成规则
- 工具链锁定（如"Bevy 锁 0.14，更高版本 macOS 黑屏"）→ 在"附录"里记技术锚点

**执行方式**：Agent 在执行完变动后，`edit_file` 本文件追加/修订对应条目，下次会话 Agent 读到本文件即继承约定。

---

## 四、附录：本项目当前约定

> 本节是 Agent 维护的动态部分，记录与本项目具体协作约定。

### A. 技术锚点

- **Bevy 锁 0.14**：更高版本在 macOS 会黑屏，不要升到 0.15+。
- **Bevy 配置用 `default-features = false`**：需要显式开 `bevy_state` / `bevy_app` / `bevy_dev_tools` 等 feature；state API 要 `use bevy_state::...` 而非 `bevy::state::...`。
- **美术资源走 .png 文件**：用 Krita 编辑，运行时用 `AssetServer::load`。**禁止**运行时过程化生成美术素材（几何 mesh / 粒子除外，sprite/tile 一律走文件）。

### B. 网络与代理

- **gitcode.com 走直连**：与当前 Mac 同在大陆网络，不走代理。
- **github.com 走代理**：本机 socks http 复用端口（端口值不入 git，见脱敏政策）。
- Agent 克隆仓库时按远端域名判断是否走代理，不要把代理端口写进任何提交内容。

### C. 设计原则

- **AI 友好剧本**：JSON 存储 + 2 空格缩进 + JSON Schema 校验（`configs/schema/`）。`metadata.ai_context` / `creator_hints` 字段保留给外部 AI 工具消费。
- **"有库优先用库"**：优先用 Bevy 原生 / 现有 crate，实在不行才自己写。
- **迁移决策**：从 Python 版迁移时，先对照参考仓是否有现成实现可抄；抄时要适配本项目 Bevy 版本与 ECS 风格，不盲目照搬。

### D. 工作流锚点

- **改动验证**：每次改动后 `cargo check` 验证编译，`cargo test` 验证测试，不要跳过。
- **不引入重依赖**：如 `jsonschema` crate 会拉一大堆传递依赖，运行时校验改用轻量手写 + 静态 schema 供外部工具。
- **commit 前核对暂存区**：按本文件"脱敏政策"第 3 条执行。

### E. 待决策的设计提案

- **双窗口渲染架构**：为 UP 主优化，游戏画面正方形独立窗口 + UI 另一窗口。方案详见 `docs/dual-window-rendering.md`，**尚未实现**，由人类决策后 Agent 执行。
