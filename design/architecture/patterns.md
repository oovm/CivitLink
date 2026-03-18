# 设计模式

GWG 元游戏引擎采用多种成熟的设计模式，以确保系统的可扩展性、可维护性和跨平台兼容性。在诺斯替（Gnostic）宇宙观的隐喻下，这些设计模式对应着不同的宇宙法则：**ECS 模式**是万物生成的基始，**插件模式**是流溢的通道，**平台抽象模式**是太一的遍在性，**数据驱动设计**是普累若麻的显现，**沙箱模式**是异端知识的安全边界。本文档详细介绍引擎核心使用的设计模式及其实现方式。

## 1. ECS 模式（Entity-Component-System）

### 模式概述

ECS 是一种数据驱动的架构模式，将游戏对象分解为三个核心概念：
- **Entity（实体）**：唯一标识符，代表游戏世界中的一个"事物"
- **Component（组件）**：纯数据结构，不包含逻辑
- **System（系统）**：纯逻辑，处理具有特定组件组合的实体

### 核心优势

1. **性能优化**：数据局部性好，缓存友好，支持并行处理
2. **组合优于继承**：通过组件组合实现灵活的对象行为
3. **数据与逻辑分离**：清晰的职责划分，易于测试和维护

### 代码示例

#### 组件定义

```rust
use bevy_ecs::prelude::*;
use serde::{Deserialize, Serialize};

/// 位置组件，存储实体在2D空间中的坐标
#[derive(Component, Clone, Copy, Debug, Serialize, Deserialize)]
pub struct Position {
    /// X 坐标
    pub x: f32,
    /// Y 坐标
    pub y: f32,
}

/// 速度组件，存储实体的移动速度
#[derive(Component, Clone, Copy, Debug, Serialize, Deserialize)]
pub struct Velocity {
    /// X 方向速度
    pub vx: f32,
    /// Y 方向速度
    pub vy: f32,
}

/// 精灵组件，存储渲染所需的资源句柄
#[derive(Component, Debug)]
pub struct Sprite {
    /// 纹理资源句柄
    pub texture_handle: Handle<Texture>,
    /// 精灵宽度
    pub width: f32,
    /// 精灵高度
    pub height: f32,
}
```

#### 系统实现

```rust
use bevy_ecs::prelude::*;

/// 移动系统：根据速度更新位置
pub fn move_system(
    time: Res<Time>,
    mut query: Query<(&mut Position, &Velocity)>,
) {
    let delta = time.delta_seconds();
    for (mut pos, vel) in query.iter_mut() {
        pos.x += vel.vx * delta;
        pos.y += vel.vy * delta;
    }
}

/// 渲染系统：绘制所有精灵
pub fn render_system(
    graphics: Res<GraphicsContext>,
    query: Query<(&Position, &Sprite)>,
) {
    for (pos, sprite) in query.iter() {
        graphics.draw_sprite(
            sprite.texture_handle,
            pos.x,
            pos.y,
            sprite.width,
            sprite.height,
        );
    }
}
```

#### 实体创建与系统调度

```rust
use bevy_ecs::prelude::*;

fn main() {
    let mut world = World::new();
    
    let mut schedule = Schedule::default();
    schedule.add_systems((move_system, render_system).chain());
    
    let player_entity = world.spawn((
        Position { x: 0.0, y: 0.0 },
        Velocity { vx: 1.0, vy: 0.0 },
        Sprite {
            texture_handle: Handle::from_path("player.png"),
            width: 64.0,
            height: 64.0,
        },
    )).id();
    
    let time = Time::new();
    world.insert_resource(time);
    
    let graphics = GraphicsContext::new();
    world.insert_resource(graphics);
    
    loop {
        world.resource_mut::<Time>().update();
        schedule.run(&mut world);
    }
}
```

---

## 2. 插件模式（Plugin System）

### 模式概述

