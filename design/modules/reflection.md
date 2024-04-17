# Reflection 模块

## 概述

Reflection 模块提供动态类型查询和属性编辑功能，基于 `bevy_reflect` 实现。

## 核心概念

### 反射（Reflection）
反射允许程序在运行时检查和操作类型信息。

### 反射注册表（ReflectionRegistry）
反射注册表管理所有可反射类型的信息。

### 属性编辑器（PropertyEditor）
属性编辑器提供动态编辑可反射类型属性的能力。

## 核心类型

### ReflectionRegistry
反射注册表，用于管理所有可反射类型。

```rust
pub struct ReflectionRegistry {
    inner: TypeRegistry,
}
```

**主要方法：**

- `new()` - 创建新的反射注册表
- `register<T>()` - 注册一个可反射类型
- `get()` - 获取类型注册表的引用
- `get_mut()` - 获取类型注册表的可变引用
- `get_type_info(type_id)` - 根据 TypeId 获取类型信息
- `get_registration(type_id)` - 根据 TypeId 获取类型注册信息
- `is_registered(type_id)` - 检查类型是否已注册

### PropertyEditor
属性编辑器 trait，用于动态编辑可反射类型的属性。

```rust
pub trait PropertyEditor {
    fn editable_properties(&self) -> Vec<PropertyInfo>;
    fn get_property(&self, name: &str) -> Option<&dyn PartialReflect>;
    fn set_property(&mut self, name: &str, value: Box<dyn PartialReflect>) -> anyhow::Result<()>;
}
```

### PropertyInfo
属性信息。

```rust
pub struct PropertyInfo {
    pub name: String,
    pub type_name: &'static str,
    pub writable: bool,
    pub description: Option<String>,
}
```

**主要方法：**

- `new(name, type_name, writable)` - 创建新的属性信息
- `with_description(description)` - 设置属性描述

### StructPropertyEditor<T>
为结构体实现的属性编辑器。

```rust
pub struct StructPropertyEditor<T: Struct> {
    value: T,
}
```

**主要方法：**

- `new(value)` - 创建新的结构体属性编辑器
- `get()` - 获取内部值的引用
- `get_mut()` - 获取内部值的可变引用
- `into_inner()` - 消费编辑器并返回内部值

## 使用示例

### 注册可反射类型

```rust
use gg_reflection::prelude::*;

#[derive(Reflect, Default)]
struct Player {
    name: String,
    health: f32,
    level: u32,
}

fn main() {
    let mut registry = ReflectionRegistry::new();
    
    // 注册类型
    registry.register::<Player>();
    
    // 检查类型是否已注册
    let type_id = TypeId::of::<Player>();
    assert!(registry.is_registered(type_id));
}
```

### 属性编辑器

```rust
use gg_reflection::prelude::*;

#[derive(Reflect, Default)]
struct Transform {
    x: f32,
    y: f32,
    rotation: f32,
}

fn main() {
    let transform = Transform::default();
    let mut editor = StructPropertyEditor::new(transform);
    
    // 获取可编辑属性
    let properties = editor.editable_properties();
    for prop in properties {
        println!("Property: {} ({})", prop.name, prop.type_name);
    }
    
    // 设置属性
    let new_x = 10.0f32;
    editor.set_property("x", Box::new(new_x)).unwrap();
    
    // 获取属性
    if let Some(x) = editor.get_property("x") {
        println!("x: {:?}", x);
    }
    
    // 获取修改后的值
    let modified = editor.into_inner();
    println!("Modified x: {}", modified.x);
}
```

### 与 GameWorld 集成

```rust
use gg_world::prelude::*;
use gg_reflection::prelude::*;

#[derive(Component, Reflect)]
struct Character {
    name: String,
    hp: i32,
}

fn main() {
    let mut world = GameWorld::new("Game".to_string());
    
    // 注册类型
    world.reflection_registry.register::<Character>();
    
    // 生成实体
    let entity = world.spawn()
        .insert(Character {
            name: "Hero".to_string(),
            hp: 100,
        })
        .id();
}
```

## 相关模块

- [world](./world.md) - 世界管理
- [ecs](./ecs.md) - ECS 核心
- [schedule](./schedule.md) - 调度器
