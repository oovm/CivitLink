# gg-world

**GG Game Engine 的游戏世界管理系统，负责管理游戏中的实体、组件和系统。**

## 📋 模块简介

gg-world 是 GG Game Engine 的游戏世界管理系统，负责管理游戏中的实体、组件和系统，是 ECS 系统的扩展和封装，提供更高级的游戏世界管理功能。

## ✨ 核心功能

- **世界管理**：创建和管理游戏世界
- **实体管理**：创建、销毁和管理游戏实体
- **组件系统**：为实体添加和移除组件
- **系统管理**：管理游戏系统的注册和执行
- **事件系统**：支持游戏事件的发送和接收
- **资源管理**：管理游戏世界中的资源

## 🚀 使用方法

### 依赖添加

```toml
[dependencies]
gg-world = { path = "projects/core/gg-world" }
```

### 基础示例

```rust
use gg_world::prelude::*;

// 定义组件
#[derive(Component)]
struct Position { x: f32, y: f32 }

#[derive(Component)]
struct Velocity { dx: f32, dy: f32 }

// 定义系统
struct MovementSystem;
impl System for MovementSystem {
    type Query = (Read<Position>, Write<Velocity>);
    
    fn run(&mut self, world: &mut World, query: Query<Self::Query>) {
        for (position, mut velocity) in query.iter() {
            velocity.dx += 0.1;
            velocity.dy += 0.1;
        }
    }
}

fn main() {
    // 创建世界
    let mut world = World::new();
    
    // 添加系统
    world.add_system(MovementSystem);
    
    // 创建实体
    let entity = world.create_entity()
        .with(Position { x: 0.0, y: 0.0 })
        .with(Velocity { dx: 0.0, dy: 0.0 })
        .build();
    
    // 运行世界
    world.update();
}
```

## 📦 依赖关系

- **gg-core**：核心功能和平台抽象
- **gg-ecs**：实体组件系统
- **gg-schedule**：任务调度系统
- **gg-error**：错误处理系统

## 📖 相关文档

- [世界管理设计](../../../design/modules/world.md) - 了解世界管理系统的设计理念