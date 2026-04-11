# *.config 文件格式规范

## 概述

*.config 文件是 GG 游戏引擎用于存储配置信息的文件格式，包含游戏的各种配置参数，如游戏设置、输入映射、音频设置等。配置文件是游戏引擎的重要组成部分，用于控制游戏的各种行为和参数。

## 文件结构

一个完整的 *.config 文件是一个 TOML 格式的文本文件，用于描述游戏的配置参数。

### 基本结构

```von
# 配置文件
ConfigFile {
    version: "1.0",
    config: Config {
        name: "GameConfig",
        description: "Main game configuration",
        platform: "all",
        profile: "default",
    },
    settings: Settings {
        game: GameSettings {
            title: "My Game",
            version: "1.0.0",
            company: "Game Studio",
            copyright: "© 2026 Game Studio",
            default_scene: "assets/scenes/main.scene",
            fps_target: 60,
            resolution: Resolution {
                width: 1920,
                height: 1080,
                fullscreen: false,
                vsync: true,
            },
        },
        audio: AudioSettings {
            master_volume: 0.8,
            music_volume: 0.7,
            sfx_volume: 0.9,
            ambient_volume: 0.6,
        },
        input: InputSettings {
            keyboard: KeyboardMapping {
                move_forward: "W",
                move_backward: "S",
                move_left: "A",
                move_right: "D",
                jump: "Space",
                attack: "LeftMouseButton",
                interact: "E",
                pause: "Escape",
            },
            gamepad: GamepadMapping {
                move_forward: "LeftStickUp",
                move_backward: "LeftStickDown",
                move_left: "LeftStickLeft",
                move_right: "LeftStickRight",
                jump: "A",
                attack: "X",
                interact: "B",
                pause: "Start",
            },
        },
        graphics: GraphicsSettings {
            quality: "high",
            shadow_quality: "medium",
            anti_aliasing: "msaa_4x",
            texture_quality: "high",
            anisotropic_filtering: 16,
            bloom: true,
            depth_of_field: true,
            motion_blur: false,
        },
        network: NetworkSettings {
            max_players: 4,
            server_port: 7777,
            client_port: 7778,
            ping_timeout: 3000,
            reconnect_attempts: 3,
        },
        debug: DebugSettings {
            enabled: false,
            show_fps: true,
            show_hud: true,
            show_colliders: false,
            show_navmesh: false,
            console_enabled: true,
        },
    },
    dependencies: [
        Dependency {
            path: "assets/scenes/main.scene",
            guid: "018dc3f0-82c9-7d1a-8c3a-9d8b7e6f0001",
        },
    ],
    references: [],
    timestamp: "2026-04-10T12:00:00Z",
    hash: "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
}
```

## 字段详细说明

### 1. 版本信息

- **`version`**：字符串，配置格式版本号，用于向后兼容。

### 2. 配置信息

- **`config`**：对象，包含配置的基本信息。
  - **`name`**：字符串，配置名称。
  - **`description`**：字符串，配置描述。
  - **`platform`**：字符串，适用平台，如 `all`、`windows`、`macos`、`linux`、`android`、`ios`、`web` 等。
  - **`profile`**：字符串，配置文件，如 `default`、`low`、`medium`、`high` 等。

### 3. 配置设置

