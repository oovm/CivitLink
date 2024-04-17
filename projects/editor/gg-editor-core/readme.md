# gg-editor-core

GG 编辑器核心模块，提供编辑器的基础功能和接口。

## 功能

- 编辑器核心接口定义
- 面板管理系统
- 编辑器事件系统
- 插件扩展机制

## 使用

```rust
use gg_editor_core::EditorCore;

let editor = EditorCore::new();
editor.initialize();
```

## 依赖

- `gg-core`: 核心类型定义
- `gg-ecs`: ECS 系统
- `gg-asset`: 资源管理