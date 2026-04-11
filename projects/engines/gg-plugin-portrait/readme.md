# gg-plugin-portrait

**GG Game Engine 的角色立绘插件，负责管理游戏中的角色立绘和动画。**

## 📋 模块简介

gg-plugin-portrait 是 GG Game Engine 的角色立绘插件，负责管理游戏中的角色立绘和动画，提供角色立绘的显示、动画和布局功能。

## ✨ 核心功能

- **角色立绘管理**：管理游戏中的角色立绘
- **表情切换**：支持角色表情的切换
- **动画系统**：支持角色立绘的动画效果
- **布局管理**：管理角色立绘的布局和位置
- **Z 轴排序**：处理角色立绘的前后顺序
- **特效支持**：支持角色立绘的特效和过渡

## 🚀 使用方法

### 依赖添加

```toml
[dependencies]
gg-plugin-portrait = { path = "projects/plugins/gg-plugin-portrait" }
```

### 基础示例

```rust
use gg_plugin_portrait::prelude::*;

fn main() {
    // 创建角色立绘系统
    let mut portrait_system = PortraitSystem::new();
    
    // 添加角色立绘
    let portrait = Portrait::new()
        .id("alice")
        .texture("assets/characters/alice.png")
        .position((100.0, 200.0))
        .expression("happy")
        .build();
    
    // 显示角色立绘
    portrait_system.add_portrait(portrait);
    
    // 更新角色立绘系统
    portrait_system.update();
}
```

## 📦 依赖关系

- **gg-core**：核心功能和平台抽象
- **gg-world**：游戏世界管理
- **gg-render**：渲染系统
- **gg-error**：错误处理系统

## 📖 相关文档

- [插件系统设计](../../../design/architecture/overview.md) - 了解插件系统的设计理念