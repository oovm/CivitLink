# API 概览

GG 元游戏引擎提供一套完整的 Rust API，支持从资深玩家开发引擎插件，到普通玩家创作游戏内容的完整链路。本文档概述了引擎的核心 API 概念、使用示例、模块分类和命名约定。

## 核心 API 概念

### EngineBuilder

引擎构建器，用于组合插件、配置运行时环境、最终构建并运行游戏引擎。

```rust
pub struct EngineBuilder {
    // 内部字段
}

impl EngineBuilder {
    /// 创建新的引擎构建器
    pub fn new() -> Self;
    
    /// 添加插件
    pub fn add_plugin<P: Plugin>(mut self, plugin: P) -> Self;
    
    /// 启用编辑器模式
    pub fn with_editor(mut self, enabled: bool) -> Self;
    
    /// 构建引擎
    pub fn build(self) -> Engine;
}
```

### Plugin trait

插件是引擎功能的扩展单元，每个插件可以定义组件、系统、资源，并与 ECS 世界交互。

```rust
pub trait Plugin {
    /// 插件名称
    fn name(&self) -> &'static str;
    
    /// 构建插件，注册组件、系统和资源
    fn build(&self, app: &mut App);
}
```

### World

世界是 ECS 的核心容器，存储所有实体、组件和资源。

```rust
pub struct World {
    // 内部字段
}

impl World {
    /// 创建空世界
    pub fn new() -> Self;
    
    /// 生成新实体
    pub fn spawn(&mut self) -> EntityBuilder;
    
    /// 获取实体引用
    pub fn entity(&self, entity: Entity) -> EntityRef;
    
    /// 获取资源
    pub fn get_resource<T: Resource>(&self) -> Option<&T>;
    
    /// 获取可变资源
    pub fn get_resource_mut<T: Resource>(&mut self) -> Option<&mut T>;
    
    /// 插入资源
    pub fn insert_resource<T: Resource>(&mut self, resource: T);
}
```

### Entity

实体是游戏对象的唯一标识符，本身不包含数据，只是一个轻量级的句柄。

```rust
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Entity {
    // 内部字段
}

impl Entity {
    /// 实体 ID
    pub fn id(self) -> u64;
    
    /// 实体生成版本
    pub fn generation(self) -> u32;
}
```

### Component

组件是附加到实体上的数据，定义实体的行为和属性。

```rust
pub trait Component: Send + Sync + 'static {
    // 可选：组件存储类型
    type Storage: ComponentStorage;
}

/// 组件派生宏
#[derive(Component)]
struct Position {
    x: f32,
    y: f32,
}

#[derive(Component)]
struct Velocity {
    x: f32,
    y: f32,
}
```

### System

系统是对组件数据进行操作的函数，定义游戏逻辑。

```rust
/// 系统 trait
pub trait System: Send + Sync + 'static {
    fn run(&mut self, world: &mut World);
}

/// 系统函数示例
fn move_system(
    mut query: Query<(&mut Position, &Velocity)>,
) {
    for (mut pos, vel) in query.iter_mut() {
        pos.x += vel.x;
        pos.y += vel.y;
    }
}
```

### Schedule

调度器组织系统的执行顺序，支持并行调度。

```rust
pub struct Schedule {
    // 内部字段
}

impl Schedule {
    /// 添加系统
    pub fn add_system<S: System>(&mut self, system: S);
    
    /// 添加系统到指定阶段
    pub fn add_system_to_stage<S: System>(&mut self, stage: impl StageLabel, system: S);
    
    /// 运行调度器
    pub fn run(&mut self, world: &mut World);
}

/// 内置执行阶段
pub enum CoreStage {
    /// 第一帧初始化
    First,
    /// 预更新
    PreUpdate,
    /// 更新
    Update,
    /// 后更新
    PostUpdate,
    /// 渲染前
    PreRender,
    /// 渲染
    Render,
    /// 最后
    Last,
}
```

### Resource

资源是单例数据，存储全局状态。

```rust
pub trait Resource: Send + Sync + 'static {}

/// 资源派生宏
#[derive(Resource)]
struct GameConfig {
    screen_width: u32,
    screen_height: u32,
    title: String,
}

#[derive(Resource)]
struct Time {
    delta_seconds: f32,
    elapsed_seconds: f32,
}
```

## API 使用示例

### 创建引擎

