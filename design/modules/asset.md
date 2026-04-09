# Asset 模块

## 概述

Asset 模块提供异步资源加载、缓存和句柄系统，是 GG 元引擎的资源管理核心。

## 核心概念

### 资源（Asset）
资源是游戏中使用的数据，例如图片、音频、模型、剧本等。

```rust
use gg_asset::prelude::*;

struct Texture {
    data: Vec<u8>,
    width: u32,
    height: u32,
}

impl Asset for Texture {
    fn type_name() -> &'static str {
        "Texture"
    }
}
```

### 句柄（Handle）
句柄是对资源的安全引用，用于避免悬空指针。

```rust
pub struct Handle<T> {
    id: u64,
    path: Arc<str>,
    _marker: PhantomData<T>,
}
```

### 资源加载器（AssetLoader）
负责从磁盘加载特定类型的资源。

```rust
trait AssetLoader<T: Asset>: Send + Sync + 'static {
    async fn load(&self, path: &Path) -> Result<T, AssetError>;
}
```

## 核心类型

### AssetServer
资源服务器，负责资源的加载和管理。

```rust
pub struct AssetServer {
    cache: AssetCache,
}
```

**主要方法：**

- `new()` - 创建新的资源服务器
- `add_asset(path, asset)` - 直接将资源添加到缓存
- `cache()` - 获取资源缓存的引用

### AssetCache
资源缓存，存储已加载的资源。

```rust
pub struct AssetCache {
    assets: DashMap<u64, Arc<dyn Any + Send + Sync>>,
    handles: DashMap<Arc<str>, u64>,
    next_id: RwLock<u64>,
}
```

**主要方法：**

- `insert(path, asset)` - 插入资源到缓存
- `get(handle)` - 根据句柄获取资源
- `get_handle(path)` - 根据路径获取资源句柄
- `contains(path)` - 检查资源是否已加载
- `remove(path)` - 从缓存中移除资源
- `clear()` - 清空缓存

### Handle<T>
资源句柄，类型安全的资源引用。

**主要方法：**

- `new(id, path)` - 创建新的资源句柄
- `id()` - 获取资源 ID
- `path()` - 获取资源路径

### AssetError
资源加载错误枚举。

```rust
pub enum AssetError {
    NotFound(String),
    LoadError(String),
    TypeMismatch,
}
```

## 使用示例

### 基础用法

```rust
use gg_asset::prelude::*;
use std::sync::Arc;

#[derive(Debug)]
struct Image {
    data: Vec<u8>,
}

impl Asset for Image {
    fn type_name() -> &'static str {
        "Image"
    }
}

#[tokio::main]
async fn main() {
    let asset_server = AssetServer::new();
    
    let image = Image {
        data: vec![0, 1, 2, 3],
    };
    
    let handle = asset_server.add_asset("textures/player.png", image).await;
    
    if let Some(loaded_image) = asset_server.cache().get(&handle) {
        println!("Loaded image: {:?}", loaded_image);
    }
}
```

### 资源缓存管理

```rust
let cache = AssetCache::new();

// 添加资源
let handle = cache.insert("audio/bgm.mp3", Audio::new()).await;

// 检查资源是否存在
if cache.contains("audio/bgm.mp3") {
    // 获取资源
    if let Some(audio) = cache.get(&handle) {
        // 使用资源
    }
}

// 移除资源
cache.remove("audio/bgm.mp3");

// 清空所有资源
cache.clear();
```

## 相关模块

- [world](./world.md) - 世界管理
- [ecs](./ecs.md) - ECS 核心
- [schedule](./schedule.md) - 调度器
