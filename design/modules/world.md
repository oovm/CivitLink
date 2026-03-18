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
    pub ecs_world: GwgWorld,
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
- `entity(entity)` - 获取实体
- `entity_mut(entity)` - 获取实体可变引用
- `despawn(entity)` - 销毁实体
- `insert_resource(resource)` - 插入全局资源
- `get_resource<T>()` - 获取全局资源
- `get_resource_mut<T>()` - 获取全局资源可变引用

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

## 使用示例

### 单世界使用

```rust
use gwg_world::prelude::*;

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
use gwg_world::prelude::*;

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

## 相关模块

- [ecs](./ecs.md) - ECS 核心
- [asset](./asset.md) - 资源管理
- [reflection](./reflection.md) - 反射系统