插件模式允许将引擎功能模块化，通过统一的接口注册和组合不同功能模块。每个插件独立开发和编译，引擎构建时静态链接。

### 核心优势

1. **模块化设计**：功能独立封装，便于开发和测试
2. **可组合性**：根据需求选择所需插件，构建定制化引擎
3. **依赖管理**：清晰的插件依赖关系声明
4. **生命周期管理**：统一的插件初始化和清理流程

### 代码示例

#### 插件 Trait 定义

```rust
use crate::world::World;
use crate::schedule::Schedule;

/// 插件 trait，所有引擎插件必须实现此 trait
pub trait Plugin: Send + Sync + 'static {
    /// 插件名称，用于调试和依赖解析
    fn name(&self) -> &'static str;
    
    /// 插件依赖列表，返回依赖的插件名称
    fn dependencies(&self) -> Vec<&'static str> {
        Vec::new()
    }
    
    /// 构建插件，向 World 和 Schedule 注册组件、系统和资源
    fn build(&self, app: &mut App);
}

/// 应用构建器，用于组合插件并构建引擎
pub struct App {
    world: World,
    schedule: Schedule,
    plugins: Vec<Box<dyn Plugin>>,
}

impl App {
    /// 创建新的应用构建器
    pub fn new() -> Self {
        App {
            world: World::new(),
            schedule: Schedule::default(),
            plugins: Vec::new(),
        }
    }
    
    /// 添加插件
    pub fn add_plugin<P: Plugin>(&mut self, plugin: P) -> &mut Self {
        self.plugins.push(Box::new(plugin));
        self
    }
    
    /// 注册组件类型
    pub fn register_component<C: Component>(&mut self) -> &mut Self {
        self.world.register_component::<C>();
        self
    }
    
    /// 添加系统到更新阶段
    pub fn add_systems<M>(&mut self, systems: impl IntoSystemConfigs<M>) -> &mut Self {
        self.schedule.add_systems(systems);
        self
    }
    
    /// 插入资源
    pub fn insert_resource<R: Resource>(&mut self, resource: R) -> &mut Self {
        self.world.insert_resource(resource);
        self
    }
    
    /// 构建并运行应用
    pub fn run(&mut self) {
        for plugin in &self.plugins {
            plugin.build(self);
        }
        
        let mut world = std::mem::take(&mut self.world);
        let mut schedule = std::mem::take(&mut self.schedule);
        
        loop {
            schedule.run(&mut world);
        }
    }
}
```

#### 具体插件实现

```rust
use crate::prelude::*;

/// 渲染插件，提供2D渲染功能
pub struct RenderPlugin;

impl Plugin for RenderPlugin {
    fn name(&self) -> &'static str {
        "render"
    }
    
    fn dependencies(&self) -> Vec<&'static str> {
        vec!["core"]
    }
    
    fn build(&self, app: &mut App) {
        app.register_component::<Sprite>()
           .register_component::<Camera>()
           .insert_resource(RenderConfig::default())
           .add_systems((
               sprite_render_system,
               camera_update_system,
           ).chain());
    }
}

/// 音频插件，提供音频播放功能
pub struct AudioPlugin;

impl Plugin for AudioPlugin {
    fn name(&self) -> &'static str {
        "audio"
    }
    
    fn build(&self, app: &mut App) {
        app.register_component::<AudioSource>()
           .insert_resource(AudioContext::new())
           .add_systems(audio_play_system);
    }
}
```

#### 引擎构建示例

```rust
use gwg_engine::prelude::*;
use galgame_plugin::GalgamePlugin;

fn main() {
    App::new()
        .add_plugin(CorePlugin)
        .add_plugin(RenderPlugin)
        .add_plugin(AudioPlugin)
        .add_plugin(UIPlugin)
        .add_plugin(GalgamePlugin)
        .insert_resource(WindowConfig {
            title: "我的 Galgame".to_string(),
            width: 1280,
            height: 720,
        })
        .run();
}
```

---

