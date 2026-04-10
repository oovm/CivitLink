# gg-asset

**GG Game Engine 的资源管理系统，负责资源的加载、缓存和管理。**

## 📋 模块简介

gg-asset 是 GG Game Engine 的资源管理系统，负责游戏资源（如纹理、音频、模型等）的加载、缓存和管理，提供高效的资源访问和生命周期管理。

## ✨ 核心功能

- **资源加载**：支持异步加载各种类型的资源
- **资源缓存**：智能缓存机制，避免重复加载
- **资源依赖**：处理资源之间的依赖关系
- **资源生命周期**：自动管理资源的生命周期
- **资源热重载**：支持开发过程中的资源热重载

## 🚀 使用方法

### 依赖添加

```toml
[dependencies]
gg-asset = { path = "projects/core/gg-asset" }
```

### 基础示例

```rust
use gg_asset::prelude::*;

fn main() {
    // 创建资源管理器
    let asset_manager = AssetManager::new();
    
    // 加载纹理资源
    let texture_handle = asset_manager.load::<Texture>("assets/textures/player.png");
    
    // 获取资源
    let texture = asset_manager.get(texture_handle).unwrap();
    println!("Texture size: {}x{}", texture.width, texture.height);
    
    // 卸载资源
    asset_manager.unload(texture_handle);
}
```

## 📦 依赖关系

- **gg-core**：核心功能和平台抽象
- **gg-error**：错误处理系统

## 📖 相关文档

- [资源管理设计](../../../design/modules/asset.md) - 了解资源管理系统的设计理念