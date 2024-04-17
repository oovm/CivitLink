# *.anim 文件格式规范

## 概述

*.anim 文件是 GG 游戏引擎用于存储动画信息的文件格式，包含动画的关键帧、曲线、时间线等信息。动画文件是动画系统的核心组成部分，用于定义游戏对象的动画效果。

## 文件结构

一个完整的 *.anim 文件是一个 TOML 格式的文本文件，用于描述动画的属性和关键帧。

### 基本结构

```ron
// 动画文件
AnimationFile({
    version: "1.0",
    animation: Animation({
        name: "IdleAnimation",
        description: "Idle animation for character",
        duration: 2.0,
        loop: true,
        speed: 1.0,
        blend_time: 0.1,
    }),
    tracks: [
        Track({
            name: "Transform",
            target: "transform",
            keyframes: [
                Keyframe({
                    time: 0.0,
                    properties: Properties({
                        position: [0, 0, 0],
                        rotation: [0, 0, 0],
                        scale: [1, 1, 1],
                    }),
                    easing: "linear",
                }),
                Keyframe({
                    time: 1.0,
                    properties: Properties({
                        position: [0, 0.1, 0],
                        rotation: [0, 0, 0],
                        scale: [1, 1, 1],
                    }),
                    easing: "ease_in_out",
                }),
                Keyframe({
                    time: 2.0,
                    properties: Properties({
                        position: [0, 0, 0],
                        rotation: [0, 0, 0],
                        scale: [1, 1, 1],
                    }),
                    easing: "ease_in_out",
                }),
            ],
        }),
        Track({
            name: "SpriteRenderer",
            target: "sprite_renderer",
            keyframes: [
                Keyframe({
                    time: 0.0,
                    properties: Properties({
                        sprite: "assets/textures/idle_01.png",
                    }),
                    easing: "step",
                }),
                Keyframe({
                    time: 0.5,
                    properties: Properties({
                        sprite: "assets/textures/idle_02.png",
                    }),
                    easing: "step",
                }),
                Keyframe({
                    time: 1.0,
                    properties: Properties({
                        sprite: "assets/textures/idle_03.png",
                    }),
                    easing: "step",
                }),
                Keyframe({
                    time: 1.5,
                    properties: Properties({
                        sprite: "assets/textures/idle_02.png",
                    }),
                    easing: "step",
                }),
                Keyframe({
                    time: 2.0,
                    properties: Properties({
                        sprite: "assets/textures/idle_01.png",
                    }),
                    easing: "step",
                }),
            ],
        }),
    ],
    events: [
        Event({
            time: 0.5,
            name: "footstep",
            params: Params({
                sound: "assets/audio/footstep.wav",
            }),
        }),
        Event({
            time: 1.5,
            name: "footstep",
            params: Params({
                sound: "assets/audio/footstep.wav",
            }),
        }),
    ],
    dependencies: [
        Dependency({
            path: "assets/textures/idle_01.png",
            guid: "018dc3f0-82c9-7d1a-8c3a-9d8b7e6f0001",
        }),
        Dependency({
            path: "assets/textures/idle_02.png",
            guid: "018dc3f0-82c9-7d1a-8c3a-9d8b7e6f0002",
        }),
        Dependency({
            path: "assets/textures/idle_03.png",
            guid: "018dc3f0-82c9-7d1a-8c3a-9d8b7e6f0003",
        }),
        Dependency({
            path: "assets/audio/footstep.wav",
            guid: "018dc3f0-82c9-7d1a-8c3a-9d8b7e6f0004",
        }),
    ],
    references: [
        Reference({
            path: "assets/prefabs/character.prefab",
            field: "animations",
        }),
    ],
    timestamp: "2026-04-10T12:00:00Z",
    hash: "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
})
```

## 字段详细说明

### 1. 版本信息

- **`version`**：字符串，动画格式版本号，用于向后兼容。

### 2. 动画信息

- **`animation`**：对象，包含动画的基本信息。
  - **`name`**：字符串，动画名称。
  - **`description`**：字符串，动画描述。
  - **`duration`**：数字，动画持续时间（秒）。
  - **`loop`**：布尔值，是否循环播放。
  - **`speed`**：数字，动画播放速度。
  - **`blend_time`**：数字，与其他动画混合的时间（秒）。

### 3. 动画轨道

- **`tracks`**：数组，包含动画的轨道。
  - 每个轨道包含：
    - **`name`**：字符串，轨道名称。
    - **`target`**：字符串，轨道目标组件。
    - **`keyframes`**：数组，轨道的关键帧。
      - 每个关键帧包含：
        - **`time`**：数字，关键帧时间（秒）。
        - **`properties`**：对象，关键帧属性，根据目标组件不同而不同。
        - **`easing`**：字符串，缓动函数，如 `linear`、`ease_in`、`ease_out`、`ease_in_out`、`step` 等。