- **`settings`**：对象，包含游戏的各种配置设置。
  - **`game`**：对象，游戏基本设置。
    - **`title`**：字符串，游戏标题。
    - **`version`**：字符串，游戏版本。
    - **`company`**：字符串，游戏公司。
    - **`copyright`**：字符串，版权信息。
    - **`default_scene`**：字符串，默认场景路径。
    - **`fps_target`**：数字，目标帧率。
    - **`resolution`**：对象，分辨率设置。
      - **`width`**：数字，宽度。
      - **`height`**：数字，高度。
      - **`fullscreen`**：布尔值，是否全屏。
      - **`vsync`**：布尔值，是否启用垂直同步。
  - **`audio`**：对象，音频设置。
    - **`master_volume`**：数字，主音量 (0-1)。
    - **`music_volume`**：数字，音乐音量 (0-1)。
    - **`sfx_volume`**：数字，音效音量 (0-1)。
    - **`ambient_volume`**：数字，环境音量 (0-1)。
  - **`input`**：对象，输入设置。
    - **`keyboard`**：对象，键盘映射。
    - **`gamepad`**：对象，游戏手柄映射。
  - **`graphics`**：对象，图形设置。
    - **`quality`**：字符串，图形质量，如 `low`、`medium`、`high`、`ultra` 等。
    - **`shadow_quality`**：字符串，阴影质量，如 `off`、`low`、`medium`、`high` 等。
    - **`anti_aliasing`**：字符串，抗锯齿设置，如 `off`、`fxaa`、`msaa_2x`、`msaa_4x`、`msaa_8x` 等。
    - **`texture_quality`**：字符串，纹理质量，如 `low`、`medium`、`high` 等。
    - **`anisotropic_filtering`**：数字，各向异性过滤级别。
    - **`bloom`**：布尔值，是否启用 bloom 效果。
    - **`depth_of_field`**：布尔值，是否启用景深效果。
    - **`motion_blur`**：布尔值，是否启用运动模糊效果。
  - **`network`**：对象，网络设置。
    - **`max_players`**：数字，最大玩家数量。
    - **`server_port`**：数字，服务器端口。
    - **`client_port`**：数字，客户端端口。
    - **`ping_timeout`**：数字， ping 超时时间（毫秒）。
    - **`reconnect_attempts`**：数字，重连尝试次数。
  - **`debug`**：对象，调试设置。
    - **`enabled`**：布尔值，是否启用调试模式。
    - **`show_fps`**：布尔值，是否显示 FPS。
    - **`show_hud`**：布尔值，是否显示 HUD。
    - **`show_colliders`**：布尔值，是否显示碰撞器。
    - **`show_navmesh`**：布尔值，是否显示导航网格。
    - **`console_enabled`**：布尔值，是否启用控制台。

### 4. 依赖关系

- **`dependencies`**：数组，包含该配置依赖的其他资源。
  - 每个依赖项包含：
    - **`path`**：字符串，依赖资源的相对路径。
    - **`guid`**：字符串，依赖资源的全局唯一标识符。

### 5. 引用关系

- **`references`**：数组，包含引用该配置的其他资源。
  - 每个引用项包含：
    - **`path`**：字符串，引用资源的相对路径。
    - **`field`**：字符串，引用该配置的字段名称。

### 6. 元数据

- **`timestamp`**：字符串，ISO 格式的时间戳，表示该配置文件的最后修改时间。
- **`hash`**：字符串，配置文件的 SHA1 哈希值，用于检测配置文件是否发生变化。

## 配置类型

### 1. 全局配置

全局配置文件，包含游戏的基本设置，如游戏标题、版本、默认场景等。

### 2. 平台特定配置

平台特定的配置文件，包含针对特定平台的设置，如 Windows、macOS、Linux、Android、iOS、Web 等。

### 3. 配置文件

不同性能配置的配置文件，如低配置、中配置、高配置等。

### 4. 开发配置

开发环境的配置文件，包含调试设置、开发工具配置等。

## 使用场景

### 1. 游戏设置

- **游戏基本设置**：设置游戏的标题、版本、公司信息等。
- **分辨率设置**：设置游戏的分辨率、全屏模式等。
- **帧率设置**：设置游戏的目标帧率。

### 2. 音频设置

- **音量设置**：设置主音量、音乐音量、音效音量等。
- **音频设备设置**：设置音频输出设备。

### 3. 输入设置

- **键盘映射**：设置键盘按键映射。
- **游戏手柄映射**：设置游戏手柄按键映射。
- **鼠标设置**：设置鼠标灵敏度、反转等。

### 4. 图形设置

- **图形质量**：设置图形质量级别。
- **特效设置**：设置各种特效的开关和质量。
- **渲染设置**：设置渲染相关的参数。

### 5. 网络设置

- **多人游戏设置**：设置最大玩家数量、端口等。
- **网络质量设置**：设置网络相关的参数。

### 6. 调试设置

- **调试模式**：启用或禁用调试模式。
- **调试信息**：设置显示哪些调试信息。

## 最佳实践

1. **配置组织**：将配置按照功能组织到不同的配置文件中。
2. **默认配置**：提供合理的默认配置，确保游戏在不同环境下都能正常运行。
3. **配置验证**：在加载配置时验证配置的有效性，避免无效配置导致游戏崩溃。
4. **依赖管理**：定期检查并清理无效的依赖关系。
5. **版本控制**：将 .config 文件纳入版本控制系统，确保团队协作时的一致性。
6. **用户配置**：区分默认配置和用户配置，用户配置应该保存在用户目录中。
7. **哈希计算**：使用 SHA1 算法计算配置文件的哈希值。

