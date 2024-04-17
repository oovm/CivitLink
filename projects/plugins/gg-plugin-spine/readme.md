# gg-plugin-spine

**GG Game Engine 的 Spine 动画插件，负责管理和播放 Spine 动画。**

## 📋 模块简介

gg-plugin-spine 是 GG Game Engine 的 Spine 动画插件，负责管理和播放 Spine 动画，为游戏提供高质量的骨骼动画效果。

## ✨ 核心功能

- **Spine 动画支持**：支持加载和播放 Spine 动画
- **骨骼动画**：管理骨骼动画的播放和控制
- **皮肤切换**：支持 Spine 动画的皮肤切换
- **动画混合**：支持动画之间的混合和过渡
- **事件系统**：支持 Spine 动画的事件触发
- **性能优化**：优化 Spine 动画的渲染性能

## 🚀 使用方法

### 依赖添加

```toml
[dependencies]
gg-plugin-spine = { path = "projects/plugins/gg-plugin-spine" }
```

### 基础示例

```rust
use gg_plugin_spine::prelude::*;

fn main() {
    // 创建 Spine 动画系统
    let mut spine_system = SpineSystem::new();
    
    // 加载 Spine 动画
    let spine_animation = SpineAnimation::new()
        .path("assets/animations/character.json")
        .position((400.0, 300.0))
        .scale(1.0)
        .build();
    
    // 添加到系统
    let animation_id = spine_system.add_animation(spine_animation);
    
    // 播放动画
    spine_system.play_animation(animation_id, "walk", true);
    
    // 更新系统
    spine_system.update();
}
```

## 📦 依赖关系

- **gg-core**：核心功能和平台抽象
- **gg-world**：游戏世界管理
- **gg-render**：渲染系统
- **gg-error**：错误处理系统

## 📖 相关文档

- [插件系统设计](../../../design/architecture/overview.md) - 了解插件系统的设计理念