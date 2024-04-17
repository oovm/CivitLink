# gg-runtime-gui

GG 引擎的 GUI 运行时模块，提供跨平台的 GUI 渲染和事件处理能力。

## 功能

- 跨平台 GUI 运行时
- GUI 渲染抽象接口
- GUI 事件处理
- 与 ECS 系统集成

## 使用

```rust
use gg_runtime_gui::{GuiRuntime, GuiRenderer};

let runtime = GuiRuntime::new(renderer);
runtime.render();
```

## 依赖

- `gg-core`: 核心类型定义
- `gg-ecs`: ECS 系统
- `gg-render`: 渲染抽象