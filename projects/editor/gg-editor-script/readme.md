# gg-editor-script

**GG Game Engine 的脚本编辑器，负责编辑和管理游戏脚本。**

## 📋 模块简介

gg-editor-script 是 GG Game Engine 的脚本编辑器，负责编辑和管理游戏脚本，提供代码编辑、语法高亮和调试功能。

## ✨ 核心功能

- **代码编辑**：提供代码编辑界面，支持语法高亮和自动补全
- **脚本管理**：创建、编辑和管理游戏脚本
- **语法检查**：实时检查脚本语法错误
- **代码模板**：提供常用代码模板
- **调试支持**：支持脚本调试和断点设置
- **LSP 集成**：集成语言服务器协议，提供高级代码分析

## 🚀 使用方法

### 依赖添加

```toml
[dependencies]
gg-editor-script = { path = "projects/editor/gg-editor-script" }
```

### 基础示例

```rust
use gg_editor_script::prelude::*;

fn main() {
    // 创建脚本编辑器面板
    let mut script_editor = ScriptEditorPanel::new();
    
    // 加载脚本
    script_editor.load_script("scripts/game.valkyrie");
    
    // 设置语法高亮
    script_editor.set_language("valkyrie");
    
    // 显示面板
    script_editor.show();
}
```

## 📦 依赖关系

- **gg-editor-shell**：编辑器外壳和基础框架
- **gg-core**：核心功能和平台抽象
- **gg-script**：脚本解析和执行
- **gg-editor-lsp**：语言服务器协议支持
- **gg-error**：错误处理系统

## 📖 相关文档

- [编辑器设计](../../../design/architecture/overview.md) - 了解编辑器系统的设计理念