# gg-ir

**GG Game Engine 的中间表示系统，负责代码的优化和转换。**

## 📋 模块简介

gg-ir 是 GG Game Engine 的中间表示系统，负责代码的优化和转换，为编译器和解释器提供中间代码表示。

## ✨ 核心功能

- **中间表示**：定义和管理中间代码表示
- **优化 passes**：提供各种代码优化 passes
- **常量折叠**：优化常量表达式
- **死代码消除**：移除未使用的代码
- **代码转换**：支持代码的转换和重构
- **适配器**：适配不同的代码格式

## 🚀 使用方法

### 依赖添加

```toml
[dependencies]
gg-ir = { path = "projects/runtime/gg-ir" }
```

### 基础示例

```rust
use gg_ir::prelude::*;

fn main() {
    // 创建中间表示模块
    let mut module = Module::new();
    
    // 添加函数
    let function = Function::new("main");
    module.add_function(function);
    
    // 应用优化 passes
    let mut optimizer = Optimizer::new();
    optimizer.add_pass(Box::new(ConstantFoldPass));
    optimizer.add_pass(Box::new(DeadCodeEliminationPass));
    
    // 优化模块
    optimizer.optimize(&mut module);
    
    // 打印优化后的模块
    println!("{:?}", module);
}
```

## 📦 依赖关系

- **gg-core**：核心功能和平台抽象
- **gg-error**：错误处理系统

## 📖 相关文档

- [中间表示系统设计](../../../design/architecture/overview.md) - 了解中间表示系统的设计理念