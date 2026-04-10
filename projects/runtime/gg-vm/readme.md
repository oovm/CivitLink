# gg-vm

**GG Game Engine 的虚拟机系统，负责执行游戏脚本和逻辑。**

## 📋 模块简介

gg-vm 是 GG Game Engine 的虚拟机系统，负责执行游戏脚本和逻辑，为游戏提供脚本执行环境。

## ✨ 核心功能

- **虚拟机执行**：执行脚本代码和字节码
- **内存管理**：管理虚拟机的内存使用
- **垃圾回收**：自动回收未使用的内存
- **函数调用**：支持函数调用和返回
- **异常处理**：处理运行时异常
- **性能优化**：优化虚拟机执行性能

## 🚀 使用方法

### 依赖添加

```toml
[dependencies]
gg-vm = { path = "projects/runtime/gg-vm" }
```

### 基础示例

```rust
use gg_vm::prelude::*;

fn main() {
    // 创建虚拟机
    let mut vm = VM::new();
    
    // 注册全局函数
    vm.register_function("print", |args| {
        if let Some(message) = args.get(0) {
            println!("{}", message);
        }
        Ok(Value::Null)
    });
    
    // 执行代码
    let code = r#"
        function main() {
            print("Hello, GG Game Engine!");
            return 42;
        }
        main();
    "#;
    
    match vm.execute(code) {
        Ok(result) => println!("Execution result: {:?}", result),
        Err(error) => println!("Execution error: {:?}", error),
    }
}
```

## 📦 依赖关系

- **gg-core**：核心功能和平台抽象
- **gg-error**：错误处理系统

## 📖 相关文档

- [虚拟机系统设计](../../../design/architecture/overview.md) - 了解虚拟机系统的设计理念