# gg-plugin-ui

GG 引擎的 UI 系统插件，提供游戏内 UI 组件和功能。

## 功能

- 基础 UI 组件（按钮、文本、面板等）
- UI 布局系统
- UI 事件处理
- 与 ECS 系统集成

## 使用

```rust
use gg_plugin_ui::UiPlugin;

app.add_plugin(UiPlugin);
```

## 依赖

- `gg-core`: 核心类型定义
- `gg-ecs`: ECS 系统
- `gg-runtime-gui`: GUI 运行时