```rust
use gg_engine::prelude::*;

fn main() {
    EngineBuilder::new()
        .add_plugin(RenderPlugin::new_2d())
        .add_plugin(AudioPlugin::default())
        .add_plugin(UIPlugin::default())
        .add_plugin(GalgamePlugin::new())
        .with_editor(true)
        .build()
        .run();
}
```

### 定义组件

```rust
use gg_engine::prelude::*;

/// 位置组件
#[derive(Component, Debug, Clone, Copy)]
pub struct Position {
    pub x: f32,
    pub y: f32,
}

impl Position {
    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }
}

/// 速度组件
#[derive(Component, Debug, Clone, Copy)]
pub struct Velocity {
    pub x: f32,
    pub y: f32,
}

impl Velocity {
    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }
}

/// 精灵组件
#[derive(Component)]
pub struct Sprite {
    pub texture: Handle<Texture>,
    pub size: (f32, f32),
    pub color: Color,
}
```

### 编写系统

```rust
use gg_engine::prelude::*;

/// 移动系统
pub fn move_system(
    time: Res<Time>,
    mut query: Query<(&mut Position, &Velocity)>,
) {
    let delta = time.delta_seconds();
    for (mut pos, vel) in query.iter_mut() {
        pos.x += vel.x * delta;
        pos.y += vel.y * delta;
    }
}

/// 输入系统
pub fn input_system(
    input: Res<Input>,
    mut query: Query<&mut Velocity>,
) {
    let speed = 100.0;
    for mut vel in query.iter_mut() {
        vel.x = 0.0;
        vel.y = 0.0;
        
        if input.key_pressed(KeyCode::W) || input.key_pressed(KeyCode::Up) {
            vel.y = speed;
        }
        if input.key_pressed(KeyCode::S) || input.key_pressed(KeyCode::Down) {
            vel.y = -speed;
        }
        if input.key_pressed(KeyCode::A) || input.key_pressed(KeyCode::Left) {
            vel.x = -speed;
        }
        if input.key_pressed(KeyCode::D) || input.key_pressed(KeyCode::Right) {
            vel.x = speed;
        }
    }
}

/// 渲染系统
pub fn render_sprite_system(
    mut renderer: ResMut<Renderer>,
    camera: Res<Camera>,
    query: Query<(&Position, &Sprite)>,
) {
    for (pos, sprite) in query.iter() {
        renderer.draw_sprite(
            &sprite.texture,
            pos.x,
            pos.y,
            sprite.size.0,
            sprite.size.1,
            sprite.color,
        );
    }
}
```

### 注册插件

```rust
use gg_engine::prelude::*;

/// 简单游戏插件
pub struct SimpleGamePlugin;

impl Plugin for SimpleGamePlugin {
    fn name(&self) -> &'static str {
        "SimpleGame"
    }
    
    fn build(&self, app: &mut App) {
        // 注册资源
        app.insert_resource(GameConfig {
            screen_width: 800,
            screen_height: 600,
            title: "简单游戏".to_string(),
        });
        
        // 注册系统
        app.add_system_to_stage(CoreStage::Update, input_system);
        app.add_system_to_stage(CoreStage::Update, move_system);
        app.add_system_to_stage(CoreStage::Render, render_sprite_system);
        
        // 初始化游戏
        app.add_startup_system(setup);
    }
}

/// 启动系统
fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    // 加载纹理
    let texture = asset_server.load("player.png");
    
    // 生成玩家实体
    commands.spawn((
        Position::new(400.0, 300.0),
        Velocity::new(0.0, 0.0),
        Sprite {
            texture,
            size: (64.0, 64.0),
            color: Color::WHITE,
        },
    ));
    
    // 生成相机
    commands.spawn(Camera2d::default());
}
```

## 模块 API 分类

### 核心层 (Core)

| 模块 | 说明 |
|------|------|
| `gg_core::ecs` | ECS 核心（基于 bevy_ecs） |
| `gg_core::asset` | 资源管理系统（加载、缓存、句柄） |
| `gg_core::schedule` | 系统调度器扩展 |
| `gg_core::world` | 世界管理 |
| `gg_core::reflection` | 反射系统 |

### 平台抽象层 (Platform Abstraction)

