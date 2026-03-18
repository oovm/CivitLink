# ECS 模块

## 概述

ECS（Entity-Component-System）是 GWG 元引擎的核心框架，基于 `bevy_ecs` 提供高性能、数据驱动的游戏对象系统。

## 核心概念

### 实体（Entity）
实体是游戏世界中对象的唯一标识符，本身不包含任何数据或逻辑。

```rust
use gwg_ecs::prelude::*;

let mut world = GwgWorld::new();
let entity = world.spawn().id();
```

### 组件（Component）
组件是纯数据结构，附加在实体上。

```rust
#[derive(Component)]
struct Position {
    x: f32,
    y: f32,
}

#[derive(Component)]
struct Velocity {
    x: f32,
    y: f32,
}
```

### 系统（System）
系统是纯逻辑函数，操作具有特定组件的实体。

```rust
fn move_system(mut query: Query<(&mut Position, &Velocity)>) {
    for (mut pos, vel) in query.iter_mut() {
        pos.x += vel.x;
        pos.y += vel.y;
    }
}
```

### 资源（Resource）
资源是全局单例数据，例如配置、时间等。

```rust
#[derive(Resource)]
struct GameTime {
    delta_seconds: f32,
}
```

## 核心类型

### GwgWorld

游戏世界的主容器，管理所有实体、组件和资源。

```rust
pub struct GwgWorld {
    inner: bevy_ecs::world::World,
}
```

**主要方法：**

- `new()` - 创建新的空世界
- `spawn()` - 生成新实体
- `entity_mut(entity)` - 获取实体可变引用
- `entity(entity)` - 获取实体不可变引用
- `despawn(entity)` - 销毁实体
- `insert_resource(resource)` - 插入全局资源
- `get_resource<T>()` - 获取全局资源

### GwgSchedule

系统调度器，用于组织和执行系统。

```rust
pub struct GwgSchedule {
    inner: bevy_ecs::schedule::Schedule,
}
```

**主要方法：**

- `new()` - 创建新的空调度器
- `add_system(system)` - 添加系统
- `run(world)` - 运行调度器中的所有系统

## 使用示例

### 基础用法

```rust
use gwg_ecs::prelude::*;

#[derive(Component, Debug)]
struct Health(f32);

#[derive(Component)]
struct Damage(f32);

fn damage_system(mut query: Query<(&mut Health, &Damage)>) {
    for (mut health, damage) in query.iter_mut() {
        health.0 -= damage.0;
        if health.0 <= 0.0 {
            println!("Entity died!");
        }
    }
}

fn main() {
    let mut world = GwgWorld::new();
    let mut schedule = GwgSchedule::new();

    schedule.add_system(damage_system);

    let player = world.spawn()
        .insert(Health(100.0))
        .insert(Damage(10.0))
        .id();

    schedule.run(&mut world);
}
```

### 查询过滤

```rust
fn system(
    query: Query<(&Position, &Velocity), With<Player>>,
    without_query: Query<&Position, Without<Enemy>>,
) {
    // ...
}
```

### 变更检测

```rust
fn system(mut query: Query<&mut Position, Changed<Position>>) {
    for mut pos in query.iter_mut() {
        // 只处理位置改变的实体
    }
}
```

## 相关模块

- [schedule](./schedule.md) - 调度器扩展
- [world](./world.md) - 世界管理
- [reflection](./reflection.md) - 反射系统
- [asset](./asset.md) - 资源管理
