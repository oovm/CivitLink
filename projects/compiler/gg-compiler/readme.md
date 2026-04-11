# gg-compiler

**GG Game Engine 的编译器核心模块，提供编译器的基础功能和流水线。**

## 📋 模块简介

gg-compiler 是 GG Game Engine 的编译器核心模块，提供编译器的基础功能和流水线，为其他编译器模块提供统一的接口和工具。

## ✨ 核心功能

- **编译流水线**：定义和管理编译过程的各个阶段
- **上下文管理**：管理编译过程中的上下文信息
- **转换器**：支持代码和中间表示的转换
- **产物管理**：管理编译生成的产物
- **错误处理**：提供编译错误的处理和报告

## 🚀 使用方法

### 依赖添加

```toml
[dependencies]
gg-compiler = { path = "projects/compiler/gg-compiler" }
```

### 基础示例

```rust
use gg_compiler_core::prelude::*;

fn main() {
    // 创建编译上下文
    let mut context = CompileContext::new();
    
    // 创建编译流水线
    let mut pipeline = Pipeline::new();
    
    // 添加转换器
    pipeline.add_transformer(Box::new(MyTransformer));
    
    // 执行编译
    let result = pipeline.compile(&mut context);
    
    match result {
        Ok(artifact) => println!("Compilation successful: {:?}", artifact),
        Err(error) => println!("Compilation error: {:?}", error),
    }
}

// 自定义转换器
struct MyTransformer;
impl Transformer for MyTransformer {
    fn transform(&self, context: &mut CompileContext) -> Result<()> {
        // 执行转换逻辑
        Ok(())
    }
}
```

## 📦 依赖关系

- **gg-core**：核心功能和平台抽象
- **gg-error**：错误处理系统

## 📖 相关文档

- [编译器设计](../../../design/architecture/overview.md) - 了解编译器系统的设计理念