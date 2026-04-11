# gg-plugin-save

**GG Game Engine 的存档系统插件，负责管理游戏的存档和读档功能。**

## 📋 模块简介

gg-plugin-save 是 GG Game Engine 的存档系统插件，负责管理游戏的存档和读档功能，提供游戏进度的保存和加载。

## ✨ 核心功能

- **存档管理**：创建、读取和删除存档
- **自动存档**：支持自动存档功能
- **存档槽管理**：管理多个存档槽
- **存档数据**：保存和加载游戏状态数据
- **存档验证**：验证存档的完整性和有效性
- **跨平台支持**：支持不同平台的存档管理

## 🚀 使用方法

### 依赖添加

```toml
[dependencies]
gg-plugin-save = { path = "projects/plugins/gg-plugin-save" }
```

### 基础示例

```rust
use gg_plugin_save::prelude::*;

fn main() {
    // 创建存档管理器
    let mut save_manager = SaveManager::new();
    
    // 保存游戏
    let save_data = SaveData::new()
        .set("player_health", 100)
        .set("player_position", (100.0, 200.0))
        .set("current_level", 5)
        .build();
    
    // 保存到存档槽 1
    save_manager.save(1, save_data).unwrap();
    
    // 加载存档
    let loaded_data = save_manager.load(1).unwrap();
    let player_health = loaded_data.get("player_health").unwrap();
    println!("Player health: {:?}", player_health);
}
```

## 📦 依赖关系

- **gg-core**：核心功能和平台抽象
- **gg-world**：游戏世界管理
- **gg-error**：错误处理系统

## 📖 相关文档

- [插件系统设计](../../../design/architecture/overview.md) - 了解插件系统的设计理念