### 4. 动画事件

- **`events`**：数组，包含动画的事件。
  - 每个事件包含：
    - **`time`**：数字，事件触发时间（秒）。
    - **`name`**：字符串，事件名称。
    - **`params`**：对象，事件参数。

### 5. 依赖关系

- **`dependencies`**：数组，包含该动画依赖的其他资源。
  - 每个依赖项包含：
    - **`path`**：字符串，依赖资源的相对路径。
    - **`guid`**：字符串，依赖资源的全局唯一标识符。

### 6. 引用关系

- **`references`**：数组，包含引用该动画的其他资源。
  - 每个引用项包含：
    - **`path`**：字符串，引用资源的相对路径。
    - **`field`**：字符串，引用该动画的字段名称。

### 7. 元数据

- **`timestamp`**：字符串，ISO 格式的时间戳，表示该动画文件的最后修改时间。
- **`hash`**：字符串，动画文件的 SHA1 哈希值，用于检测动画文件是否发生变化。

## 动画类型

### 1. 变换动画

变换动画用于控制游戏对象的位置、旋转、缩放等变换属性。

### 2. 精灵动画

精灵动画用于控制精灵渲染器的精灵帧，实现帧动画效果。

### 3. 材质动画

材质动画用于控制材质的属性，如颜色、透明度等。

### 4. 自定义动画

自定义动画，用于控制自定义组件的属性。

## 使用场景

### 1. 角色动画

- ** idle 动画**：角色静止时的动画。
- **行走动画**：角色行走时的动画。
- **跑步动画**：角色跑步时的动画。
- **跳跃动画**：角色跳跃时的动画。
- **攻击动画**：角色攻击时的动画。
- **受击动画**：角色受击时的动画。

### 2. 环境动画

- **门开关动画**：门的开关动画。
- **灯光闪烁动画**：灯光的闪烁动画。
- **水流动画**：水流的动画。

### 3. UI 动画

- **按钮动画**：按钮的点击、悬停动画。
- **面板动画**：面板的显示、隐藏动画。
- **进度条动画**：进度条的填充动画。

### 4. 编辑器集成

- **动画编辑器**：在编辑器中可视化编辑动画的关键帧。
- **动画预览**：在编辑器中实时预览动画效果。
- **动画模板**：基于现有动画创建动画模板，快速生成新动画。

## 最佳实践

1. **动画组织**：将动画按照角色或物体类型组织到不同的文件夹中。
2. **关键帧管理**：合理设置关键帧，避免过多的关键帧导致性能问题。
3. **动画复用**：尽量复用动画，减少动画数量。
4. **依赖管理**：定期检查并清理无效的依赖关系。
5. **版本控制**：将 .anim 文件纳入版本控制系统，确保团队协作时的一致性。
6. **性能优化**：根据目标平台，合理设置动画的复杂度。
7. **哈希计算**：使用 SHA1 算法计算动画文件的哈希值。

## 示例完整文件

### 角色行走动画示例

