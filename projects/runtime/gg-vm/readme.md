# GG VM

GG 引擎字节码虚拟机模块，执行 gg-ir 生成的中间表示指令。

## 功能

- 虚拟机执行环境
- 指令解析和执行
- 宿主接口交互
- 错误处理

## 使用示例

```rust
use gg_vm::{Vm, Host, VmResult};
use gg_ir::{IrModule, IrFunction, OpCode, IrValue};

// 实现宿主接口
struct ExampleHost;
impl Host for ExampleHost {
    fn spawn_entity(&mut self) -> u64 {
        1
    }

    fn despawn_entity(&mut self, _entity_id: u64) {
        // 实现实体销毁逻辑
    }

    fn add_component(&mut self, _entity_id: u64, _component_type: &str, _value: IrValue) {
        // 实现添加组件逻辑
    }

    fn get_component_field(
        &mut self,
        _entity_id: u64,
        _component_type: &str,
        _field: &str,
    ) -> Option<IrValue> {
        Some(IrValue::Null)
    }

    fn set_component_field(
        &mut self,
        _entity_id: u64,
        _component_type: &str,
        _field: &str,
        _value: IrValue,
    ) {
        // 实现设置组件字段逻辑
    }

    fn call_host_function(&mut self, _name: &str, _args: Vec<IrValue>) -> Option<IrValue> {
        Some(IrValue::Null)
    }
}

// 创建 IR 模块
let mut module = IrModule::new("test");

// 创建函数
let function = IrFunction {
    name: "main".to_string(),
    param_count: 0,
    local_count: 0,
    instructions: vec![
        OpCode::LoadNull,
        OpCode::Return,
    ],
};

// 添加函数到模块
module.add_function(function);

// 创建虚拟机和宿主
let mut vm = Vm::new();
let mut host = ExampleHost;

// 执行函数
let result = vm.execute(&module, "main", &mut host);
println!("执行结果: {:?}", result);
```
