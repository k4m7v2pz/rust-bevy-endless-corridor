# 素材处理提示词（Assets Processing Prompts）

> 来源：Python 版 `src/engine/graphics/perception_filters.py` 的**处理逻辑**（perception filter pipeline）。
> 用途：把「白底转透明 + 视觉干扰遮罩」两类图像处理操作转换为可直接交给 AI 图像工具 / 编辑流程的提示词。
> 约定：不保留 Python 代码，也不写成 Rust 代码；美术资源最终以 .png 文件入库（见 AGENTS.md「美术走 .png」约定）。

---

## 操作一：白底转透明（Background Normalization）

**提示词（默认参数）：**

> 将这张图片的纯白/近白背景转换为透明背景：RGB 三个通道均满足「通道值 ≥ 240 − 15」的像素判定为白色，将其 alpha 设为 0，其余像素保持不变；随后对边缘做 1 次半径为 0.5 的高斯柔化以消除锯齿。输出 RGBA PNG。

**可调参数：**

| 参数 | 默认 | 说明 |
|---|---|---|
| 阈值 white_threshold | 240 | 白色判定阈值（0–255），越高越严格 |
| 容差 tolerance | 15 | 允许偏离白色的范围 |
| 边缘柔化 edge_smoothing | 1 | 0–3，柔化次数 |

---

## 操作二：视觉干扰遮罩（Interference Mask）

**提示词（默认：右下角半透明覆盖）：**

> 在图片右下角添加一个视觉干扰遮罩：遮罩尺寸为「图片宽 × 15%、高 × 15%」；使用覆盖模式在遮罩区域叠加一层透明层。位置与模式可按下表替换。

**可调参数：**

| 参数 | 默认 | 说明 |
|---|---|---|
| 遮罩宽占比 mask_width_percent | 0.15 | 遮罩宽度 / 图片宽度 |
| 遮罩高占比 mask_height_percent | 0.15 | 遮罩高度 / 图片高度 |
| 位置 mask_position | bottom-right | top-left / top-right / bottom-left / bottom-right / center |
| 模式 mask_mode | overlay_alpha | crop（裁掉该区域）/ overlay_alpha（透明覆盖）/ overlay_black（黑色覆盖） |
| 覆盖透明度 overlay_alpha | 255 | 0–255，仅覆盖模式生效 |

---

## 操作三：组合管线（Pipeline）

**提示词：**

> 依次执行：① 白底转透明（阈值 240、容差 15、边缘柔化 1）；② 在右下角加 15%×15% 干扰遮罩（overlay_black 模式）。输出处理后的 RGBA PNG。

---

## 附：适用场景

- **白底转透明**：处理带白底的精灵图 / 贴图，去除背景后用于游戏画面叠加。
- **干扰遮罩**：对素材角部做视觉干扰处理（恐怖/故障风），或裁掉/盖住角部不需要的内容。
