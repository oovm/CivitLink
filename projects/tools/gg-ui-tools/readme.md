# gg-ui-tools

**GG Game Engine 的 UI 工具集，提供各种 UI 相关的工具和辅助功能。**

## 📋 模块简介

gg-ui-tools 是 GG Game Engine 的 UI 工具集，提供各种 UI 相关的工具和辅助功能，简化 UI 开发和调试过程。

## ✨ 核心功能

- **UI 工具**：各种 UI 开发和调试工具
- **UI 编辑器**：辅助 UI 设计和布局的编辑器工具
- **UI 测试**：UI 功能和性能测试工具
- **UI 生成器**：自动生成 UI 代码和资源的工具
- **UI 模板**：常用 UI 组件和布局的模板
- **UI 分析**：UI 性能和使用情况分析工具

## 🚀 使用方法

### 依赖添加

```toml
[dependencies]
gg-ui-tools = { path = "projects/tools/gg-ui-tools" }
```

### 基础示例

```rust
use gg_ui_tools::prelude::*;

fn main() {
    // 创建 UI 工具管理器
    let ui_tools_manager = UiToolsManager::new();
    
    // 使用 UI 编辑器
    let ui_editor = ui_tools_manager.create_ui_editor();
    
    // 加载 UI 模板
    let template = ui_editor.load_template("main_menu");
    
    // 编辑 UI
    template.add_button("StartGame", "开始游戏");
    template.add_button("Options", "选项");
    
    // 生成代码
    let code = ui_editor.generate_code(template);
    println!("生成的 UI 代码: {}", code);
    
    // 测试 UI
    ui_tools_manager.test_ui(template);
}
```

## 📦 依赖关系

- **gg-error**：错误处理系统

## 📖 相关文档

- [UI API 文档](../../../doc/ui-api.md) - 了解 UI 系统的 API 使用
- [工具使用指南](../../../design/guide/examples.md) - 了解如何使用 UI 工具集