## 3. 平台抽象模式（Platform Abstraction）

### 模式概述

平台抽象模式通过定义统一的 trait 接口，屏蔽不同操作系统和运行环境的差异。上层代码只需依赖抽象接口，具体实现由各平台提供。

### 核心优势

1. **跨平台兼容**：一套代码支持多个平台
2. **易于扩展**：新增平台只需实现对应 trait
3. **测试友好**：可轻松 mock 平台实现进行单元测试
4. **关注点分离**：平台特定代码与业务逻辑分离

### 代码示例

#### 文件系统抽象

```rust
use std::path::Path;
use async_trait::async_trait;
use anyhow::Result;

/// 文件类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileType {
    /// 普通文件
    File,
    /// 目录
    Directory,
    /// 符号链接
    Symlink,
}

/// 文件元数据
#[derive(Debug, Clone)]
pub struct Metadata {
    /// 文件类型
    pub file_type: FileType,
    /// 文件大小（字节）
    pub len: u64,
    /// 最后修改时间
    pub modified: Option<std::time::SystemTime>,
}

/// 文件系统抽象 trait
#[async_trait]
pub trait FileSystem: Send + Sync + 'static {
    /// 检查文件是否存在
    async fn exists(&self, path: &Path) -> bool;
    
    /// 读取文件内容
    async fn read(&self, path: &Path) -> Result<Vec<u8>>;
    
    /// 读取文本文件
    async fn read_to_string(&self, path: &Path) -> Result<String> {
        let bytes = self.read(path).await?;
        Ok(String::from_utf8(bytes)?)
    }
    
    /// 写入文件内容
    async fn write(&self, path: &Path, content: &[u8]) -> Result<()>;
    
    /// 创建目录
    async fn create_dir(&self, path: &Path) -> Result<()>;
    
    /// 递归创建目录
    async fn create_dir_all(&self, path: &Path) -> Result<()>;
    
    /// 读取目录内容
    async fn read_dir(&self, path: &Path) -> Result<Vec<String>>;
    
    /// 获取文件元数据
    async fn metadata(&self, path: &Path) -> Result<Metadata>;
    
    /// 删除文件
    async fn remove_file(&self, path: &Path) -> Result<()>;
    
    /// 删除目录
    async fn remove_dir(&self, path: &Path) -> Result<()>;
}
```

#### 桌面平台实现

```rust
use std::path::Path;
use async_trait::async_trait;
use tokio::fs;
use anyhow::Result;
use super::{FileSystem, FileType, Metadata};

/// 桌面平台文件系统实现
pub struct DesktopFileSystem;

#[async_trait]
impl FileSystem for DesktopFileSystem {
    async fn exists(&self, path: &Path) -> bool {
        fs::try_exists(path).await.unwrap_or(false)
    }
    
    async fn read(&self, path: &Path) -> Result<Vec<u8>> {
        Ok(fs::read(path).await?)
    }
    
    async fn write(&self, path: &Path, content: &[u8]) -> Result<()> {
        fs::write(path, content).await?;
        Ok(())
    }
    
    async fn create_dir(&self, path: &Path) -> Result<()> {
        fs::create_dir(path).await?;
        Ok(())
    }
    
    async fn create_dir_all(&self, path: &Path) -> Result<()> {
        fs::create_dir_all(path).await?;
        Ok(())
    }
    
    async fn read_dir(&self, path: &Path) -> Result<Vec<String>> {
        let mut entries = Vec::new();
        let mut read_dir = fs::read_dir(path).await?;
        while let Some(entry) = read_dir.next_entry().await? {
            if let Some(name) = entry.file_name().to_str() {
                entries.push(name.to_string());
            }
        }
        Ok(entries)
    }
    
    async fn metadata(&self, path: &Path) -> Result<Metadata> {
        let meta = fs::metadata(path).await?;
        let file_type = if meta.is_dir() {
            FileType::Directory
        } else if meta.is_file() {
            FileType::File
        } else {
            FileType::Symlink
        };
        Ok(Metadata {
            file_type,
            len: meta.len(),
            modified: meta.modified().ok(),
        })
    }
    
    async fn remove_file(&self, path: &Path) -> Result<()> {
        fs::remove_file(path).await?;
        Ok(())
    }
    
    async fn remove_dir(&self, path: &Path) -> Result<()> {
        fs::remove_dir_all(path).await?;
        Ok(())
    }
}
```

