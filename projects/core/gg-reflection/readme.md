# gg-reflection

**GG Game Engine 的反射系统，提供运行时类型信息和动态类型操作。**

## 📋 模块简介

gg-reflection 是 GG Game Engine 的反射系统，提供运行时类型信息和动态类型操作，为编辑器和序列化系统提供支持。

## ✨ 核心功能

- **运行时类型信息**：获取类型的名称、字段、方法等信息
- **动态类型操作**：支持动态创建对象、调用方法、访问字段
- **类型注册**：支持自定义类型的注册和管理
- **序列化支持**：为序列化和反序列化提供基础
- **编辑器集成**：为编辑器提供类型信息和属性编辑支持

## 🚀 使用方法

### 依赖添加

```toml
[dependencies]
gg-reflection = { path = "projects/core/gg-reflection" }
```

### 基础示例

```rust
use gg_reflection::prelude::*;

// 定义可反射类型
#[derive(Reflect)]
struct Player {
    name: String,
    health: f32,
    position: (f32, f32),
}

impl Player {
    fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            health: 100.0,
            position: (0.0, 0.0),
        }
    }
    
    fn take_damage(&mut self, amount: f32) {
        self.health -= amount;
    }
}

fn main() {
    // 获取类型信息
    let type_info = TypeInfo::of::<Player>();
    println!("Type name: {}", type_info.name());
    
    // 创建实例
    let mut player = Player::new("Alice");
    
    // 动态访问字段
    let health_field = type_info.field("health").unwrap();
    let health_value = health_field.get(&player).unwrap();
    println!("Player health: {:?}", health_value);
    
    // 动态调用方法
    let take_damage_method = type_info.method("take_damage").unwrap();
    take_damage_method.invoke(&mut player, &[Value::from(20.0)]).unwrap();
    println!("Player health after damage: {}", player.health);
}
```

## 📦 依赖关系

- **gg-core**：核心功能和平台抽象
- **gg-error**：错误处理系统

## 📖 相关文档

- [反射系统设计](../../../design/modules/reflection.md) - 了解反射系统的设计理念