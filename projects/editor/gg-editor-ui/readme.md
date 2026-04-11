# gg-editor-ui

**GG Game Engine 的编辑器 UI 系统，基于 Valkyrie Widget 构建，提供现代化的编辑器界面。**

## 📋 模块简介

gg-editor-ui 是 GG Game Engine 的编辑器 UI 系统，基于 Valkyrie Widget 构建，提供现代化、可定制的编辑器界面，支持各种编辑器面板和交互功能。

## ✨ 核心功能

- **现代化 UI**：基于 Valkyrie Widget 的现代化编辑器界面
- **可定制面板**：支持可拖拽、可停靠的编辑器面板
- **主题支持**：支持明暗主题切换
- **响应式设计**：适配不同屏幕尺寸
- **事件系统**：完善的 UI 事件处理系统
- **样式系统**：可自定义的 UI 样式系统

## 🚀 使用方法

### 依赖添加

```toml
[dependencies]
gg-editor-ui = { path = "projects/editor/gg-editor-ui" }
```

### 基础示例

```rust
use gg_editor_ui::prelude::*;

fn main() {
    // 创建编辑器 UI 管理器
    let ui_manager = EditorUiManager::new();
    
    // 创建主窗口
    let main_window = ui_manager.create_window("GG Editor");
    
    // 添加面板
    main_window.add_panel("Scene", || {
        // 场景编辑器面板内容
    });
    
    main_window.add_panel("Inspector", || {
        // 属性检查器面板内容
    });
    
    // 运行 UI 循环
    ui_manager.run();
}
```

## 📦 依赖关系

- **oak-voc**：Valkyrie Widget 核心库

## 📖 相关文档

- [UI API 文档](../../../doc/ui-api.md) - 了解 UI 系统的 API 使用
- [编辑器架构设计](../../../design/architecture/overview.md) - 了解编辑器的整体架构