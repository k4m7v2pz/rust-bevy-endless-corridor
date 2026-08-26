# 下雨覆盖层系统设计提案

## 🎯 设计目标

为 Narrative Path Craft 添加下雨环境效果，增强游戏氛围和沉浸感，同时保持引擎的简洁性和可配置性。

## 🏗️ 系统架构设计

### 1. 核心组件划分

```
Rain System Components:
├── RainVisualEffect     # 视觉效果渲染
├── RainSoundManager     # 音效管理（室内外差异化）
├── RainEventManager     # 事件处理与调度
└── RainConfigLoader     # 配置解析与管理
```

### 2. 与现有系统的集成点

- **环境特效系统** (`environment_overlay.py`)：作为新的 EnvironmentEffectType
- **事件总线** (`event_bus.py`)：新增降雨相关事件类型
- **配置系统**：扩展现有配置结构

## 🎨 视觉效果设计

### 基础雨滴效果
```python
class RainDrop:
    def __init__(self, x, y, speed, size):
        self.x = x
        self.y = y
        self.speed = speed
        self.size = size
        self.alpha = random.uniform(0.3, 0.8)
```

### 雨层渲染策略
- **近景雨滴**：大颗粒，高速度，高透明度
- **远景雨丝**：小颗粒，中等速度，较低透明度
- **地面溅射**：撞击地面时的小水花效果

## 🔊 音效系统设计

### 室内外差异化处理

**室外音效特征**：
- 广阔的空间感
- 均匀的雨声分布
- 较高的高频成分

**室内音效特征**：
- 低通滤波效果
- 声音更加沉闷
- 带有轻微的混响

### 音效控制参数
```json
{
  "rain_sounds": {
    "outdoor": {
      "file": "sounds/rain/outdoor_heavy.wav",
      "volume": 0.7,
      "low_pass": false,
      "spatial_blend": 0.0
    },
    "indoor": {
      "file": "sounds/rain/indoor_light.wav", 
      "volume": 0.4,
      "low_pass": true,
      "cutoff_frequency": 800,
      "spatial_blend": 0.3
    }
  }
}
```

## ⚙️ 配置驱动设计

### JSON 配置结构
```json
{
  "rain_effect": {
    "enabled": true,
    "intensity_levels": {
      "light": {
        "drop_density": 50,
        "drop_speed_min": 200,
        "drop_speed_max": 400,
        "sound_volume": 0.3
      },
      "moderate": {
        "drop_density": 120,
        "drop_speed_min": 300,
        "drop_speed_max": 600,
        "sound_volume": 0.6
      },
      "heavy": {
        "drop_density": 200,
        "drop_speed_min": 400,
        "drop_speed_max": 800,
        "sound_volume": 0.9
      }
    },
    "visual_settings": {
      "drop_colors": [[180, 220, 255, 180], [200, 230, 255, 120]],
      "splash_effects": true,
      "wind_influence": 0.3
    }
  }
}
```

## 🔄 事件驱动机制

### 新增事件类型
```python
class RainEventType(Enum):
    RAIN_STARTED = "rain_started"
    RAIN_INTENSITY_CHANGED = "rain_intensity_changed"
    RAIN_STOPPED = "rain_stopped"
    LOCATION_ENTERED_INDOOR = "location_entered_indoor"
    LOCATION_EXITED_INDOOR = "location_exited_indoor"
```

### 事件流转示例
```
1. 游戏触发降雨事件
   ↓
2. 环境系统启用雨效
   ↓
3. 音效系统切换到室外雨声
   ↓
4. 玩家进入建筑物
   ↓
5. 音效系统切换到室内雨声
   ↓
6. 视觉效果微调（减弱）
```

## 🎮 使用场景示例

### 场景1：恐怖氛围营造
```python
# 在恐怖场景中启动暴雨
rain_system.start_rain(intensity="heavy", duration=120)
rain_system.set_visual_theme("dark_and_stormy")

# 配合音效增强恐怖感
audio_system.play_thunder(after_delay=30)
```

### 场景2：解谜线索提示
```python
# 雨水冲刷出隐藏线索
if player_near_hidden_object:
    rain_system.increase_local_intensity(
        position=hidden_object_location,
        radius=50,
        duration=10
    )
```

### 场景3：环境叙事
```python
# 根据剧情进展改变天气
def on_story_progress(progress_level):
    if progress_level == "storm_approaching":
        rain_system.transition_to("moderate")
    elif progress_level == "climax":
        rain_system.transition_to("heavy")
```

## 🛠️ 技术实现要点

### 1. 性能优化策略
- **LOD系统**：根据距离调整雨滴密度
- **批量渲染**：合并相近的雨滴为线条
- **视锥剔除**：只渲染屏幕内的雨效

### 2. 内存管理
- **对象池**：预分配雨滴对象，避免频繁创建销毁
- **纹理缓存**：缓存常用的雨滴贴图

### 3. 兼容性考虑
- **分辨率适配**：自动调整雨滴密度
- **帧率补偿**：delta_time 校正确保视觉一致性

## 📋 待决策事项

### 需要确认的问题：
1. 音效文件的格式和来源（是否需要内置默认音效？）
2. 是否需要支持雨滴撞击不同材质的声音差异？
3. 雨效与现有光照系统的交互方式
4. 存档时是否需要保存降雨状态？

### 建议的下一步：
1. 先实现基础视觉效果
2. 集成简单的音效播放
3. 完善配置系统
4. 添加事件驱动机制

---
*此提案遵循引擎设计原则：配置驱动、模块解耦、叙事导向*