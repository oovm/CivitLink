# GG Runtime Core

GG 引擎运行时核心模块，提供基本的运行时系统和游戏循环。

## 功能

- 运行时系统管理
- 游戏循环实现
- 插件系统集成
- 示例系统和组件

## 使用示例

```rust
use gg_runtime_core::{Runtime, EntityCountSystem, MovementSystem, Position, Velocity};
use gg_render::RenderComponent;

// 创建运行时
let mut runtime = Runtime::new().unwrap();

// 获取世界
let world = runtime.scheduler().world();

// 注册系统
world.register_system(Box::new(EntityCountSystem::new()));
world.register_system(Box::new(MovementSystem::new()));

// 创建实体
let entity = world.spawn();

// 添加组件
world.add_component(entity, Position { x: 0.0, y: 0.0 }).unwrap();
world.add_component(entity, Velocity { dx: 1.0, dy: 1.0 }).unwrap();
world.add_component(entity, RenderComponent {
    width: 50.0,
    height: 50.0,
    color: [1.0, 0.0, 0.0, 1.0],
}).unwrap();

// 启动运行时
runtime.start().unwrap();
```
