# gg-compiler-script

**GG Game Engine 的脚本编译器，负责编译游戏脚本。**

## 📋 模块简介

gg-compiler-script 是 GG Game Engine 的脚本编译器，负责编译游戏脚本（如 Valkyrie 脚本），将脚本代码转换为可执行的字节码或中间表示。

## ✨ 核心功能

- **脚本解析**：解析脚本代码，生成抽象语法树
- **语义分析**：分析脚本的语义，检查类型和作用域
- **代码生成**：生成字节码或中间表示
- **增量编译**：支持增量编译，提高编译速度
- **错误处理**：提供详细的编译错误信息

## 🚀 使用方法

### 依赖添加

```toml
[dependencies]
gg-compiler-script = { path = "projects/compiler/gg-compiler-script" }
```

### 基础示例

```rust
use gg_compiler_script::prelude::*;

fn main() {
    // 创建脚本编译器
    let mut compiler = ScriptCompiler::new();
    
    // 编译脚本
    let script = r#"
        function main() {
            print("Hello, GG Game Engine!");
        }
    "#;
    
    match compiler.compile(script) {
        Ok(bytecode) => println!("Script compiled successfully: {:?}", bytecode),
        Err(error) => println!("Script compilation error: {:?}", error),
    }
}
```

## 📦 依赖关系

- **gg-compiler-core**：编译器核心功能
- **gg-core**：核心功能和平台抽象
- **gg-error**：错误处理系统

## 📖 相关文档

- [脚本系统设计](../../../design/architecture/overview.md) - 了解脚本系统的设计理念