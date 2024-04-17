# gg-macros

**GG Game Engine 的宏模块，提供各种实用的宏定义。**

## 📋 模块简介

gg-macros 是 GG Game Engine 的宏模块，提供了各种实用的宏定义，简化代码编写和提高开发效率。

## ✨ 核心功能

- **派生宏**：为各种类型自动生成代码
- **属性宏**：为结构体、枚举和函数添加额外的功能
- **过程宏**：实现更复杂的代码生成逻辑

## 🚀 使用方法

### 依赖添加

```toml
[dependencies]
gg-macros = { path = "projects/core/gg-macros" }
```

### 基础示例

```rust
use gg_macros::MyMacro;

#[derive(MyMacro)]
struct MyStruct {
    field1: String,
    field2: i32,
}

fn main() {
    let my_struct = MyStruct {
        field1: "Hello".to_string(),
        field2: 42,
    };
    
    println!("MyStruct: {:?}", my_struct);
}
```

## 📦 依赖关系

- **syn**：Rust 语法分析库
- **quote**：Rust 代码生成库
- **proc-macro2**：过程宏支持库

## 📖 相关文档

- [宏系统设计](../../../design/architecture/patterns.md) - 了解宏系统的设计理念
- [核心模块文档](../../../design/modules/world.md) - 深入了解核心模块的功能