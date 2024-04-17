# gg-compiler-aot

**GG Game Engine 的 AOT（ Ahead-of-Time）编译器，负责预编译代码以提高运行性能。**

## 📋 模块简介

gg-compiler-aot 是 GG Game Engine 的 AOT（Ahead-of-Time）编译器，负责预编译代码以提高运行性能，将高级代码转换为机器代码或优化的中间表示。

## ✨ 核心功能

- **AOT 编译**：将代码预编译为机器代码或优化的中间表示
- **后端支持**：支持不同的编译后端和目标平台
- **优化**：提供代码优化和性能分析
- **目标管理**：管理编译目标和配置
- **错误处理**：提供详细的编译错误信息

## 🚀 使用方法

### 依赖添加

```toml
[dependencies]
gg-compiler-aot = { path = "projects/compiler/gg-compiler-aot" }
```

### 基础示例

```rust
use gg_compiler_aot::prelude::*;

fn main() {
    // 创建 AOT 编译器
    let mut compiler = AotCompiler::new();
    
    // 设置编译目标
    let target = Target::new("x86_64-unknown-windows-msvc");
    compiler.set_target(target);
    
    // 编译代码
    let code = r#"
        fn main() {
            println!("Hello, GG Game Engine!");
        }
    "#;
    
    match compiler.compile(code) {
        Ok(artifact) => println!("AOT compilation successful: {:?}", artifact),
        Err(error) => println!("AOT compilation error: {:?}", error),
    }
}
```

## 📦 依赖关系

- **gg-compiler-core**：编译器核心功能
- **gg-core**：核心功能和平台抽象
- **gg-error**：错误处理系统

## 📖 相关文档

- [编译器设计](../../../design/architecture/overview.md) - 了解编译器系统的设计理念