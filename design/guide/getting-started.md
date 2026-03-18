# 快速开始

本节将帮助你在 10 分钟内创建第一个基于 gwg 元引擎框架的游戏引擎项目。

## 环境搭建

### 1. 安装 Rust

首先，你需要安装 Rust 编程语言。gwg 元引擎基于 Rust 开发，利用其强大的性能和安全性特性。

访问 [Rust 官网](https://www.rust-lang.org/tools/install) 下载并安装最新版本的 Rust：

**Windows:**
```bash
# 使用 rustup 安装
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

安装完成后，验证 Rust 是否已正确安装：

```bash
rustc --version
cargo --version
```

### 2. 配置工具链

gwg 元引擎需要稳定版 Rust 工具链：

```bash
# 安装稳定版工具链（如果尚未安装）
rustup default stable

# 验证工具链
rustup show
```

对于跨平台构建（如 WASM），需要安装相应的目标：

```bash
# 安装 WASM 目标
rustup target add wasm32-unknown-unknown
```

### 3. 系统依赖（可选）

根据目标平台，可能需要安装额外的系统依赖：

**Windows:**
- 安装 Visual Studio Build Tools（包含 C++ 编译器）

**macOS:**
```bash
xcode-select --install
```

**Linux (Ubuntu/Debian):**
```bash
sudo apt install build-essential pkg-config libssl-dev libasound2-dev
```

## 创建第一个引擎项目

### 1. 初始化项目

使用 Cargo 创建新的 Rust 项目：

```bash
cargo new my-game-engine
cd my-game-engine
```

### 2. 配置 Cargo.toml

编辑 `Cargo.toml`，添加 gwg 元引擎框架的依赖：

```toml
[package]
name = "my-game-engine"
version = "0.1.0"
edition = "2021"

[dependencies]
# 核心框架
gwg-core = { path = "../../crates/frameworks/gwg-core" }
gwg-platform = { path = "../../crates/platforms" }
gwg-runtime = { path = "../../crates/runtime" }
gwg-engine = { path = "../../crates/engine" }

# 内置模块
gwg-modules-rendering = { path = "../../crates/modules/rendering" }
gwg-modules-audio = { path = "../../crates/modules/audio" }
gwg-modules-ui = { path = "../../crates/modules/ui" }
gwg-modules-input = { path = "../../crates/modules/input" }

# 工具库
tokio = { version = "1", features = ["full"] }
serde = { version = "1", features = ["derive"] }
tracing = "0.1"
tracing-subscriber = "0.3"

[features]
default = ["desktop"]
desktop = []
editor = ["gwg-engine/editor"]
```

### 3. 编写 main.rs

创建 `src/main.rs` 文件，实现一个简单的游戏引擎入口：

```rust
use gwg_engine::prelude::*;
use gwg_modules_rendering::RenderPlugin;
use gwg_modules_audio::AudioPlugin;
use gwg_modules_ui::UIPlugin;
use gwg_modules_input::InputPlugin;

/// 主函数
fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 初始化日志系统
    tracing_subscriber::fmt::init();

    // 构建并运行引擎
    EngineBuilder::new()
        .with_name("我的游戏引擎")
        .with_version("0.1.0")
        // 添加核心插件
        .add_plugin(RenderPlugin::new_2d())
        .add_plugin(AudioPlugin::default())
        .add_plugin(UIPlugin::default())
        .add_plugin(InputPlugin::default())
        // 启用编辑器（可选）
        .with_editor(cfg!(feature = "editor"))
        // 构建并运行
        .build()?
        .run();

    Ok(())
}
```

## 简单的插件使用示例

### 1. 创建自定义插件

让我们创建一个简单的游戏插件，添加一些基本的游戏逻辑。在 `src/` 目录下创建 `plugins/` 文件夹，并添加 `my_game_plugin.rs`：

```rust
//! 我的游戏插件
//! 
//! 提供基本的游戏功能和系统

use gwg_core::prelude::*;
use gwg_engine::prelude::*;

/// 游戏配置资源
#[derive(Resource, Debug, Clone)]
pub struct GameConfig {
    /// 游戏标题
    pub title: String,
    /// 初始关卡
    pub starting_level: u32,
}

impl Default for GameConfig {
    fn default() -> Self {
        Self {
            title: "我的游戏".to_string(),
            starting_level: 1,
        }
    }
}

/// 玩家组件
#[derive(Component, Debug, Clone)]
pub struct Player {
    /// 生命值
    pub health: f32,
    /// 移动速度
    pub speed: f32,
}

impl Default for Player {
    fn default() -> Self {
        Self {
            health: 100.0,
            speed: 5.0,
        }
    }
}

