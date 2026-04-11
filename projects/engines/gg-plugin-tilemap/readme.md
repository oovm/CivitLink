# gg-plugin-tilemap

**GG Game Engine 的瓦片地图插件，负责管理和渲染瓦片地图。**

## 📋 模块简介

gg-plugin-tilemap 是 GG Game Engine 的瓦片地图插件，负责管理和渲染瓦片地图，为游戏提供 2D 地图的创建和管理功能。

## ✨ 核心功能

- **瓦片地图管理**：创建和管理瓦片地图
- **瓦片集支持**：支持加载和使用瓦片集
- **图层管理**：管理地图的多个图层
- **碰撞检测**：支持瓦片地图的碰撞检测
- **地图编辑**：支持地图的编辑和修改
- **性能优化**：优化瓦片地图的渲染性能

## 🚀 使用方法

### 依赖添加

```toml
[dependencies]
gg-plugin-tilemap = { path = "projects/plugins/gg-plugin-tilemap" }
```

### 基础示例

```rust
use gg_plugin_tilemap::prelude::*;

fn main() {
    // 创建瓦片地图系统
    let mut tilemap_system = TilemapSystem::new();
    
    // 加载瓦片地图
    let tilemap = Tilemap::new()
        .tileset("assets/tilesets/terrain.png")
        .size((10, 10))
        .tile_size((32, 32))
        .build();
    
    // 设置瓦片
    tilemap.set_tile((0, 0), 1);
    tilemap.set_tile((1, 0), 2);
    tilemap.set_tile((0, 1), 3);
    tilemap.set_tile((1, 1), 4);
    
    // 添加到系统
    tilemap_system.add_tilemap(tilemap);
    
    // 更新系统
    tilemap_system.update();
}
```

## 📦 依赖关系

- **gg-core**：核心功能和平台抽象
- **gg-world**：游戏世界管理
- **gg-render**：渲染系统
- **gg-error**：错误处理系统

## 📖 相关文档

- [插件系统设计](../../../design/architecture/overview.md) - 了解插件系统的设计理念