| 模块 | 说明 |
|------|------|
| `gg_platform::window` | 窗口管理（winit 封装） |
| `gg_platform::input` | 输入抽象（键盘、鼠标、触摸、手柄） |
| `gg_platform::graphics` | 图形抽象（wgpu 封装） |
| `gg_platform::audio` | 音频抽象 |
| `gg_platform::filesystem` | 文件系统抽象（AssetIo trait） |
| `gg_platform::time` | 时间抽象 |

### 运行时层 (Runtime)

| 模块 | 说明 |
|------|------|
| `gg_runtime::app` | 应用生命周期管理 |
| `gg_runtime::scene` | 场景管理 |
| `gg_runtime::prefab` | 预制体系统 |
| `gg_runtime::serialization` | 序列化 |

### 虚拟机层 (VM)

| 模块 | 说明 |
|------|------|
| `gg_vm::core` | 虚拟机核心接口 |
| `gg_vm::wasmtime` | Wasmtime 后端（桌面） |
| `gg_vm::wasmi` | 轻量级解释器后端（嵌入式） |
| `gg_vm::api` | 暴露给脚本的 Rust API |
| `gg_vm::bindings` | 语言绑定生成 |

### 引擎插件框架 (Engine)

| 模块 | 说明 |
|------|------|
| `gg_engine::plugin` | 插件 trait 定义 |
| `gg_engine::registry` | 插件注册表 |
| `gg_engine::builder` | 引擎构建器 |
| `gg_engine::manifest` | 引擎清单处理 |

### 编辑器框架 (Editor)

| 模块 | 说明 |
|------|------|
| `gg_editor::ui` | 编辑器 UI 组件（egui） |
| `gg_editor::inspector` | 属性编辑器 |
| `gg_editor::scene_view` | 场景视图 |
| `gg_editor::asset_browser` | 资源浏览器 |
| `gg_editor::plugin` | 编辑器插件系统 |

### 内置功能模块 (Modules)

| 模块 | 说明 |
|------|------|
| `gg_modules::rendering` | 渲染模块（2D/3D、精灵、文本、相机） |
| `gg_modules::physics` | 物理模块（2D/3D、碰撞检测、Rapier） |
| `gg_modules::animation` | 动画模块（精灵动画、变换动画、状态机） |
| `gg_modules::audio` | 音频模块（播放器、混音器、空间音频） |
| `gg_modules::ui` | UI 模块（核心、控件、布局、交互） |
| `gg_modules::input` | 输入模块（键盘、鼠标、触摸、手柄、映射） |
| `gg_modules::network` | 网络模块（核心、客户端、服务器、同步） |

## 命名约定

| 类型 | 命名 | 示例 |
|------|------|------|
| Trait | Xxx | Plugin, Component, Resource, System |
| Struct | Xxx | EngineBuilder, World, Entity, Position |
| Enum | Xxx | CoreStage, KeyCode, Color |
| 资源 | Xxx（实现 Resource trait） | GameConfig, Time, Input |
| 组件 | Xxx（实现 Component trait） | Position, Velocity, Sprite |
| 系统 | xxx_system | move_system, input_system, render_system |
| 插件 | XxxPlugin | RenderPlugin, AudioPlugin, GalgamePlugin |
| Builder | XxxBuilder | EngineBuilder, ScheduleBuilder |
| Handle | Handle&lt;Xxx&gt; | Handle&lt;Texture&gt;, Handle&lt;Audio&gt; |
| Query | Query&lt;...&gt; | Query&lt;(&amp;mut Position, &amp;Velocity)&gt; |
| Res | Res&lt;Xxx&gt; / ResMut&lt;Xxx&gt; | Res&lt;Time&gt;, ResMut&lt;Renderer&gt; |
| Commands | Commands | 用于生成/销毁实体、插入/移除组件 |

### 模块命名

- 小写单词，用下划线分隔：`gg_core`, `gg_platform`, `gg_modules::rendering`
- 避免缩写，除非是广泛认可的（如 `ecs`, `vm`）

### 函数命名

- 小写单词，用下划线分隔：`spawn`, `get_resource`, `add_system`
- 布尔查询函数用 `is_` 或 `has_` 前缀：`is_empty`, `has_component`
- 转换函数用 `to_` 或 `into_` 前缀：`to_string`, `into_inner`

### 常量命名

- 全大写，用下划线分隔：`MAX_ENTITIES`, `DEFAULT_SCREEN_WIDTH`
