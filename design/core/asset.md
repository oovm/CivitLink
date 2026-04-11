# Asset 模块

## 概述

Asset 模块提供统一的资源加载、管理和热更新功能，支持编辑器UI和游戏UI的资源分离。

## 核心概念

### 资源（Asset）
资源是游戏中使用的数据，例如图片、音频、模型、剧本等。

```rust
use gg_asset::prelude::*;

#[derive(Debug, Clone)]
struct TextureAsset {
    data: Vec<u8>,
    width: u32,
    height: u32,
}

impl Asset for TextureAsset {
    fn asset_type() -> AssetType {
        AssetType::Texture
    }

    fn load(path: &Path) -> GResult<Self> {
        let data = std::fs::read(path)?;
        Ok(Self {
            data,
            width: 0,
            height: 0,
        })
    }

    fn unload(&mut self) {
        self.data.clear();
    }

    fn size(&self) -> usize {
        self.data.len()
    }
}
```

### 句柄（Handle）
句柄是对资源的安全引用，用于避免悬空指针。

```rust
pub struct Handle<T: Asset> {
    id: u64,
    _marker: std::marker::PhantomData<T>,
}
```

### 资源服务器（AssetServer）
负责资源的加载和管理。

## 核心类型

### AssetType

资源类型枚举，用于分类不同类型的资源。

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AssetType {
    Texture,
    Font,
    Material,
    Model,
    Audio,
    Script,
    Ui,
    EditorUi,
    Other,
}
```

### AssetState

资源状态枚举，表示资源的加载状态。

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AssetState {
    Unloaded,
    Loading,
    Loaded,
    Failed,
}
```

### AssetMeta

资源元数据，包含资源的路径、类型、大小等信息。

```rust
#[derive(Debug, Clone)]
pub struct AssetMeta {
    pub path: PathBuf,
    pub asset_type: AssetType,
    pub size: usize,
    pub last_modified: Instant,
    pub dependencies: HashSet<u64>,
}
```

### AssetServer

资源服务器，负责资源的加载、管理和热更新。

```rust
pub struct AssetServer {
    assets: HashMap<u64, AssetItem>,
    path_to_id: HashMap<PathBuf, u64>,
    next_id: u64,
    editor_ui_assets: HashSet<u64>,
    game_ui_assets: HashSet<u64>,
    hot_reload_enabled: bool,
    hot_reload_interval: Duration,
    last_hot_reload_check: Instant,
}
```

**主要方法：**

- `new()` - 创建新的资源服务器
- `load<T>(path, is_editor_ui)` - 加载指定路径的资源
- `get<T>(handle)` - 获取资源的不可变引用
- `get_mut<T>(handle)` - 获取资源的可变引用
- `release<T>(handle)` - 释放资源
- `enable_hot_reload(enabled)` - 启用或禁用热更新
- `check_hot_reload()` - 检查并执行热更新
- `cleanup_unused(max_age)` - 清理未使用的资源

### AssetSystem

资源系统，作为 ECS 系统运行，定期检查热更新。

```rust
pub struct AssetSystem {
    asset_server: Arc<RwLock<AssetServer>>,
}
```

**主要方法：**

- `new()` - 创建新的资源系统
- `asset_server()` - 获取资源服务器的引用

## 内置资源类型

### TextureAsset
纹理资源，用于存储图像数据。

### FontAsset
字体资源，用于存储字体数据。

### MaterialAsset
材质资源，用于存储材质数据。

### UiAsset
UI资源，用于存储游戏UI相关数据。

### EditorUiAsset
编辑器UI资源，用于存储编辑器UI相关数据。

## 使用示例

### 基础用法

```rust
use gg_asset::prelude::*;

fn main() {
    let mut asset_server = AssetServer::new();
    
    // 加载纹理资源
    let texture_handle = asset_server.load::<TextureAsset>(
        Path::new("textures/player.png"),
        false // 不是编辑器UI资源
    ).unwrap();
    
    // 获取资源
    if let Some(texture) = asset_server.get(&texture_handle) {
        println!("Texture size: {} bytes", texture.size());
    }
    
    // 释放资源
    asset_server.release(texture_handle);
}
```

### 热更新

```rust
use gg_asset::prelude::*;

fn main() {
    let mut asset_server = AssetServer::new();
    
    // 启用热更新
    asset_server.enable_hot_reload(true);
    
    // 加载资源
    let texture_handle = asset_server.load::<TextureAsset>(
        Path::new("textures/player.png"),
        false
    ).unwrap();
    
    // 定期检查热更新
    loop {
        let reloaded = asset_server.check_hot_reload().unwrap();
        if !reloaded.is_empty() {
            println!("Reloaded assets: {:?}", reloaded);
        }
        std::thread::sleep(std::time::Duration::from_secs(1));
    }
}
```

### 与 ECS 集成

```rust
use gg_ecs::prelude::*;
use gg_asset::prelude::*;

struct TextureComponent {
    handle: Handle<TextureAsset>,
}
impl Component for TextureComponent {}

struct RenderSystem {
    asset_server: Arc<RwLock<AssetServer>>,
}

impl System for RenderSystem {
    fn name(&self) -> &str {
        "RenderSystem"
    }

    fn execute(&mut self, world: &mut World) -> GResult<()> {
        let query = world.query::<TextureComponent>();
        let asset_server = self.asset_server.read().unwrap();
        
        for (entity, texture_comp) in query.iter() {
            if let Some(texture) = asset_server.get(&texture_comp.handle) {
                // 渲染纹理
                println!("Rendering texture with size: {}x{}", texture.width, texture.height);
            }
        }
        
        Ok(())
    }
}

fn main() {
    let mut world = World::new();
    let asset_server = Arc::new(RwLock::new(AssetServer::new()));
    
    // 加载纹理
    let texture_handle = asset_server.write().unwrap().load::<TextureAsset>(
        Path::new("textures/player.png"),
        false
    ).unwrap();
    
    // 创建实体并添加纹理组件
    world.spawn()
        .insert(TextureComponent { handle: texture_handle });
    
    // 注册渲染系统
    world.register_system(Box::new(RenderSystem {
        asset_server: asset_server.clone(),
    }));
    
    // 运行系统
    world.run_systems().unwrap();
}
```

## 相关模块

- [ecs](../core/ecs.md) - ECS 核心
- [world](../core/world.md) - 世界管理