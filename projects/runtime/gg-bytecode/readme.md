# gg-bytecode

**GG Game Engine 的字节码系统，负责处理和执行字节码。**

## 📋 模块简介

gg-bytecode 是 GG Game Engine 的字节码系统，负责处理和执行字节码，为脚本系统提供底层支持。

## ✨ 核心功能

- **字节码格式**：定义和解析字节码格式
- **解释器**：执行字节码指令
- **字节码读取**：读取编译后的字节码
- **字节码写入**：生成和写入字节码
- **主机接口**：提供字节码与主机系统的交互
- **性能优化**：优化字节码执行性能

## 🚀 使用方法

### 依赖添加

```toml
[dependencies]
gg-bytecode = { path = "projects/runtime/gg-bytecode" }
```

### 基础示例

```rust
use gg_bytecode::prelude::*;

fn main() {
    // 创建字节码读取器
    let mut reader = BytecodeReader::new();
    
    // 读取字节码文件
    let bytecode = reader.read_file("script.bytecode").unwrap();
    
    // 创建解释器
    let mut interpreter = Interpreter::new();
    
    // 执行字节码
    interpreter.execute(&bytecode).unwrap();
}
```

## 📦 依赖关系

- **gg-core**：核心功能和平台抽象
- **gg-error**：错误处理系统

## 📖 相关文档

- [字节码系统设计](../../../design/architecture/overview.md) - 了解字节码系统的设计理念