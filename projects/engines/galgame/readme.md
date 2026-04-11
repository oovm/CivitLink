# gg-galgame

基于 GG 元引擎框架的 Galgame（视觉小说）游戏引擎，提供完整的视觉小说游戏开发和运行能力。

## 功能特性

- **完整的视觉小说功能**：支持对话系统、角色立绘、场景切换等核心功能
- **插件化架构**：基于 GG 元引擎的插件系统，可灵活扩展功能
- **ECS 架构**：使用实体组件系统，提供高效的游戏逻辑管理
- **资源管理**：支持图像、音频等资源的加载和管理
- **配置系统**：提供灵活的游戏配置选项
- **跨平台支持**：支持桌面平台运行

## 架构设计

Galgame 引擎基于 GG 元引擎框架，采用以下架构：

- **核心引擎**：负责游戏生命周期管理、渲染、输入处理等基础功能
- **插件系统**：集成多个专用插件，提供视觉小说特有的功能
- **ECS 系统**：使用实体组件系统管理游戏对象和逻辑
- **资源系统**：负责资源的加载、管理和释放

### 集成的插件

- **DialoguePlugin**：提供对话系统，支持文本显示、选择分支等功能
- **PortraitPlugin**：提供角色立绘系统，支持角色显示、表情切换等功能
- **SceneTransitionPlugin**：提供场景切换系统，支持各种过渡效果
- **SavePlugin**：提供游戏存档系统，支持保存和加载游戏进度

## 配置选项

Galgame 引擎支持以下配置选项：

### 游戏信息
- `name`：游戏名称
- `version`：游戏版本
- `initial_scene`：初始场景

### 显示配置
- `width`：窗口宽度（默认 1280）
- `height`：窗口高度（默认 720）
- `fullscreen`：是否全屏（默认 false）

### 音频配置
- `master_volume`：主音量（默认 1.0）
- `bgm_volume`：BGM 音量（默认 0.8）
- `se_volume`：音效音量（默认 1.0）

## 使用方法

### 作为游戏运行时

通过游戏启动器或编辑器运行 Galgame 游戏：

1. 创建游戏配置文件 `game.toml`
2. 编写游戏脚本和资源
3. 使用 Galgame 引擎运行游戏

### 示例配置

```toml
[game]
name = "My Galgame"
version = "1.0.0"
initial_scene = "start"

[display]
width = 1280
height = 720
fullscreen = false

[audio]
master_volume = 1.0
bgm_volume = 0.8
se_volume = 1.0
```

### 游戏脚本示例

```script
// start.script
show background "bg1.png"
show character "character1.png" at center

"Hello, welcome to my galgame!"

choice {
    "Option 1": {
        "You chose option 1!"
    }
    "Option 2": {
        "You chose option 2!"
    }
}

"The end."
```

## 项目结构

```
galgame/
├── src/
│   ├── config.rs     # 配置模块
│   ├── engine.rs     # 核心引擎
│   └── main.rs       # 入口点
├── template/         # 游戏模板
│   ├── scripts/
│   │   └── start.script
│   └── game.toml
├── Cargo.toml        # 项目配置
└── readme.md         # 文档
```

## 与其他引擎的关系

Galgame 引擎是 GG 元引擎框架下的一个具体游戏引擎实现，与 STG 和 Platformer 引擎并列，专注于视觉小说游戏类型。

## 扩展和定制

Galgame 引擎支持通过以下方式扩展和定制：

1. **添加新插件**：通过 GG 元引擎的插件系统添加新功能
2. **修改配置**：通过配置文件调整游戏行为
3. **自定义脚本**：通过游戏脚本实现自定义游戏逻辑
4. **扩展资源**：添加自定义图像、音频等资源

## 开发环境

- **语言**：Rust
- **依赖**：GG 元引擎核心组件
- **构建工具**：Cargo

## 构建和运行

```bash
# 构建引擎
cargo build

# 运行示例游戏
cargo run --example basic

# 运行游戏（带命令行参数）
cargo run -- --project path/to/game

# 以编辑器模式运行
cargo run -- --editor
```

### 命令行参数

- `--project <path>`：指定游戏项目路径
- `--editor`：以编辑器模式运行

## 后续计划

- 支持更多平台
- 增加更多视觉效果
- 提供更丰富的脚本功能
- 完善编辑器集成