#### Web 平台实现

```rust
use std::path::Path;
use async_trait::async_trait;
use anyhow::{anyhow, Result};
use wasm_bindgen::prelude::*;
use web_sys::window;
use super::{FileSystem, FileType, Metadata};

/// Web 平台文件系统实现
pub struct WebFileSystem {
    base_url: String,
}

impl WebFileSystem {
    /// 创建新的 Web 文件系统
    pub fn new(base_url: String) -> Self {
        WebFileSystem { base_url }
    }
}

#[async_trait]
impl FileSystem for WebFileSystem {
    async fn exists(&self, path: &Path) -> bool {
        let url = format!("{}/{}", self.base_url, path.to_string_lossy());
        let window = match window() {
            Some(w) => w,
            None => return false,
        };
        let fetch = window.fetch_with_str(&url);
        let promise = js_sys::Promise::resolve(&fetch);
        let result = wasm_bindgen_futures::JsFuture::from(promise).await;
        result.is_ok()
    }
    
    async fn read(&self, path: &Path) -> Result<Vec<u8>> {
        let url = format!("{}/{}", self.base_url, path.to_string_lossy());
        let window = window().ok_or_else(|| anyhow!("No window"))?;
        let resp_value = wasm_bindgen_futures::JsFuture::from(window.fetch_with_str(&url))
            .await?;
        let resp: web_sys::Response = resp_value.dyn_into()?;
        let array_buffer = wasm_bindgen_futures::JsFuture::from(resp.array_buffer()?).await?;
        let uint8_array = js_sys::Uint8Array::new(&array_buffer);
        Ok(uint8_array.to_vec())
    }
    
    async fn write(&self, _path: &Path, _content: &[u8]) -> Result<()> {
        Err(anyhow!("Write not supported in Web environment"))
    }
    
    async fn create_dir(&self, _path: &Path) -> Result<()> {
        Err(anyhow!("Create dir not supported in Web environment"))
    }
    
    async fn create_dir_all(&self, _path: &Path) -> Result<()> {
        Err(anyhow!("Create dir all not supported in Web environment"))
    }
    
    async fn read_dir(&self, _path: &Path) -> Result<Vec<String>> {
        Err(anyhow!("Read dir not supported in Web environment"))
    }
    
    async fn metadata(&self, _path: &Path) -> Result<Metadata> {
        Err(anyhow!("Metadata not supported in Web environment"))
    }
    
    async fn remove_file(&self, _path: &Path) -> Result<()> {
        Err(anyhow!("Remove file not supported in Web environment"))
    }
    
    async fn remove_dir(&self, _path: &Path) -> Result<()> {
        Err(anyhow!("Remove dir not supported in Web environment"))
    }
}
```

#### 平台选择器

```rust
use std::path::PathBuf;
use super::{FileSystem, DesktopFileSystem, WebFileSystem};

/// 获取当前平台的文件系统实现
#[cfg(not(target_arch = "wasm32"))]
pub fn get_file_system() -> Box<dyn FileSystem> {
    Box::new(DesktopFileSystem)
}

/// 获取 Web 平台的文件系统实现
#[cfg(target_arch = "wasm32")]
pub fn get_file_system() -> Box<dyn FileSystem> {
    let base_url = "/assets".to_string();
    Box::new(WebFileSystem::new(base_url))
}

/// 资源加载器，使用平台抽象的文件系统
pub struct AssetLoader {
    fs: Box<dyn FileSystem>,
    base_path: PathBuf,
}

impl AssetLoader {
    /// 创建新的资源加载器
    pub fn new(base_path: PathBuf) -> Self {
        AssetLoader {
            fs: get_file_system(),
            base_path,
        }
    }
    
    /// 加载资源
    pub async fn load_asset(&self, path: &str) -> anyhow::Result<Vec<u8>> {
        let full_path = self.base_path.join(path);
        self.fs.read(&full_path).await
    }
}
```

