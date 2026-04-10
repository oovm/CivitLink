# gg-galgame-schema

**GG Game Engine 的 GAL 游戏数据 schema，定义 GAL 游戏的数据结构和格式。**

## 📋 模块简介

gg-galgame-schema 是 GG Game Engine 的 GAL 游戏数据 schema，定义 GAL 游戏的数据结构和格式，为 GAL 游戏的开发和管理提供标准化的数据模型。

## ✨ 核心功能

- **数据结构定义**：定义 GAL 游戏的各种数据结构
- **组件系统**：提供 GAL 游戏的组件定义
- **资源管理**：定义 GAL 游戏的资源结构
- **Manifest 管理**：管理 GAL 游戏的配置和元数据
- **序列化支持**：支持数据的序列化和反序列化
- **验证功能**：验证数据的完整性和有效性

## 🚀 使用方法

### 依赖添加

```toml
[dependencies]
gg-galgame-schema = { path = "projects/plugins/gg-galgame-schema" }
```

### 基础示例

```rust
use gg_galgame_schema::prelude::*;

fn main() {
    // 创建游戏配置
    let game_config = GameManifest::new()
        .title("My First GAL Game")
        .author("Game Developer")
        .version("1.0.0")
        .build();
    
    // 序列化配置
    let json = game_config.to_json().unwrap();
    println!("Game config: {}", json);
    
    // 反序列化配置
    let loaded_config = GameManifest::from_json(&json).unwrap();
    println!("Loaded game title: {}", loaded_config.title);
}
```

## 📦 依赖关系

- **gg-core**：核心功能和平台抽象
- **gg-error**：错误处理系统

## 📖 相关文档

- [插件系统设计](../../../design/architecture/overview.md) - 了解插件系统的设计理念