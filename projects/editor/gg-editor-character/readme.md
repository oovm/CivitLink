# gg-editor-character

**GG Game Engine 的角色编辑器，负责编辑和管理游戏中的角色。**

## 📋 模块简介

gg-editor-character 是 GG Game Engine 的角色编辑器，负责编辑和管理游戏中的角色，提供角色属性、动画和行为的编辑功能。

## ✨ 核心功能

- **角色编辑**：创建、编辑和管理游戏角色
- **属性编辑**：编辑角色的各种属性（名称、健康值、攻击力等）
- **动画管理**：管理角色的动画和动作
- **行为编辑**：编辑角色的行为和状态
- **外观定制**：定制角色的外观和服装
- **预览功能**：实时预览角色效果

## 🚀 使用方法

### 依赖添加

```toml
[dependencies]
gg-editor-character = { path = "projects/editor/gg-editor-character" }
```

### 基础示例

```rust
use gg_editor_character::prelude::*;

fn main() {
    // 创建角色编辑器面板
    let mut character_editor = CharacterEditorPanel::new();
    
    // 创建新角色
    let mut character = character_editor.create_character("Player");
    
    // 设置角色属性
    character.set_property("health", 100.0);
    character.set_property("attack", 20.0);
    
    // 添加动画
    character.add_animation("idle", "animations/idle.animation");
    character.add_animation("walk", "animations/walk.animation");
    
    // 显示面板
    character_editor.show();
}
```

## 📦 依赖关系

- **gg-editor-shell**：编辑器外壳和基础框架
- **gg-core**：核心功能和平台抽象
- **gg-reflection**：反射系统
- **gg-error**：错误处理系统

## 📖 相关文档

- [编辑器设计](../../../design/architecture/overview.md) - 了解编辑器系统的设计理念