# GG IR

GG 引擎 IR 模块，提供中间表示和指令集定义。

## 功能

- IR 值类型定义
- IR 操作码定义
- IR 函数和模块管理

## 使用示例

```rust
use gg_ir::{IrModule, IrFunction, OpCode, IrValue};

// 创建 IR 模块
let mut module = IrModule::new("test");

// 添加常量
let str_index = module.add_constant(IrValue::String(0));

// 创建函数
let mut function = IrFunction {
    name: "main".to_string(),
    param_count: 0,
    local_count: 0,
    instructions: vec![
        OpCode::LoadConst(str_index),
        OpCode::Return,
    ],
};

// 添加函数到模块
module.add_function(function);

// 查找函数
if let Some(func) = module.find_function("main") {
    println!("Found function: {}", func.name);
}
```
