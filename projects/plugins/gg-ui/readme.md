# gg-ui

**GG Game Engine 的 UI 系统，负责创建和管理游戏中的用户界面。**

## 📋 模块简介

gg-ui 是 GG Game Engine 的 UI 系统，负责创建和管理游戏中的用户界面，提供各种 UI 组件和布局功能。

## ✨ 核心功能

- **UI 组件**：提供各种 UI 组件（按钮、文本框、面板等）
- **布局系统**：支持各种布局方式（水平、垂直、网格等）
- **事件系统**：处理 UI 事件（点击、 hover、输入等）
- **样式系统**：支持 UI 样式的自定义和管理
- **节点系统**：基于节点的 UI 层次结构
- **渲染系统**：优化的 UI 渲染

## 🚀 使用方法

### 依赖添加

```toml
[dependencies]
gg-ui = { path = "projects/plugins/gg-ui" }
```

### 基础示例

```rust
use gg_ui::prelude::*;

fn main() {
    // 创建 UI 系统
    let mut ui_system = UiSystem::new();
    
    // 创建 UI 根节点
    let root = ui_system.create_root();
    
    // 创建按钮
    let button = Button::new()
        .text("Click Me")
        .position((100.0, 100.0))
        .size((200.0, 50.0))
        .on_click(|| {
            println!("Button clicked!");
        })
        .build();
    
    // 添加到根节点
    root.add_child(button);
    
    // 更新 UI 系统
    ui_system.update();
}
```

## 📦 依赖关系

- **gg-core**：核心功能和平台抽象
- **gg-world**：游戏世界管理
- **gg-render**：渲染系统
- **gg-error**：错误处理系统

## 📖 相关文档

- [插件系统设计](../../../design/architecture/overview.md) - 了解插件系统的设计理念