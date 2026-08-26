# 标题画面系统

本文档详细介绍 Narrative Path Craft 引擎的标题画面系统。

## 设计理念

> **Title Screen Generator（标题界面生成器 / 模板系统）**

面向对象：
- **游戏开发者（Game Developers）**
- **内容创作者（Content Creators）**

目标：

> **几分钟搭一个可用的游戏入口，而不是从零写 UI**

核心思想：**Progressive Disclosure（渐进式暴露复杂度）**

> 给不同水平开发者不同深度的控制能力

---

## 系统架构

标题系统分为四个层次：

### 1. Data Layer（数据层）

存储标题界面的基础数据：
- 标题文本（title text）
- 按钮配置（buttons）
- 资源引用（assets）

### 2. Layout Layer（布局层）

处理 UI 的空间组织：
- **Anchor（锚点）**：定位参考点
- **Flex（弹性布局）**：灵活的排列方式
- **Resolution Independent（分辨率无关）**：自适应不同屏幕

### 3. Scene Layer（场景层）

提供视觉背景支持：
- 游戏地图作为背景
- 动画效果
- Shader 渲染

### 4. Override Layer（覆盖层）

提供高级定制能力：
- **Script（脚本）**：脚本覆盖
- **Plugin（插件）**：插件扩展
- **Mod Support（模组支持）**：允许模组修改

---

## 三种运行模式

系统支持三种不同复杂度的标题界面模式，开发者可根据需求选择：

### Mode 1：Template Mode（模板模式）

面向：**新手开发者**

提供开箱即用的默认模板，包含：
- 标题图片
- 背景音乐
- 预设按钮（New Game / Continue / Exit）
- UI 布局模板

配置示例：

```yaml
title:
  template: default
  title_text: "My Game"
  buttons:
    - new_game
    - continue
    - exit
```

目标：

> "不写代码也能跑"

---

### Mode 2：Scene Mode（场景模式）

面向：**中级开发者**

特点：**直接复用游戏场景作为背景**

配置示例：

```yaml
title_scene:
  background_scene: forest_map
  ui_overlay: title_ui
```

设计灵感来源于 **OpenRA**：

> OpenRA 的本质是 "menu = scene, UI = overlay, gameplay engine = reused for UI"

这种设计称为：**In-Game Menu System（游戏内场景菜单系统）**

目标：

> "进阶用户可以复用游戏资产"

---

### Mode 3：Code Mode（代码模式）

面向：**高级开发者**

提供完全的代码控制能力：

```python
def render_title():
    draw_custom_ui()
```

目标：

> "完全自由，无任何限制"

---

## 核心优势

相比传统游戏引擎（如 RPG Maker），本系统有三个关键升级点：

### 1. 不锁死 UI

| 传统引擎 | 本系统 |
|---------|--------|
| 固定菜单 | 可组合 UI System |

### 2. 支持 Scene-as-Background

- 采用 OpenRA 思路
- 可动态变化的标题画面
- 游戏场景与 UI 无缝融合

### 3. 分层可扩展

渐进式学习曲线：

```
Template Mode → Scene Mode → Code Mode
   (新手)         (中级)        (高级)
```

---

## 默认层与自定义层

系统本质上分为两层设计：

### Default Layer（默认层 / 开箱即用）

提供：
- New Game / Continue / Exit 按钮
- 标题图片
- 背景音乐
- UI 布局模板

目标：

> "不写代码也能跑"

### Custom Layer（自定义层 / 可扩展）

允许：
- 修改 UI Layout
- 修改动画效果
- 修改标题逻辑
- 直接用代码覆盖任何部分

目标：

> "进阶用户可以完全接管"

---

## 技术实现要点

### 分辨率无关布局

采用锚点+弹性布局，确保标题界面在不同分辨率下保持一致的美观度。

### 资源热插拔

支持运行时切换：
- 背景音乐
- 标题图片
- 按钮样式

### 事件驱动

标题界面的所有交互通过统一的事件总线处理，便于：
- 自定义按钮行为
- 添加动画效果
- 与游戏逻辑联动

---

## 快速开始

### 最简配置（使用默认模板）

```yaml
title:
  template: default
  title_text: "My Awesome Game"
```

### 自定义按钮

```yaml
title:
  template: default
  title_text: "My Game"
  buttons:
    - id: start
      text: "开始游戏"
    - id: load
      text: "继续游戏"
    - id: settings
      text: "设置"
    - id: exit
      text: "退出"
```

### 使用游戏场景作为背景

```yaml
title:
  template: scene
  background_scene: forest_map
  title_text: "Forest Adventure"
  ui_overlay:
    position: center
    buttons:
      - new_game
      - continue
      - options
      - exit
```

---

## 总结

> **Title Screen Generator 不只是一个"标题界面"，而是一个支持模板、场景和代码三层控制的"可扩展游戏入口系统"，目标是让开发者既能快速上手，也能完全自定义。**

通过 **Progressive Disclosure** 设计理念，Narrative Path Craft 的标题系统为不同水平的开发者提供了最适合的解决方案：
- 新手：使用模板，快速启动
- 中级：复用场景，灵活定制
- 高级：代码接管，完全自由