---

## 4. 数据驱动设计（Data-Driven Design）

### 模式概述

数据驱动设计将游戏逻辑与数据分离，通过配置文件定义游戏内容，而非硬编码。引擎负责解释和执行数据，创作者只需编辑数据即可创作游戏。

### 核心优势

1. **无需编程**：普通玩家通过配置文件创作游戏
2. **快速迭代**：修改数据无需重新编译
3. **热更新支持**：运行时动态更新游戏内容
4. **内容与引擎分离**：同一引擎可运行多款不同游戏

### 代码示例

#### 游戏配置定义

```rust
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 游戏配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameConfig {
    /// 游戏标题
    pub title: String,
    /// 游戏版本
    pub version: String,
    /// 作者
    pub author: String,
    /// 窗口配置
    pub window: WindowConfig,
    /// 默认字体
    pub default_font: String,
    /// 初始场景
    pub initial_scene: String,
}

/// 窗口配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindowConfig {
    /// 窗口宽度
    pub width: u32,
    /// 窗口高度
    pub height: u32,
    /// 是否全屏
    pub fullscreen: bool,
    /// 窗口标题
    pub title: String,
}

/// 场景配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SceneConfig {
    /// 场景名称
    pub name: String,
    /// 背景图片
    pub background: String,
    /// BGM
    pub bgm: Option<String>,
    /// 角色列表
    pub characters: Vec<CharacterConfig>,
    /// 对话列表
    pub dialogues: Vec<DialogueConfig>,
}

/// 角色配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CharacterConfig {
    /// 角色ID
    pub id: String,
    /// 角色名称
    pub name: String,
    /// 默认立绘
    pub default_sprite: String,
    /// 立绘变体
    pub sprites: HashMap<String, String>,
    /// 初始位置
    pub position: (f32, f32),
}

/// 对话配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DialogueConfig {
    /// 对话ID
    pub id: String,
    /// 说话角色ID（None 表示旁白）
    pub speaker: Option<String>,
    /// 对话文本
    pub text: String,
    /// 角色立绘变化
    pub sprite_change: Option<(String, String)>,
    /// 下一个对话ID
    pub next: Option<String>,
    /// 选项列表
    pub choices: Vec<ChoiceConfig>,
}

/// 选项配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChoiceConfig {
    /// 选项文本
    pub text: String,
    /// 选择后跳转的对话ID
    pub next: String,
    /// 显示条件
    pub condition: Option<String>,
}

impl Default for WindowConfig {
    fn default() -> Self {
        WindowConfig {
            width: 1280,
            height: 720,
            fullscreen: false,
            title: "GWG Game".to_string(),
        }
    }
}

impl Default for GameConfig {
    fn default() -> Self {
        GameConfig {
            title: "Untitled Game".to_string(),
            version: "0.1.0".to_string(),
            author: "Unknown".to_string(),
            window: WindowConfig::default(),
            default_font: "default.ttf".to_string(),
            initial_scene: "start".to_string(),
        }
    }
}
```

#### 数据加载器

