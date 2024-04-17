# gg-ecs

**GG Game Engine 的实体组件系统，提供高效的游戏对象管理。**

## 📋 模块简介

gg-ecs 是 GG Game Engine 的实体组件系统（Entity Component System），提供高效的游戏对象管理和系统更新机制，是游戏逻辑的核心基础。

## ✨ 核心功能

- **实体管理**：创建、销毁和管理游戏实体
- **组件系统**：为实体添加和移除组件
- **系统更新**：基于组件类型的系统更新机制
- **查询系统**：高效的组件查询和过滤
- **批处理**：支持批量实体操作

## 🚀 使用方法

### 依赖添加

```toml
[dependencies]
gg-ecs = { path = "projects/core/gg-ecs" }
```

### 基础示例

```rust
use gg_ecs::prelude::*;

// 定义组件
#[derive(Component)]
struct Position { x: f32, y: f32 }

#[derive(Component)]
struct Velocity { dx: f32, dy: f32 }

// 定义系统
struct MovementSystem;

impl System for MovementSystem {
    type Query = (Read<Position>, Write<Velocity>);
    
    fn update(&mut self, entities: Query<Self::Query>) {
        for (position, mut velocity) in entities.iter() {
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
    
    // 运行系统
    world.update();
}
```

## 📦 依赖关系

- **gg-core**：核心功能和平台抽象
- **gg-error**：错误处理系统

## 📖 相关文档

- [ECS 设计](../../../design/modules/ecs.md) - 了解实体组件系统的设计理念