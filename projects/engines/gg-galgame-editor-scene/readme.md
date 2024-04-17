# gg-editor-scene-galgame

**GG Game Engine 的 GAL 游戏场景编辑器，负责编辑和管理 GAL 游戏场景。**

## 📋 模块简介

gg-editor-scene-galgame 是 GG Game Engine 的 GAL 游戏场景编辑器，专门用于编辑和管理 GAL 游戏场景，提供对话、角色立绘和场景管理功能。

## ✨ 核心功能

- **GAL 场景编辑**：创建、编辑和管理 GAL 游戏场景
- **对话编辑**：编辑游戏中的对话和剧情
- **角色立绘管理**：管理角色立绘和表情
- **场景过渡**：设置场景之间的过渡效果
- **背景管理**：管理场景背景和氛围
- **预览功能**：实时预览 GAL 游戏场景效果

## 🚀 使用方法

### 依赖添加

```toml
[dependencies]
gg-editor-scene-galgame = { path = "projects/editor/gg-editor-scene-galgame" }
```

### 基础示例

```rust
use gg_editor_scene_galgame::prelude::*;

fn main() {
    // 创建 GAL 场景编辑器面板
    let mut galgame_scene_editor = GalgameSceneEditorPanel::new();
    
    // 加载场景
    galgame_scene_editor.load_scene("scenes/galgame.scene");
    
    // 添加对话
    let dialogue = galgame_scene_editor.add_dialogue();
    dialogue.set_speaker("Alice");
    dialogue.set_text("Hello, welcome to GG Game Engine!");
    
    // 添加角色立绘
    let portrait = galgame_scene_editor.add_portrait("Alice");
    portrait.set_expression("happy");
    portrait.set_position((100.0, 200.0));
    
    // 显示面板
    galgame_scene_editor.show();
}
```

## 📦 依赖关系

- **gg-editor-shell**：编辑器外壳和基础框架
- **gg-core**：核心功能和平台抽象
- **gg-editor-scene**：场景编辑器
- **gg-plugin-dialogue**：对话系统插件
- **gg-plugin-portrait**：角色立绘插件
- **gg-error**：错误处理系统

## 📖 相关文档

- [编辑器设计](../../../design/architecture/overview.md) - 了解编辑器系统的设计理念