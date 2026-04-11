# 快速开始

本节将帮助不同角色的开发者快速上手 GG 游戏引擎。根据你的角色选择对应的入门路径：

- **引擎开发人员**：使用 Rust 开发引擎核心和插件
- **游戏开发人员**：使用 Valkyrie 脚本开发游戏逻辑
- **Mod 开发者**：填充内容资源，无需编程

---

# 引擎开发人员

引擎开发人员需要 Rust 开发环境，负责开发引擎核心、插件和底层功能。

## 环境搭建

### 1. 安装 Rust

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

gg 元引擎需要稳定版 Rust 工具链：

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

## 创建引擎插件

### 1. 创建自定义插件

在 `src/` 目录下创建 `plugins/` 文件夹，并添加 `my_game_plugin.rs`：

```rust
//! 我的游戏插件
//! 
//! 提供基本的游戏功能和系统

use gg_core::prelude::*;
use gg_engine::prelude::*;

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

/// 我的游戏插件
pub struct MyGamePlugin;

impl Plugin for MyGamePlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(GameConfig::default());
        app.add_systems(Startup, init_game_system);
        app.add_systems(Update, player_movement_system);
    }
}
```

### 2. 编译运行

```bash
# 普通运行
cargo run

# 启用编辑器功能
cargo run --features editor

# 发布模式编译（优化性能）
cargo run --release
```

---

# 游戏开发人员

游戏开发人员使用 Valkyrie 脚本开发游戏逻辑，无需 Rust 环境。

## 环境搭建

### 1. 安装 GG Editor

下载并安装 GG Editor（图形化编辑器）。

### 2. 创建游戏项目

在 GG Editor 中创建新项目，项目结构如下：

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
│   └── main.script
├── dlc/
└── mods/
```

### 3. 编写 Valkyrie 脚本

创建 `scripts/main.script` 文件：

```valkyrie
/// 游戏入口脚本
/// 
/// 定义游戏的基本配置和初始化逻辑

using gg::prelude::*;

/// 游戏配置
@config
struct GameConfig {
    title: string = "我的游戏",
    version: string = "0.1.0",
    window_width: i32 = 1280,
    window_height: i32 = 720,
}

/// 玩家状态
state PlayerState {
    health: f32 = 100.0,
    speed: f32 = 5.0,
    position: Vec3 = Vec3::ZERO,
}

/// 主入口函数
@main
fn main() {
    console::log("游戏启动！");
    
    // 初始化玩家
    let player = PlayerState::new();
    
    // 游戏主循环
    game::run(micro(delta_time) {
        update_player(player, delta_time);
    });
}

/// 更新玩家状态
fn update_player(player: mut PlayerState, delta_time: f32) {
    // 处理输入
    let mut movement = Vec3::ZERO;
    
    if input::key_pressed("W") or input::key_pressed("Up") {
        movement.y += 1.0;
    }
    if input::key_pressed("S") or input::key_pressed("Down") {
        movement.y -= 1.0;
    }
    if input::key_pressed("A") or input::key_pressed("Left") {
        movement.x -= 1.0;
    }
    if input::key_pressed("D") or input::key_pressed("Right") {
        movement.x += 1.0;
    }
    
    // 更新位置
    if movement.length() > 0.0 {
        movement = movement.normalize() * player.speed * delta_time;
        player.position = player.position + movement;
    }
}
```

### 4. 配置游戏

创建 `game.toml` 配置文件：

```toml
[game]
name = "我的第一个游戏"
version = "0.1.0"

[display]
width = 1280
height = 720
fullscreen = false

[audio]
master_volume = 1.0
bgm_volume = 0.8
se_volume = 1.0
```

### 5. 运行游戏

在 GG Editor 中点击"运行"按钮，或使用命令行：

```bash
gg-cli run
```

---

# Mod 开发者

Mod 开发者无需编程，只需填充内容资源（图片、音频、文本等）。

## 环境搭建

### 1. 安装 GG Editor

下载并安装 GG Editor（图形化编辑器）。

### 2. 创建 Mod 项目

在 GG Editor 中创建新 Mod 项目，项目结构如下：

```
my-mod/
├── mod.toml
├── assets/
│   ├── images/
│   │   └── characters/
│   ├── audio/
│   │   ├── bgm/
│   │   └── se/
│   └── fonts/
└── data/
    └── characters.json
```

### 3. 配置 Mod

创建 `mod.toml` 配置文件：

```toml
[mod]
name = "我的Mod"
version = "0.1.0"
author = "开发者名称"
description = "Mod描述"

[dependencies]
# 依赖的其他Mod（可选）
```

### 4. 添加内容资源

#### 添加角色

在 `data/characters.json` 中定义角色：

```json
{
    "characters": [
        {
            "id": "hero",
            "name": "勇者",
            "portrait": "assets/images/characters/hero.png",
            "stats": {
                "health": 100,
                "attack": 10,
                "defense": 5
            }
        },
        {
            "id": "villain",
            "name": "反派",
            "portrait": "assets/images/characters/villain.png",
            "stats": {
                "health": 200,
                "attack": 15,
                "defense": 8
            }
        }
    ]
}
```

#### 添加对话

在 `data/dialogues.json` 中定义对话：

```json
{
    "dialogues": [
        {
            "id": "intro",
            "speaker": "hero",
            "text": "你好，世界！",
            "portrait_expression": "happy"
        },
        {
            "id": "response",
            "speaker": "villain",
            "text": "哼，有意思。",
            "portrait_expression": "smirk"
        }
    ]
}
```

### 5. 打包发布

在 GG Editor 中点击"打包"按钮，生成 `.ggmod` 文件：

```bash
gg-cli package my-mod
```

---

# 角色对比

| 特性 | 引擎开发人员 | 游戏开发人员 | Mod 开发者 |
|------|-------------|-------------|-----------|
| 编程语言 | Rust | Valkyrie 脚本 | 无需编程 |
| 开发环境 | Rust 工具链 | GG Editor | GG Editor |
| 主要工作 | 引擎核心、插件 | 游戏逻辑、系统 | 内容资源、数据 |
| 技术要求 | 高 | 中 | 低 |
| 输出产物 | 动态库、插件 | 字节码、脚本 | 资源包、Mod |

---

# 下一步

- [核心优势](/guide/advantages) - 了解 gg 元引擎的设计理念
- [架构设计](/architecture/overview) - 深入了解 gg 的架构
- [Valkyrie 脚本指南](/guide/valkyrie) - 学习 Valkyrie 脚本语言
- [Mod 开发指南](/guide/mod-development) - 学习 Mod 开发流程
