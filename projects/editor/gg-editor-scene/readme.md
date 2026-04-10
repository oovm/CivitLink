# gg-editor-scene

**GG Game Engine 的场景编辑器，负责编辑和管理游戏场景。**

## 📋 模块简介

gg-editor-scene 是 GG Game Engine 的场景编辑器，负责编辑和管理游戏场景，提供可视化的场景编辑界面。

## ✨ 核心功能

- **场景编辑**：创建、编辑和管理游戏场景
- **对象操作**：添加、删除、移动、缩放和旋转场景对象
- **视图控制**：控制场景视图的平移、旋转和缩放
- **网格和对齐**：提供网格辅助和对象对齐功能
- **场景层次**：管理场景对象的层次结构
- **场景预览**：实时预览场景效果

## 🚀 使用方法

### 依赖添加

```toml
[dependencies]
gg-editor-scene = { path = "projects/editor/gg-editor-scene" }
```

### 基础示例

```rust
use gg_editor_scene::prelude::*;

fn main() {
    // 创建场景编辑器面板
    let mut scene_editor = SceneEditorPanel::new();
    
    // 加载场景
    scene_editor.load_scene("scenes/main.scene");
    
    // 添加对象
    let object = scene_editor.add_object("Player");
    object.set_position((100.0, 100.0));
    
    // 显示面板
    scene_editor.show();
}
```

## 📦 依赖关系

- **gg-editor-shell**：编辑器外壳和基础框架
- **gg-core**：核心功能和平台抽象
- **gg-world**：游戏世界管理
- **gg-render**：渲染系统
- **gg-error**：错误处理系统

## 📖 相关文档

- [编辑器设计](../../../design/architecture/overview.md) - 了解编辑器系统的设计理念