## 示例完整文件

### 全局配置示例

```von
# 全局配置示例
ConfigFile {
    version: "1.0",
    config: Config {
        name: "GlobalConfig",
        description: "Global game configuration",
        platform: "all",
        profile: "default",
    },
    settings: Settings {
        game: GameSettings {
            title: "Adventure Game",
            version: "1.0.0",
            company: "Adventure Studio",
            copyright: "© 2026 Adventure Studio",
            default_scene: "assets/scenes/main.scene",
            fps_target: 60,
            resolution: Resolution {
                width: 1920,
                height: 1080,
                fullscreen: false,
                vsync: true,
            },
        },
        audio: AudioSettings {
            master_volume: 0.8,
            music_volume: 0.7,
            sfx_volume: 0.9,
            ambient_volume: 0.6,
        },
        input: InputSettings {
            keyboard: KeyboardMapping {
                move_forward: "W",
                move_backward: "S",
                move_left: "A",
                move_right: "D",
                jump: "Space",
                attack: "LeftMouseButton",
                interact: "E",
                pause: "Escape",
                inventory: "I",
                map: "M",
                character: "C",
            },
            gamepad: GamepadMapping {
                move_forward: "LeftStickUp",
                move_backward: "LeftStickDown",
                move_left: "LeftStickLeft",
                move_right: "LeftStickRight",
                jump: "A",
                attack: "X",
                interact: "B",
                pause: "Start",
                inventory: "Y",
                map: "Back",
                character: "RightShoulder",
            },
        },
        graphics: GraphicsSettings {
            quality: "high",
            shadow_quality: "medium",
            anti_aliasing: "msaa_4x",
            texture_quality: "high",
            anisotropic_filtering: 16,
            bloom: true,
            depth_of_field: true,
            motion_blur: false,
            ambient_occlusion: true,
            dynamic_lighting: true,
        },
        network: NetworkSettings {
            max_players: 4,
            server_port: 7777,
            client_port: 7778,
            ping_timeout: 3000,
            reconnect_attempts: 3,
            bandwidth_limit: 1024,
        },
        debug: DebugSettings {
            enabled: false,
            show_fps: true,
            show_hud: true,
            show_colliders: false,
            show_navmesh: false,
            console_enabled: true,
            log_level: "info",
        },
    },
    dependencies: [
        Dependency {
            path: "assets/scenes/main.scene",
            guid: "018dc3f0-82c9-7d1a-8c3a-9d8b7e6f0003",
        },
    ],
    references: [],
    timestamp: "2026-04-10T12:00:00Z",
    hash: "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
}
```

### 低配置示例

```von
# 低配置示例
ConfigFile {
    version: "1.0",
    config: Config {
        name: "LowConfig",
        description: "Low performance configuration",
        platform: "all",
        profile: "low",
    },
    settings: Settings {
        game: GameSettings {
            fps_target: 30,
            resolution: Resolution {
                width: 1280,
                height: 720,
                fullscreen: false,
                vsync: false,
            },
        },
        graphics: GraphicsSettings {
            quality: "low",
            shadow_quality: "off",
            anti_aliasing: "off",
            texture_quality: "low",
            anisotropic_filtering: 1,
            bloom: false,
            depth_of_field: false,
            motion_blur: false,
            ambient_occlusion: false,
            dynamic_lighting: false,
        },
    },
    dependencies: [],
    references: [],
    timestamp: "2026-04-10T12:30:00Z",
    hash: "a94a8fe5ccb19ba61c4c0873d391e987982fbbd3",
}
```

## 总结

*.config 文件格式为 GG 游戏引擎提供了一种统一、有效的方式来管理游戏配置。通过存储游戏的各种配置参数，它解决了游戏设置和参数管理的问题。

这种设计使得 GG 游戏引擎能够：
- 快速创建和管理游戏配置
- 支持多种配置类型（全局配置、平台特定配置、配置文件等）
- 提供灵活的配置参数设置
- 支持配置的版本控制和依赖管理
- 区分默认配置和用户配置

配置系统是 GG 游戏引擎中重要的组成部分，为游戏开发者提供了一种高效、灵活的方式来管理游戏的各种设置和参数。