```rust
use crate::config::*;
use crate::filesystem::FileSystem;
use std::path::Path;
use anyhow::Result;

/// 游戏数据加载器
pub struct GameDataLoader {
    fs: Box<dyn FileSystem>,
}

impl GameDataLoader {
    /// 创建新的数据加载器
    pub fn new(fs: Box<dyn FileSystem>) -> Self {
        GameDataLoader { fs }
    }
    
    /// 加载游戏配置
    pub async fn load_game_config(&self, path: &Path) -> Result<GameConfig> {
        let content = self.fs.read_to_string(path).await?;
        let config: GameConfig = serde_json::from_str(&content)?;
        Ok(config)
    }
    
    /// 加载场景配置
    pub async fn load_scene_config(&self, path: &Path) -> Result<SceneConfig> {
        let content = self.fs.read_to_string(path).await?;
        let config: SceneConfig = serde_json::from_str(&content)?;
        Ok(config)
    }
    
    /// 从目录加载所有场景
    pub async fn load_all_scenes(&self, dir: &Path) -> Result<HashMap<String, SceneConfig>> {
        let mut scenes = HashMap::new();
        let entries = self.fs.read_dir(dir).await?;
        
        for entry in entries {
            if entry.ends_with(".json") {
                let path = dir.join(entry);
                let scene = self.load_scene_config(&path).await?;
                scenes.insert(scene.name.clone(), scene);
            }
        }
        
        Ok(scenes)
    }
}
```

#### 游戏配置示例（JSON）

```json
{
    "title": "我的 Galgame",
    "version": "1.0.0",
    "author": "灵之镜工作室",
    "window": {
        "width": 1280,
        "height": 720,
        "fullscreen": false,
        "title": "我的 Galgame"
    },
    "default_font": "fonts/msyh.ttf",
    "initial_scene": "prologue"
}
```

#### 场景配置示例（JSON）

```json
{
    "name": "prologue",
    "background": "images/bg/school.jpg",
    "bgm": "audio/bgm/school.mp3",
    "characters": [
        {
            "id": "heroine",
            "name": "女主角",
            "default_sprite": "images/char/heroine_normal.png",
            "sprites": {
                "normal": "images/char/heroine_normal.png",
                "smile": "images/char/heroine_smile.png",
                "surprise": "images/char/heroine_surprise.png"
            },
            "position": [640, 360]
        }
    ],
    "dialogues": [
        {
            "id": "d1",
            "speaker": null,
            "text": "这是一个晴朗的早晨...",
            "next": "d2"
        },
        {
            "id": "d2",
            "speaker": "heroine",
            "text": "早上好！",
            "sprite_change": ["heroine", "smile"],
            "next": "d3"
        },
        {
            "id": "d3",
            "speaker": null,
            "text": "你要怎么回应？",
            "choices": [
                {
                    "text": "早上好！",
                    "next": "d4"
                },
                {
                    "text": "你是谁？",
                    "next": "d5"
                }
            ]
        }
    ]
}
```

---

## 5. 沙箱模式（Sandbox Pattern）

### 模式概述

沙箱模式为虚拟机脚本提供安全的执行环境，隔离脚本与宿主系统，通过受限的 API 暴露必要功能。脚本只能访问允许的资源和功能，无法直接操作内存或文件系统。

### 核心优势

1. **安全性**：防止恶意脚本破坏系统
2. **可控性**：精确控制脚本可访问的 API
3. **稳定性**：脚本崩溃不影响引擎主进程
4. **跨平台**：WASM 沙箱天然跨平台

### 代码示例

#### 虚拟机核心接口

