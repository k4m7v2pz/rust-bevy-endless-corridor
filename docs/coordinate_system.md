# 坐标系统设计

本文档详细介绍 Narrative Path Craft 引擎的坐标系统设计。

## 核心设计原则

> **逻辑坐标用 int，渲染/运动用 float**

### 分层结构（标准做法）

```text
grid_position: int (逻辑)
render_position: float (表现)
```

---

## 为什么不直接用 double/int 混用？

### ❌ 直接用 double 作为主坐标的问题：

- 难以做 tile 对齐（会出现 3.999999）
- 碰撞判断复杂
- 存档不稳定（浮点误差）
- 事件系统不好写（"站在格子上"变模糊）

---

## 推荐架构

### 1. 逻辑层（Game Logic）

```text
grid_x: int
grid_y: int
```

**用于：**

- 碰撞检测
- Tile 事件触发
- NPC 寻路
- 存档

---

### 2. 渲染层（Visual / Movement）

```text
render_x: float
render_y: float
```

**用于：**

- 平滑移动动画
- 插值（lerp）
- 摄像机跟随

---

### 3. 关键桥接：移动过程

当角色从 A → B：

```text
grid: (x, y) -> (x+1, y)

render:
  x += speed * dt
```

直到：

```text
render_position ≈ target_grid_position
```

然后再更新 grid。

---

## 偏移量方案规范

### 推荐结构（经典 Tile Smooth Movement）

```text
Entity:
  grid_x: int
  grid_y: int

  offset_x: float   (−1 ~ +1 或 0~1)
  offset_y: float
```

### 或者更简单（更推荐）

直接不用 offset，改成：

```text
world_x: float
world_y: float

grid_x = floor(world_x)
grid_y = floor(world_y)
```

---

## 两种方案对比

### 方案 A：grid + offset（RPG Maker 风格）

**优点：**

- 很适合 tile event
- 易控制"站在格子上"的逻辑
- 经典 RPG 思路

**缺点：**

- offset 状态要处理
- 容易写复杂

---

### 方案 B：float world position（现代引擎风格）

**优点：**

- movement 很自然（lerp / velocity）
- AI / physics 更简单
- 不需要 offset 概念

**缺点：**

- tile 对齐要 floor/round
- event system 要做映射

---

## 恐怖 RPG 引擎推荐方案

结合需求（方格 + 恐怖 + 事件触发）：

### ✔ 最优折中：

```text
Logic:
  grid_x (int)
  grid_y (int)

Render:
  world_x (float)
  world_y (float)

Mapping:
  world = grid * tile_size + interpolation
```

### 移动流程：

1. 玩家按方向
2. 检查目标 grid 是否可走
3. 设置 target_grid
4. lerp 到 target world position
5. 到达后更新 grid

---

## 关键设计点（避坑指南）

### ❗ 不要让"碰撞 = float"

碰撞一定要：

```text
grid-based collision
```

否则恐怖 RPG 会变得不可控（事件触发会漂移）

---

### ❗ 不要用"移动期间 grid 改变"

grid 只在"完成移动后"更新

---

## 一句话总结

> ✔ 用 int 管"在哪个格子"
> ✔ 用 float 管"怎么移动过去"
