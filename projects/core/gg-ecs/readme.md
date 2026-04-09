# GG ECS

GG 引擎 ECS 核心模块，提供实体-组件-系统架构。

## 功能

- 实体管理
- 组件存储和检索
- 系统注册和执行
- 系统调度器

## 使用示例

```rust
use gg_ecs::{World, Component, System};
use gg_core::GResult;

// 定义组件
struct Position { x: f32, y: f32 }
impl Component for Position {}

struct Velocity { dx: f32, dy: f32 }
impl Component for Velocity {}

// 定义系统
struct MovementSystem;
impl System for MovementSystem {
    fn name(&self) -> &str { "MovementSystem" }
    
    fn execute(&mut self, world: &mut World) -> GResult<()> {
        for entity in world.entities() {
            if let (Some(pos), Some(vel)) = (
                world.get_component::<Position>(*entity),
                world.get_component::<Velocity>(*entity)
            ) {
                // 移动实体
            }
        }
        Ok(())
    }
}

// 使用 ECS
let mut world = World::new();
let entity = world.spawn();
world.add_component(entity, Position { x: 0.0, y: 0.0 }).unwrap();
world.add_component(entity, Velocity { dx: 1.0, dy: 1.0 }).unwrap();
world.register_system(Box::new(MovementSystem));
world.run_systems().unwrap();
```
