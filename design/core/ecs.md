# ECS 模块

## 概述

ECS（Entity-Component-System）是 GG 元引擎的核心框架，采用自研实现，提供高性能、数据驱动的游戏对象系统。

## 核心概念

### 实体（Entity）
实体是游戏世界中对象的唯一标识符，本身不包含任何数据或逻辑。

```rust
use gg_ecs::prelude::*;

let mut world = World::new();
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
struct MoveSystem;

impl System for MoveSystem {
    fn name(&self) -> &str {
        "MoveSystem"
    }

    fn execute(&mut self, world: &mut World) -> GResult<()> {
        let mut query = world.query::<Position>();
        for (entity, mut pos) in query.iter_mut() {
            if let Some(vel) = world.get_component::<Velocity>(entity) {
                pos.x += vel.x;
                pos.y += vel.y;
            }
        }
        Ok(())
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

### World

游戏世界的主容器，管理所有实体、组件和资源。

```rust
pub struct World {
    entity_allocator: EntityAllocator,
    archetype_graph: ArchetypeGraph,
    resources: HashMap<TypeId, Box<dyn Any + Send + Sync>>,
    systems: Vec<Box<dyn System>>,
    tick: u32,
    changed_ticks: HashMap<(Entity, TypeId), u32>,
}
```

**主要方法：**

- `new()` - 创建新的空世界
- `spawn()` - 生成新实体并返回实体构建器
- `despawn(entity)` - 销毁实体
- `add_component(entity, component)` - 向实体添加组件
- `get_component<T>(entity)` - 获取实体的组件
- `get_component_mut<T>(entity)` - 获取实体的组件可变引用
- `remove_component<T>(entity)` - 移除实体的组件
- `insert_resource(resource)` - 插入全局资源
- `get_resource<T>()` - 获取全局资源
- `get_resource_mut<T>()` - 获取全局资源可变引用
- `query<T>()` - 查询拥有指定组件的所有实体
- `register_system(system)` - 注册系统
- `run_systems()` - 执行所有注册的系统

### EntityBuilder

实体构建器，用于链式创建实体并添加组件。

```rust
pub struct EntityBuilder<'a> {
    world: &'a mut World,
    entity: Entity,
}
```

**主要方法：**

- `insert(component)` - 向实体添加组件，返回自身以支持链式调用
- `id()` - 获取实体 ID

### Query

组件查询迭代器，用于遍历拥有指定组件类型且满足过滤条件的所有实体。

```rust
pub struct Query<'a, T: Component, F: QueryFilter = NoneFilter> {
    inner: Option<storage::ComponentColumnIter<'a, T>>,
    world: &'a World,
    _filter: PhantomData<F>,
    _marker: PhantomData<&'a T>,
}
```

**主要方法：**

- `iter()` - 获取迭代器，遍历所有匹配的实体和组件
- `iter_mut()` - 获取可变迭代器，遍历所有匹配的实体和组件

### MultiQuery

多组件查询迭代器，支持元组查询。

```rust
pub struct MultiQuery<'a, Q: query::WorldQuery> {
    world: &'a World,
    matching_archetypes: Vec<(ArchetypeId, Q::State)>,
    current_archetype_idx: usize,
    current_row: usize,
    _marker: PhantomData<Q>,
}
```

**主要方法：**

- `for_each(f)` - 使用回调函数遍历所有匹配的实体和组件数据

## 过滤条件

### With<T>
仅匹配包含指定组件的实体。

### Without<T>
仅匹配不包含指定组件的实体。

### Changed<T>
仅匹配自上次 tick 以来组件值发生变化的实体。

## 使用示例

### 基础用法

```rust
use gg_ecs::prelude::*;

#[derive(Component, Debug)]
struct Health(f32);

#[derive(Component)]
struct Damage(f32);

struct DamageSystem;

impl System for DamageSystem {
    fn name(&self) -> &str {
        "DamageSystem"
    }

    fn execute(&mut self, world: &mut World) -> GResult<()> {
        let mut query = world.query::<Health>();
        for (entity, mut health) in query.iter_mut() {
            if let Some(damage) = world.get_component::<Damage>(entity) {
                health.0 -= damage.0;
                if health.0 <= 0.0 {
                    println!("Entity died!");
                }
            }
        }
        Ok(())
    }
}

fn main() {
    let mut world = World::new();

    let player = world.spawn()
        .insert(Health(100.0))
        .insert(Damage(10.0))
        .id();

    world.register_system(Box::new(DamageSystem));
    world.run_systems().unwrap();
}
```

### 查询过滤

```rust
struct Player;
impl Component for Player {}

struct Enemy;
impl Component for Enemy {}

struct PlayerSystem;

impl System for PlayerSystem {
    fn name(&self) -> &str {
        "PlayerSystem"
    }

    fn execute(&mut self, world: &mut World) -> GResult<()> {
        // 只查询带有 Player 组件的实体
        let query = world.query_filtered::<Position, With<Player>>();
        for (entity, pos) in query.iter() {
            println!("Player at: ({}, {})");
        }
        
        // 只查询不带有 Enemy 组件的实体
        let query = world.query_filtered::<Position, Without<Enemy>>();
        for (entity, pos) in query.iter() {
            println!("Non-enemy at: ({}, {})");
        }
        Ok(())
    }
}
```

### 变更检测

```rust
struct PositionSystem;

impl System for PositionSystem {
    fn name(&self) -> &str {
        "PositionSystem"
    }

    fn execute(&mut self, world: &mut World) -> GResult<()> {
        // 只处理位置改变的实体
        let query = world.query_filtered::<Position, Changed<Position>>();
        for (entity, pos) in query.iter() {
            println!("Position changed to: ({}, {})");
        }
        Ok(())
    }
}
```

## 相关模块

- [asset](../core/asset.md) - 资源管理
- [world](../core/world.md) - 世界管理
- [reflection](../core/reflection.md) - 反射系统
- [schedule](../core/schedule.md) - 调度器