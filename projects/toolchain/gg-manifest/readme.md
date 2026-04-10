# gg-manifest

**GG Game Engine 的项目配置管理，负责管理项目的配置和元数据。**

## 📋 模块简介

gg-manifest 是 GG Game Engine 的项目配置管理模块，负责管理项目的配置和元数据，为项目提供统一的配置管理功能。

## ✨ 核心功能

- **配置管理**：管理项目的配置文件
- **元数据管理**：管理项目的元数据
- **模板系统**：支持配置模板
- **序列化支持**：支持配置的序列化和反序列化
- **验证功能**：验证配置的完整性和有效性
- **版本管理**：管理项目版本信息

## 🚀 使用方法

### 依赖添加

```toml
[dependencies]
gg-manifest = { path = "projects/toolchain/gg-manifest" }
```

### 基础示例

```rust
use gg_manifest::prelude::*;

fn main() {
    // 加载项目配置
    let manifest = Manifest::load("game.toml").unwrap();
    
    // 访问配置
    println!("Game title: {}", manifest.title);
    println!("Game version: {}", manifest.version);
    
    // 修改配置
    let mut manifest = manifest;
    manifest.title = "My Awesome Game".to_string();
    
    // 保存配置
    manifest.save("game.toml").unwrap();
    
    println!("Manifest updated successfully!");
}
```

## 📦 依赖关系

- **gg-core**：核心功能和平台抽象
- **gg-error**：错误处理系统

## 📖 相关文档

- [工具链设计](../../../design/architecture/overview.md) - 了解工具链系统的设计理念