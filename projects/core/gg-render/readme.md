# GG Render

GG 引擎渲染系统，提供窗口管理和 2D 图形渲染功能。

## 功能

- 渲染组件定义
- 渲染系统实现
- 帧渲染管理

## 使用示例

```rust
use gg_render::{RenderComponent, RenderSystem};
use gg_ecs::World;

// 创建渲染系统
let mut render_system = RenderSystem::new().unwrap();
render_system.init().unwrap();

// 创建世界和实体
let mut world = World::new();
let entity = world.spawn();

// 添加渲染组件
world.add_component(entity, RenderComponent {
    width: 100.0,
    height: 100.0,
    color: [1.0, 0.0, 0.0, 1.0], // 红色
}).unwrap();

// 渲染一帧
render_system.render().unwrap();
```