```rust
use anyhow::{anyhow, Result};
use wasmtime::*;
use std::sync::{Arc, Mutex};

/// 虚拟机上下文
pub struct VmContext {
    /// ECS 世界访问器
    world: Arc<Mutex<WorldAccessor>>,
    /// 资源管理器
    asset_manager: Arc<Mutex<AssetManager>>,
    /// 事件队列
    event_queue: Arc<Mutex<EventQueue>>,
}

impl VmContext {
    /// 创建新的虚拟机上下文
    pub fn new(
        world: WorldAccessor,
        asset_manager: AssetManager,
        event_queue: EventQueue,
    ) -> Self {
        VmContext {
            world: Arc::new(Mutex::new(world)),
            asset_manager: Arc::new(Mutex::new(asset_manager)),
            event_queue: Arc::new(Mutex::new(event_queue)),
        }
    }
}

/// 虚拟机实例
pub struct VmInstance {
    /// WASM 模块实例
    instance: Instance,
    /// 内存
    memory: Memory,
    /// 上下文
    context: VmContext,
}

impl VmInstance {
    /// 创建新的虚拟机实例
    pub fn new(wasm_bytes: &[u8], context: VmContext) -> Result<Self> {
        let engine = Engine::default();
        let module = Module::from_binary(&engine, wasm_bytes)?;
        
        let mut linker = Linker::new(&engine);
        let mut store = Store::new(&engine, context.clone());
        
        Self::register_host_functions(&mut linker, &mut store)?;
        
        let instance = linker.instantiate(&mut store, &module)?;
        let memory = instance.get_memory(&mut store, "memory")
            .ok_or_else(|| anyhow!("No memory export"))?;
        
        Ok(VmInstance {
            instance,
            memory,
            context,
        })
    }
    
    /// 注册宿主函数
    fn register_host_functions(
        linker: &mut Linker<VmContext>,
        store: &mut Store<VmContext>,
    ) -> Result<()> {
        linker.func_wrap(
            "env",
            "print",
            |mut caller: Caller<'_, VmContext>, ptr: i32, len: i32| {
                let memory = caller.get_export("memory")
                    .and_then(|e| e.into_memory())
                    .ok_or_else(|| anyhow!("No memory"))?;
                
                let data = memory.data(&caller);
                let text = std::str::from_utf8(&data[ptr as usize..(ptr + len) as usize])?;
                println!("[Script] {}", text);
                Ok(())
            },
        )?;
        
        linker.func_wrap(
            "env",
            "create_entity",
            |mut caller: Caller<'_, VmContext>| -> Result<i32> {
                let ctx = caller.data_mut();
                let mut world = ctx.world.lock().unwrap();
                let entity = world.spawn_empty().id();
                Ok(entity.index() as i32)
            },
        )?;
        
        Ok(())
    }
    
    /// 调用脚本初始化函数
    pub fn init(&mut self) -> Result<()> {
        let init = self.instance.get_typed_func::<(), ()>(&mut self.store, "init")?;
        init.call(&mut self.store, ())?;
        Ok(())
    }
    
    /// 调用脚本更新函数
    pub fn update(&mut self, delta_time: f32) -> Result<()> {
        let update = self.instance.get_typed_func::<f32, ()>(&mut self.store, "update")?;
        update.call(&mut self.store, delta_time)?;
        Ok(())
    }
}
```

#### ECS API 封装

```rust
use bevy_ecs::prelude::*;
use std::sync::{Arc, Mutex};

/// 世界访问器，提供受控的 ECS 访问
pub struct WorldAccessor {
    world: Arc<Mutex<World>>,
    allowed_components: Vec<String>,
}

impl WorldAccessor {
    /// 创建新的世界访问器
    pub fn new(world: World) -> Self {
        WorldAccessor {
            world: Arc::new(Mutex::new(world)),
            allowed_components: vec![
                "Position".to_string(),
                "Velocity".to_string(),
            ],
        }
    }
    
    /// 创建空实体
    pub fn spawn_empty(&mut self) -> EntityMut {
        let mut world = self.world.lock().unwrap();
        world.spawn_empty()
    }
    
    /// 添加组件（受限）
    pub fn add_component<C: Component>(&mut self, entity: Entity, component: C) -> Result<()> {
        let component_name = std::any::type_name::<C>();
        if !self.allowed_components.iter().any(|c| component_name.contains(c)) {
            return Err(anyhow!("Component {} not allowed", component_name));
        }
        
        let mut world = self.world.lock().unwrap();
        let mut entity = world.entity_mut(entity);
        entity.insert(component);
        Ok(())
    }
    
    /// 获取组件（受限）
    pub fn get_component<C: Component>(&self, entity: Entity) -> Result<Option<&C>> {
        let component_name = std::any::type_name::<C>();
        if !self.allowed_components.iter().any(|c| component_name.contains(c)) {
            return Err(anyhow!("Component {} not allowed", component_name));
        }
        
        let world = self.world.lock().unwrap();
        Ok(world.entity(entity).get::<C>())
    }
}
```

