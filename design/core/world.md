# World 模块

## 概述

World 模块提供游戏世界的创建、销毁和管理功能，整合了 ECS 世界、资源服务器和反射注册表。

## 核心概念

### 游戏世界（GameWorld）
游戏世界是一个独立的游戏环境，包含自己的实体、资源和规则。

### 世界管理器（WorldManager）
世界管理器负责创建、管理和销毁多个游戏世界。

## 核心类型

### GameWorld

游戏世界，整合 ECS 世界、资源服务器和反射注册表。

```rust
pub struct GameWorld {
    pub ecs_world: World,
    pub asset_server: AssetServer,
    pub reflection_registry: ReflectionRegistry,
    name: String,
    is_destroyed: bool,
}
```

**主要方法：**

- `new(name)` - 创建新的游戏世界
- `name()` - 获取世界名称
- `is_destroyed()` - 检查世界是否已销毁
- `destroy()` - 销毁世界
- `clear()` - 清空世界中的所有实体和资源
- `spawn()` - 生成新实体
- `despawn(entity)` - 销毁实体
- `entity(entity)` - 获取实体引用
- `add_component(entity, component)` - 向实体添加组件
- `get_component<T>(entity)` - 获取实体的组件
- `get_component_mut<T>(entity)` - 获取实体的组件可变引用
- `remove_component<T>(entity)` - 移除实体的组件
- `query<T>()` - 查询拥有指定组件的所有实体
- `insert_resource(resource)` - 插入全局资源
- `get_resource<T>()` - 获取全局资源
- `get_resource_mut<T>()` - 获取全局资源可变引用
- `register_system(system)` - 注册系统
- `run_systems()` - 执行所有注册的系统
- `serialize(registry)` - 序列化世界数据
- `deserialize(data, registry)` - 反序列化世界数据

### EntityRef

实体引用，用于访问指定世界中的实体信息。

```rust
pub struct EntityRef<'a> {
    world: &'a GameWorld,
    entity: Entity,
}
```

**主要方法：**

- `id()` - 获取实体标识符
- `is_alive()` - 检查实体是否存活

### WorldManager

世界管理器，负责管理多个游戏世界。

```rust
pub struct WorldManager {
    worlds: Vec<Option<GameWorld>>,
    active_world: Option<usize>,
    next_id: usize,
}
```

**主要方法：**

- `new()` - 创建新的世界管理器
- `create_world(name)` - 创建新的游戏世界
- `destroy_world(id)` - 销毁指定 ID 的世界
- `get_world(id)` - 获取指定 ID 的世界
- `get_world_mut(id)` - 获取指定 ID 的世界的可变引用
- `active_world()` - 获取当前活动的世界
- `active_world_mut()` - 获取当前活动的世界的可变引用
- `set_active_world(id)` - 设置活动世界
- `world_ids()` - 获取所有世界的 ID 列表
- `destroy_all_worlds()` - 销毁所有世界

### SceneData

场景序列化数据，用于保存和加载游戏场景。

```rust
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SceneData {
    pub version: String,
    pub name: String,
    pub entities: Vec<EntityData>,
}
```

### EntityData

实体序列化数据，用于保存和加载实体信息。

```rust
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct EntityData {
    pub id: u64,
    pub name: String,
    pub parent_id: Option<u64>,
    pub components: Vec<ComponentData>,
}
```

### ComponentData

组件序列化数据，用于保存和加载组件信息。

```rust
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ComponentData {
    pub type_name: String,
    pub properties: serde_json::Value,
}
```

## 使用示例

### 单世界使用

```rust
use gg_world::prelude::*;
use gg_ecs::prelude::*;

#[derive(Component)]
struct Position {
    x: f32,
    y: f32,
}

#[derive(Component)]
struct Player;

#[derive(Resource)]
struct GameConfig {
    max_players: u32,
    difficulty: Difficulty,
}

#[derive(Debug, Clone, Copy)]
enum Difficulty {
    Easy,
    Normal,
    Hard,
}

fn main() {
    let mut world = GameWorld::new("My Game".to_string());
    
    // 生成实体
    let player = world.spawn()
        .insert(Position { x: 0.0, y: 0.0 })
        .insert(Player)
        .id();
    
    // 插入全局资源
    world.insert_resource(GameConfig {
        max_players: 4,
        difficulty: Difficulty::Normal,
    });
    
    // 获取资源
    if let Some(config) = world.get_resource::<GameConfig>() {
        println!("Max players: {}", config.max_players);
    }
    
    // 清空世界
    world.clear();
    
    // 销毁世界
    world.destroy();
}
```

### 多世界管理

```rust
use gg_world::prelude::*;
use gg_ecs::prelude::*;

#[derive(Component)]
struct MenuButton;

#[derive(Component)]
struct Player;

fn main() {
    let mut manager = WorldManager::new();
    
    // 创建多个世界
    let menu_world_id = manager.create_world("Menu".to_string());
    let game_world_id = manager.create_world("Game".to_string());
    
    // 获取世界并操作
    if let Some(menu_world) = manager.get_world_mut(menu_world_id) {
        menu_world.spawn().insert(MenuButton);
    }
    
    // 切换活动世界
    manager.set_active_world(game_world_id);
    
    // 获取当前活动世界
    if let Some(active_world) = manager.active_world_mut() {
        active_world.spawn().insert(Player);
    }
    
    // 销毁单个世界
    manager.destroy_world(menu_world_id);
    
    // 销毁所有世界
    manager.destroy_all_worlds();
}
```

### 场景序列化与反序列化

```rust
use gg_world::prelude::*;
use gg_ecs::prelude::*;
use gg_reflection::prelude::*;

#[derive(Component, Reflect)]
struct Position {
    x: f32,
    y: f32,
}

#[derive(Component, Reflect)]
struct Player {
    name: String,
    health: f32,
}

fn main() {
    let mut world = GameWorld::new("Game".to_string());
    let mut registry = ReflectionRegistry::new();
    
    // 注册类型
    registry.register::<Position>();
    registry.register::<Player>();
    
    // 生成实体
    world.spawn()
        .insert(Position { x: 10.0, y: 20.0 })
        .insert(Player { name: "Hero".to_string(), health: 100.0 });
    
    // 序列化场景
    let scene_data = SceneSerializer::serialize(&world, &registry);
    let ron_string = scene_data.to_ron().unwrap();
    println!("Scene data: {}", ron_string);
    
    // 反序列化场景
    let loaded_scene_data = SceneData::from_ron(&ron_string).unwrap();
    let new_world = SceneDeserializer::deserialize(&loaded_scene_data, &registry);
    
    // 验证加载结果
    let query = new_world.query::<Player>();
    for (entity, player) in query.iter() {
        println!("Loaded player: {}, health: {}", player.name, player.health);
    }
}
```

## 相关模块

- [ecs](../core/ecs.md) - ECS 核心
- [asset](../core/asset.md) - 资源管理
- [reflection](../core/reflection.md) - 反射系统