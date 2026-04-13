# gg-plugin-dialogue

**GG Game Engine 的对话系统插件，负责处理游戏中的对话和剧情。**

## 📋 模块简介

gg-plugin-dialogue 是 GG Game Engine 的对话系统插件，负责处理游戏中的对话和剧情，提供对话显示、历史记录和类型writer效果等功能。

## ✨ 核心功能

- **对话显示**：显示游戏中的对话和台词
- **角色管理**：管理对话中的角色和发言
- **历史记录**：记录对话历史，支持回顾
- **类型writer效果**：提供文字逐字显示的效果
- **对话命令**：支持对话中的命令和特效
- **剧情管理**：管理游戏剧情的流程和分支

## 🚀 使用方法

### 依赖添加

```toml
[dependencies]
gg-plugin-dialogue = { path = "projects/plugins/gg-plugin-dialogue" }
```

### 基础示例

```rust
use gg_plugin_dialogue::prelude::*;

fn main() {
    // 创建对话系统
    let mut dialogue_system = DialogueSystem::new();
    
    // 添加对话
    let dialogue = Dialogue::new()
        .speaker("Alice")
        .text("Hello, welcome to GG Game Engine!")
        .build();
    
    // 显示对话
    dialogue_system.show_dialogue(dialogue);
    
    // 更新对话系统
    dialogue_system.update();
}
```

## 📦 依赖关系

- **gg-core**：核心功能和平台抽象
- **gg-world**：游戏世界管理
- **gg-render**：渲染系统
- **gg-error**：错误处理系统

## 📖 相关文档

- [插件系统设计](../../../design/architecture/overview.md) - 了解插件系统的设计理念