#### 脚本示例（WAT 格式）

```wat
(module
    (import "env" "print" (func $print (param i32 i32)))
    (import "env" "create_entity" (func $create_entity (result i32)))
    
    (memory 1)
    
    (data (i32.const 0) "Hello from script!")
    
    (func (export "init")
        (call $print (i32.const 0) (i32.const 18))
        (call $create_entity)
        drop
    )
    
    (func (export "update") (param $delta f32)
    )
)
```

#### 虚拟机管理器

```rust
use crate::vm::*;
use std::collections::HashMap;
use std::path::Path;

/// 虚拟机管理器
pub struct VmManager {
    /// 已加载的脚本
    scripts: HashMap<String, VmInstance>,
    /// 上下文
    context: VmContext,
}

impl VmManager {
    /// 创建新的虚拟机管理器
    pub fn new(context: VmContext) -> Self {
        VmManager {
            scripts: HashMap::new(),
            context,
        }
    }
    
    /// 加载脚本
    pub async fn load_script(&mut self, name: &str, path: &Path, fs: &dyn FileSystem) -> Result<()> {
        let wasm_bytes = fs.read(path).await?;
        let instance = VmInstance::new(&wasm_bytes, self.context.clone())?;
        self.scripts.insert(name.to_string(), instance);
        Ok(())
    }
    
    /// 初始化所有脚本
    pub fn init_all(&mut self) -> Result<()> {
        for (name, instance) in &mut self.scripts {
            if let Err(e) = instance.init() {
                eprintln!("Script {} init failed: {}", name, e);
            }
        }
        Ok(())
    }
    
    /// 更新所有脚本
    pub fn update_all(&mut self, delta_time: f32) -> Result<()> {
        for (name, instance) in &mut self.scripts {
            if let Err(e) = instance.update(delta_time) {
                eprintln!("Script {} update failed: {}", name, e);
            }
        }
        Ok(())
    }
}
```

---

## 模式组合应用

在诺斯替神话框架下，这些设计模式在 GWG 引擎中相互配合，共同构建完整的游戏创造宇宙：

### 神话视角下的模式协作

1. **太一的流溢（ECS + 插件模式）**  
   插件作为流溢，向 ECS 世界（太一的基始）注册组件和系统，德穆革（具体引擎）由此诞生。

2. **遍在性的显现（平台抽象 + 数据驱动）**  
   数据驱动的资源加载通过平台抽象实现，使普累若麻（游戏项目）能在不同物质载体（平台）上显现。

3. **异端知识的边界（沙箱 + ECS）**  
   脚本（异端知识）通过沙箱 API 安全地访问 ECS 世界，既允许灵性干预，又维持物质世界的稳定。

4. **世界的创造（插件 + 平台抽象）**  
   插件可通过平台抽象提供跨平台功能，德穆革由此能在不同物质载体上创造世界。

### 技术视角下的模式协作

1. **ECS + 插件模式**：插件向 ECS 世界注册组件和系统
2. **平台抽象 + 数据驱动**：数据驱动的资源加载通过平台抽象实现
3. **沙箱 + ECS**：脚本通过沙箱 API 安全地访问 ECS 世界
4. **插件 + 平台抽象**：插件可通过平台抽象提供跨平台功能

通过这些模式的组合，GWG 引擎实现了高性能、可扩展、跨平台的元游戏引擎架构，同时在神话层面展现出深层的象征意义：太一作为超越源头，德穆革作为有限造物主，执权者作为世界管理者，玩家作为灵性寻求者，共同构成一个完整的游戏创造宇宙。