```ron
// 角色行走动画示例
AnimationFile({
    version: "1.0",
    animation: Animation({
        name: "WalkAnimation",
        description: "Walking animation for character",
        duration: 1.0,
        loop: true,
        speed: 1.0,
        blend_time: 0.1,
    }),
    tracks: [
        Track({
            name: "Transform",
            target: "transform",
            keyframes: [
                Keyframe({
                    time: 0.0,
                    properties: Properties({
                        position: [0, 0, 0],
                        rotation: [0, 0, 0],
                        scale: [1, 1, 1],
                    }),
                    easing: "linear",
                }),
                Keyframe({
                    time: 0.25,
                    properties: Properties({
                        position: [0.5, 0, 0],
                        rotation: [0, 0, 0],
                        scale: [1, 1, 1],
                    }),
                    easing: "linear",
                }),
                Keyframe({
                    time: 0.5,
                    properties: Properties({
                        position: [1.0, 0, 0],
                        rotation: [0, 0, 0],
                        scale: [1, 1, 1],
                    }),
                    easing: "linear",
                }),
                Keyframe({
                    time: 0.75,
                    properties: Properties({
                        position: [1.5, 0, 0],
                        rotation: [0, 0, 0],
                        scale: [1, 1, 1],
                    }),
                    easing: "linear",
                }),
                Keyframe({
                    time: 1.0,
                    properties: Properties({
                        position: [2.0, 0, 0],
                        rotation: [0, 0, 0],
                        scale: [1, 1, 1],
                    }),
                    easing: "linear",
                }),
            ],
        }),
        Track({
            name: "SpriteRenderer",
            target: "sprite_renderer",
            keyframes: [
                Keyframe({
                    time: 0.0,
                    properties: Properties({
                        sprite: "assets/textures/walk_01.png",
                    }),
                    easing: "step",
                }),
                Keyframe({
                    time: 0.25,
                    properties: Properties({
                        sprite: "assets/textures/walk_02.png",
                    }),
                    easing: "step",
                }),
                Keyframe({
                    time: 0.5,
                    properties: Properties({
                        sprite: "assets/textures/walk_03.png",
                    }),
                    easing: "step",
                }),
                Keyframe({
                    time: 0.75,
                    properties: Properties({
                        sprite: "assets/textures/walk_04.png",
                    }),
                    easing: "step",
                }),
                Keyframe({
                    time: 1.0,
                    properties: Properties({
                        sprite: "assets/textures/walk_01.png",
                    }),
                    easing: "step",
                }),
            ],
        }),
    ],
    events: [
        Event({
            time: 0.25,
            name: "footstep",
            params: Params({
                sound: "assets/audio/footstep.wav",
            }),
        }),
        Event({
            time: 0.75,
            name: "footstep",
            params: Params({
                sound: "assets/audio/footstep.wav",
            }),
        }),
    ],
    dependencies: [
        Dependency({
            path: "assets/textures/walk_01.png",
            guid: "018dc3f0-82c9-7d1a-8c3a-9d8b7e6f0006",
        }),
        Dependency({
            path: "assets/textures/walk_02.png",
            guid: "018dc3f0-82c9-7d1a-8c3a-9d8b7e6f0007",
        }),
        Dependency({
            path: "assets/textures/walk_03.png",
            guid: "018dc3f0-82c9-7d1a-8c3a-9d8b7e6f0008",
        }),
        Dependency({
            path: "assets/textures/walk_04.png",
            guid: "018dc3f0-82c9-7d1a-8c3a-9d8b7e6f0009",
        }),
        Dependency({
            path: "assets/audio/footstep.wav",
            guid: "018dc3f0-82c9-7d1a-8c3a-9d8b7e6f0010",
        }),
    ],
    references: [],
    timestamp: "2026-04-10T12:00:00Z",
    hash: "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
})
```

### UI 按钮动画示例

```ron
// UI 按钮动画示例
AnimationFile({
    version: "1.0",
    animation: Animation({
        name: "ButtonClickAnimation",
        description: "Button click animation",
        duration: 0.2,
        loop: false,
        speed: 1.0,
        blend_time: 0.05,
    }),
    tracks: [
        Track({
            name: "Transform",
            target: "transform",
            keyframes: [
                Keyframe({
                    time: 0.0,
                    properties: Properties({
                        scale: [1, 1, 1],
                    }),
                    easing: "ease_in_out",
                }),
                Keyframe({
                    time: 0.1,
                    properties: Properties({
                        scale: [0.9, 0.9, 1],
                    }),
                    easing: "ease_in_out",
                }),
                Keyframe({
                    time: 0.2,
                    properties: Properties({
                        scale: [1, 1, 1],
                    }),
                    easing: "ease_in_out",
                }),
            ],
        }),
        Track({
            name: "SpriteRenderer",
            target: "sprite_renderer",
            keyframes: [
                Keyframe({
                    time: 0.0,
                    properties: Properties({
                        color: [1, 1, 1, 1],
                    }),
                    easing: "ease_in_out",
                }),
                Keyframe({
                    time: 0.1,
                    properties: Properties({
                        color: [0.8, 0.8, 0.8, 1],
                    }),
                    easing: "ease_in_out",
                }),
                Keyframe({
                    time: 0.2,
                    properties: Properties({
                        color: [1, 1, 1, 1],
                    }),
                    easing: "ease_in_out",
                }),
            ],
        }),
    ],
    events: [
        Event({
            time: 0.1,
            name: "click_sound",
            params: Params({
                sound: "assets/audio/button_click.wav",
            }),
        }),
    ],
    dependencies: [
        Dependency({
            path: "assets/audio/button_click.wav",
            guid: "018dc3f0-82c9-7d1a-8c3a-9d8b7e6f0012",
        }),
    ],
    references: [],
    timestamp: "2026-04-10T12:30:00Z",
    hash: "a94a8fe5ccb19ba61c4c0873d391e987982fbbd3",
})
```

## 总结

*.anim 文件格式为 GG 游戏引擎提供了一种统一、有效的方式来管理动画。通过存储动画的关键帧、曲线、时间线和事件，它解决了游戏对象动画效果的问题。

这种设计使得 GG 游戏引擎能够：
- 快速创建和管理动画
- 支持多种动画类型（变换动画、精灵动画、材质动画等）
- 与组件系统无缝集成
- 提供灵活的动画事件系统
- 支持动画的版本控制和依赖管理

动画系统是 GG 游戏引擎中重要的组成部分，为游戏开发者提供了一种高效、灵活的方式来定义和管理游戏对象的动画效果。