# gg-script

**GG Game Engine 的脚本解析和执行模块，负责处理游戏脚本的解析和执行。**

## 📋 模块简介

gg-script 是 GG Game Engine 的脚本解析和执行模块，负责处理游戏脚本的解析和执行，为游戏提供脚本化的逻辑处理能力。

## ✨ 核心功能

- **脚本解析**：解析脚本代码，生成抽象语法树
- **脚本执行**：执行脚本代码，处理游戏逻辑
- **脚本编译**：将脚本编译为可执行形式
- **错误处理**：提供详细的脚本执行错误信息
- **与引擎集成**：与游戏引擎的其他部分集成

## 🚀 使用方法

### 依赖添加

```toml
[dependencies]
gg-script = { path = "projects/compiler/gg-script" }
```

### 基础示例

```rust
use gg_script::prelude::*;

fn main() {
    // 创建脚本引擎
    let mut engine = ScriptEngine::new();
    
    // 注册全局函数
    engine.register_function("print", |args| {
        if let Some(message) = args.get(0) {
            println!("{}", message);
        }
        Ok(Value::Null)
    });
    
    // 执行脚本
    let script = r#"
        function main() {
            print("Hello, GG Game Engine!");
            return 42;
        }
    "#;
    
    match engine.execute(script) {
        Ok(result) => println!("Script executed successfully: {:?}", result),
        Err(error) => println!("Script execution error: {:?}", error),
    }
}
```

## 📦 依赖关系

- **gg-core**：核心功能和平台抽象
- **gg-error**：错误处理系统

## 📖 相关文档

- [脚本系统设计](../../../design/architecture/overview.md) - 了解脚本系统的设计理念