/// 玩家移动系统
fn player_movement_system(
    time: Res<Time>,
    input: Res<InputState>,
    mut query: Query<(&Player, &mut Transform)>,
) {
    let delta_time = time.delta_seconds();
    
    for (player, mut transform) in query.iter_mut() {
        let mut movement = Vec3::ZERO;
        
        if input.key_pressed(KeyCode::W) || input.key_pressed(KeyCode::Up) {
            movement.y += 1.0;
        }
        if input.key_pressed(KeyCode::S) || input.key_pressed(KeyCode::Down) {
            movement.y -= 1.0;
        }
        if input.key_pressed(KeyCode::A) || input.key_pressed(KeyCode::Left) {
            movement.x -= 1.0;
        }
        if input.key_pressed(KeyCode::D) || input.key_pressed(KeyCode::Right) {
            movement.x += 1.0;
        }
        
        if movement.length_squared() > 0.0 {
            movement = movement.normalize() * player.speed * delta_time;
            transform.translation += movement;
        }
    }
}

/// 初始化游戏系统
fn init_game_system(
    mut commands: Commands,
    config: Res<GameConfig>,
) {
    tracing::info!("初始化游戏: {}", config.title);
    
    // 生成玩家实体
    commands.spawn((
        Player::default(),
        Transform::from_xyz(0.0, 0.0, 0.0),
        Name::new("玩家"),
    ));
}

/// 我的游戏插件
pub struct MyGamePlugin;

impl Plugin for MyGamePlugin {
    fn build(&self, app: &mut App) {
        // 插入资源
        app.insert_resource(GameConfig::default());
        
        // 添加系统
        app.add_systems(Startup, init_game_system);
        app.add_systems(Update, player_movement_system);
    }
}
```

### 2. 更新 main.rs 使用自定义插件

修改 `src/main.rs` 以包含你的自定义插件：

```rust
use gwg_engine::prelude::*;
use gwg_modules_rendering::RenderPlugin;
use gwg_modules_audio::AudioPlugin;
use gwg_modules_ui::UIPlugin;
use gwg_modules_input::InputPlugin;

// 导入自定义插件
mod plugins;
use plugins::my_game_plugin::MyGamePlugin;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();

    EngineBuilder::new()
        .with_name("我的游戏引擎")
        .with_version("0.1.0")
        .add_plugin(RenderPlugin::new_2d())
        .add_plugin(AudioPlugin::default())
        .add_plugin(UIPlugin::default())
        .add_plugin(InputPlugin::default())
        // 添加自定义插件
        .add_plugin(MyGamePlugin)
        .with_editor(cfg!(feature = "editor"))
        .build()?
        .run();

    Ok(())
}
```

同时创建 `src/plugins/mod.rs`：

```rust
pub mod my_game_plugin;
```

## 运行和测试

### 1. 编译运行

使用 Cargo 编译并运行你的引擎：

```bash
# 普通运行
cargo run

# 启用编辑器功能
cargo run --features editor

# 发布模式编译（优化性能）
cargo run --release
```

### 2. 跨平台构建

#### 构建 Windows 版本：

```bash
cargo build --release --target x86_64-pc-windows-msvc
```

可执行文件将位于 `target/x86_64-pc-windows-msvc/release/my-game-engine.exe`。

#### 构建 WebAssembly (H5) 版本：

首先安装 wasm-bindgen：

```bash
cargo install wasm-bindgen-cli
```

然后构建：

```bash
# 构建 WASM
cargo build --release --target wasm32-unknown-unknown

# 生成绑定代码
wasm-bindgen --out-dir ./web --target web target/wasm32-unknown-unknown/release/my-game-engine.wasm
```

在 `web/` 目录创建一个简单的 HTML 文件 `index.html`：

```html
<!DOCTYPE html>
<html>
<head>
    <meta charset="UTF-8">
    <title>我的游戏引擎</title>
    <style>
        body { margin: 0; padding: 0; background: #000; }
        canvas { display: block; width: 100vw; height: 100vh; }
    </style>
</head>
<body>
    <script type="module">
        import init from './my-game-engine.js';
        
        async function run() {
            await init();
        }
        
        run();
    </script>
</body>
</html>
```

使用本地服务器测试：

```bash
# 使用 Python 3 启动简单服务器
python -m http.server 8080
```

然后在浏览器访问 `http://localhost:8080`。

### 3. 创建游戏项目

引擎运行后，你可以创建游戏项目。在引擎工作目录下创建游戏项目结构：

```
my-first-game/
├── game.toml
├── assets/
│   ├── images/
│   │   └── characters/
│   ├── audio/
│   │   ├── bgm/
│   │   └── se/
│   └── fonts/
├── scripts/
│   └── main.yarn
├── dlc/
└── mods/
```

创建 `game.toml` 配置文件：

```toml
[game]
name = "我的第一个游戏"
version = "0.1.0"
engine = "my-game-engine"
engine_version = "0.1.0"

[display]
width = 1280
height = 720
fullscreen = false

[audio]
master_volume = 1.0
bgm_volume = 0.8
se_volume = 1.0
```

## 下一步

- [核心优势](/guide/advantages) - 了解 gwg 元引擎的设计理念
- [架构设计](/architecture/overview) - 深入了解 gwg 的架构
- [模块文档](/modules/rendering) - 了解各个功能模块的使用
