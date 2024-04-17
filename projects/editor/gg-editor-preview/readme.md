# gg-editor-preview

**GG Game Engine 的游戏预览模块，负责在编辑器中预览游戏效果。**

## 📋 模块简介

gg-editor-preview 是 GG Game Engine 的游戏预览模块，负责在编辑器中预览游戏效果，提供实时预览和热重载功能。

## ✨ 核心功能

- **游戏预览**：在编辑器中实时预览游戏效果
- **热重载**：支持代码和资源的热重载，无需重启游戏
- **调试支持**：提供游戏调试功能
- **性能分析**：显示游戏性能指标
- **输入模拟**：模拟用户输入，测试游戏交互
- **场景切换**：支持在不同场景之间切换预览

## 🚀 使用方法

### 依赖添加

```toml
[dependencies]
gg-editor-preview = { path = "projects/editor/gg-editor-preview" }
```

### 基础示例

```rust
use gg_editor_preview::prelude::*;

fn main() {
    // 创建预览面板
    let mut preview = PreviewPanel::new();
    
    // 加载游戏
    preview.load_game("game.toml");
    
    // 启动预览
    preview.start_preview();
    
    // 显示面板
    preview.show();
}
```

## 📦 依赖关系

- **gg-editor-shell**：编辑器外壳和基础框架
- **gg-core**：核心功能和平台抽象
- **gg-runtime**：运行时核心
- **gg-error**：错误处理系统

## 📖 相关文档

- [编辑器设计](../../../design/architecture/overview.md) - 了解编辑器系统的设计理念