use std::collections::HashMap;

use gg_bytecode::format::{BytecodeFunction, BytecodeInstruction};
use gg_ir::{IrFunction, IrModule, IrValue, OpCode};
use gg_vm::{BytecodeModule, BytecodeValue, Host, Vm, VmResult};

/// 测试用宿主实现
struct TestHost;

impl Host for TestHost {
    fn spawn_entity(&mut self) -> u64 {
        0
    }

    fn despawn_entity(&mut self, _entity_id: u64) {}

    fn add_component(&mut self, _entity_id: u64, _component_type: &str, _value: BytecodeValue) {}

    fn get_component_field(&mut self, _entity_id: u64, _component_type: &str, _field: &str) -> Option<BytecodeValue> {
        None
    }

    fn set_component_field(&mut self, _entity_id: u64, _component_type: &str, _field: &str, _value: BytecodeValue) {}

    fn call_host_function(&mut self, _name: &str, _args: Vec<BytecodeValue>) -> Option<BytecodeValue> {
        None
    }
}

/// 测试 Vm::new() 初始状态
#[test]
fn test_vm_new() {
    let vm = Vm::new();
    assert!(vm.stack().is_empty());
    assert!(vm.call_stack().is_empty());
    assert!(!vm.is_running());
}

/// 测试通过 BytecodeModule 执行基本函数
#[test]
fn test_vm_execute_basic() {
    let module = BytecodeModule {
        name: "test".to_string(),
        version: 1,
        constants: vec![BytecodeValue::Int(42)],
        int_constants: vec![42],
        float_constants: vec![],
        string_constants: vec![],
        entity_constants: vec![],
        bool_constants: vec![],
        string_pool: vec![],
        functions: vec![BytecodeFunction {
            name: "main".to_string(),
            param_count: 0,
            local_count: 0,
            local_names: vec![],
            instructions: vec![BytecodeInstruction::LoadConst { index: 0 }, BytecodeInstruction::Return],
        }],
        debug_info: None,
        entry_points: vec![],
        function_index: HashMap::new(),
    };

    let mut vm = Vm::new();
    let mut host = TestHost;
    let result = vm.execute(&module, "main", &mut host);

    match result {
        VmResult::Return(BytecodeValue::Int(42)) => {}
        other => panic!("期望返回 Int(42)，实际: {:?}", other),
    }
}

/// 测试通过 IrModule 执行函数
#[test]
fn test_vm_execute_ir() {
    let module = IrModule {
        name: "test".to_string(),
        constants: vec![IrValue::Int(42)],
        string_pool: vec![],
        functions: vec![IrFunction {
            name: "main".to_string(),
            param_count: 0,
            local_count: 0,
            local_names: vec![],
            instructions: vec![OpCode::LoadConst(0), OpCode::Return],
            is_entry: false,
            target: None,
        }],
        entry_points: vec![],
        target_platform: None,
    };

    let mut vm = Vm::new();
    let mut host = TestHost;
    let result = vm.execute_ir(&module, "main", &mut host);

    match result {
        VmResult::Return(BytecodeValue::Int(42)) => {}
        other => panic!("期望返回 Int(42)，实际: {:?}", other),
    }
}

/// 测试执行栈访问
#[test]
fn test_vm_stack_access() {
    let module = BytecodeModule {
        name: "test".to_string(),
        version: 1,
        constants: vec![BytecodeValue::Int(42)],
        int_constants: vec![42],
        float_constants: vec![],
        string_constants: vec![],
        entity_constants: vec![],
        bool_constants: vec![],
        string_pool: vec![],
        functions: vec![BytecodeFunction {
            name: "main".to_string(),
            param_count: 0,
            local_count: 0,
            local_names: vec![],
            instructions: vec![
                BytecodeInstruction::LoadConst { index: 0 },
                BytecodeInstruction::Dup,
                BytecodeInstruction::Return,
            ],
        }],
        debug_info: None,
        entry_points: vec![],
        function_index: HashMap::new(),
    };

    let mut vm = Vm::new();
    let mut host = TestHost;
    let result = vm.execute(&module, "main", &mut host);

    match result {
        VmResult::Return(BytecodeValue::Int(42)) => {}
        other => panic!("期望返回 Int(42)，实际: {:?}", other),
    }

    assert_eq!(vm.stack().len(), 1);
    assert_eq!(vm.stack()[0], BytecodeValue::Int(42));
}

/// 测试调用栈访问
#[test]
fn test_vm_call_stack() {
    let module = BytecodeModule {
        name: "test".to_string(),
        version: 1,
        constants: vec![BytecodeValue::Int(10), BytecodeValue::String("helper".to_string())],
        int_constants: vec![10],
        float_constants: vec![],
        string_constants: vec!["helper".to_string()],
        entity_constants: vec![],
        bool_constants: vec![],
        string_pool: vec![],
        functions: vec![
            BytecodeFunction {
                name: "helper".to_string(),
                param_count: 0,
                local_count: 0,
                local_names: vec![],
                instructions: vec![BytecodeInstruction::LoadConst { index: 0 }, BytecodeInstruction::Return],
            },
            BytecodeFunction {
                name: "main".to_string(),
                param_count: 0,
                local_count: 0,
                local_names: vec![],
                instructions: vec![
                    BytecodeInstruction::LoadConst { index: 1 },
                    BytecodeInstruction::Call { arg_count: 0 },
                    BytecodeInstruction::Return,
                ],
            },
        ],
        debug_info: None,
        entry_points: vec![],
        function_index: HashMap::new(),
    };

    let mut vm = Vm::new();
    let mut host = TestHost;
    let result = vm.execute(&module, "main", &mut host);

    match result {
        VmResult::Return(BytecodeValue::Int(10)) => {}
        other => panic!("期望返回 Int(10)，实际: {:?}", other),
    }

    assert!(vm.call_stack().is_empty());
}
