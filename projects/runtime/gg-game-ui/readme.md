# gg-game-ui

**GG Game Engine 的游戏 UI 系统，基于 GameObject 和 Component 构建，提供游戏内 UI 解决方案。**

## 📋 模块简介

gg-game-ui 是 GG Game Engine 的游戏 UI 系统，基于 GameObject 和 Component 架构构建，为游戏提供灵活、高效的 UI 解决方案，支持各种游戏内 UI 场景。

## ✨ 核心功能

- **基于组件**：基于 GameObject 和 Component 架构
- **UI 元素**：支持按钮、文本、图片、面板等常见 UI 元素
- **布局系统**：支持各种布局方式（网格、水平、垂直等）
- **事件系统**：完善的 UI 事件处理（点击、 hover 等）
- **动画支持**：支持 UI 元素的动画效果
- **响应式设计**：适配不同屏幕尺寸
- **主题系统**：支持可定制的 UI 主题

## 🚀 使用方法

### 依赖添加

```toml
[dependencies]
gg-game-ui = { path = "projects/runtime/gg-game-ui" }
```

### 基础示例

```rust
use gg_game_ui::prelude::*;

fn main() {
    // 创建 UI 管理器
    let ui_manager = GameUiManager::new();
    
    // 创建主菜单
    let main_menu = ui_manager.create_ui("MainMenu");
    
    // 添加按钮
    main_menu.add_button("StartGame", "开始游戏", || {
        println!("开始游戏");
    });
    
    main_menu.add_button("Options", "选项", || {
        println!("打开选项");
    });
    
    main_menu.add_button("Quit", "退出游戏", || {
        println!("退出游戏");
    });
    
    // 显示 UI
    ui_manager.show("MainMenu");
}
```

## 📦 依赖关系

- **gg-ecs**：实体组件系统
- **gg-error**：错误处理系统
- **gg-render**：渲染系统

## 📖 相关文档

- [UI 系统架构](../../../doc/ui-system-architecture.md) - 了解 UI 系统的架构设计
- [游戏开发指南](../../../design/guide/getting-started.md) - 了解如何在游戏中使用 UI 系统