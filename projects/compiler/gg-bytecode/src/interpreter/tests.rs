use super::*;
use crate::{format::{BytecodeFunction, BytecodeInstruction, BytecodeModule, BytecodeValue}, host::Host};
use std::collections::HashMap;

struct TestHost {
    entities: Vec<u64>,
    log: Vec<String>,
}

impl TestHost {
    fn new() -> Self {
        Self { entities: Vec::new(), log: Vec::new() }
    }
}

impl Host for TestHost {
    fn spawn_entity(&mut self) -> u64 {
        let id = self.entities.len() as u64;
        self.entities.push(id);
        id
    }

    fn despawn_entity(&mut self, _entity_id: u64) {}

    fn add_component(&mut self, entity_id: u64, component_type: &str, value: BytecodeValue) {
        self.log.push(format!("add_component({}, {}, {:?})", entity_id, component_type, value));
    }

    fn get_component_field(&mut self, _entity_id: u64, _component_type: &str, _field: &str) -> Option<BytecodeValue> {
        None
    }

    fn set_component_field(&mut self, entity_id: u64, component_type: &str, field: &str, value: BytecodeValue) {
        self.log.push(format!("set_component_field({}, {}, {}, {:?})", entity_id, component_type, field, value));
    }

    fn call_host_function(&mut self, name: &str, args: Vec<BytecodeValue>) -> Option<BytecodeValue> {
        self.log.push(format!("call_host_function({}, {:?})", name, args));
        match name {
            "print" => None,
            _ => None,
        }
    }
}

fn make_simple_module() -> BytecodeModule {
    BytecodeModule {
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
            instructions: vec![BytecodeInstruction::LoadConst { index: 0 }, BytecodeInstruction::Return],
            local_names: vec![],
        }],
        debug_info: None,
        entry_points: vec![],
        function_index: HashMap::new(),
    }
}

#[test]
fn test_function_cache_populated_after_execute() {
    let module = make_simple_module();
    let mut interpreter = BytecodeInterpreter::new();
    let mut host = TestHost::new();

    assert!(interpreter.function_cache.is_empty());

    interpreter.execute(&module, "main", &mut host);

    assert!(interpreter.function_cache.contains_key("main"));
    assert_eq!(interpreter.function_cache.get("main"), Some(&0));
}

#[test]
fn test_function_cache_uses_module_index() {
    let mut module = make_simple_module();
    module.build_function_index();

    let mut interpreter = BytecodeInterpreter::new();
    let mut host = TestHost::new();

    interpreter.execute(&module, "main", &mut host);

    assert!(interpreter.function_cache.contains_key("main"));
    assert_eq!(interpreter.function_cache.get("main"), Some(&0));
}

#[test]
fn test_function_cache_fallback_linear_search() {
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
                instructions: vec![BytecodeInstruction::LoadConst { index: 0 }, BytecodeInstruction::Return],
                local_names: vec![],
            },
            BytecodeFunction {
                name: "main".to_string(),
                param_count: 0,
                local_count: 0,
                instructions: vec![
                    BytecodeInstruction::LoadConst { index: 1 },
                    BytecodeInstruction::Call { arg_count: 0 },
                    BytecodeInstruction::Return,
                ],
                local_names: vec![],
            },
        ],
        debug_info: None,
        entry_points: vec![],
        function_index: HashMap::new(),
    };

    let mut interpreter = BytecodeInterpreter::new();
    let mut host = TestHost::new();

    let result = interpreter.execute(&module, "main", &mut host);

    match result {
        InterpretResult::Return(BytecodeValue::Int(10)) => {}
        other => panic!("期望返回 Int(10)，实际: {:?}", other),
    }

    assert!(interpreter.function_cache.contains_key("main"));
    assert!(interpreter.function_cache.contains_key("helper"));
}

#[test]
fn test_tail_call_does_not_grow_call_stack() {
    let module = BytecodeModule {
        name: "test".to_string(),
        version: 1,
        constants: vec![BytecodeValue::Int(0), BytecodeValue::String("loop_fn".to_string())],
        int_constants: vec![0, 1, 3],
        float_constants: vec![],
        string_constants: vec!["loop_fn".to_string()],
        entity_constants: vec![],
        bool_constants: vec![],
        string_pool: vec![],
        functions: vec![
            BytecodeFunction {
                name: "loop_fn".to_string(),
                param_count: 1,
                local_count: 1,
                instructions: vec![
                    BytecodeInstruction::LoadLocal { index: 0 },
                    BytecodeInstruction::LoadConstInt { index: 0 },
                    BytecodeInstruction::Eq,
                    BytecodeInstruction::JumpIfFalse { address: 6 },
                    BytecodeInstruction::LoadConst { index: 0 },
                    BytecodeInstruction::Return,
                    BytecodeInstruction::LoadLocal { index: 0 },
                    BytecodeInstruction::LoadConstInt { index: 1 },
                    BytecodeInstruction::Sub,
                    BytecodeInstruction::LoadConst { index: 1 },
                    BytecodeInstruction::TailCall(1),
                ],
                local_names: vec![],
            },
            BytecodeFunction {
                name: "main".to_string(),
                param_count: 0,
                local_count: 0,
                instructions: vec![
                    BytecodeInstruction::LoadConstInt { index: 2 },
                    BytecodeInstruction::LoadConst { index: 1 },
                    BytecodeInstruction::Call { arg_count: 1 },
                    BytecodeInstruction::Return,
                ],
                local_names: vec![],
            },
        ],
        debug_info: None,
        entry_points: vec![],
        function_index: HashMap::new(),
    };

    let mut interpreter = BytecodeInterpreter::new();
    let mut host = TestHost::new();

    let result = interpreter.execute(&module, "main", &mut host);

    match result {
        InterpretResult::Return(BytecodeValue::Int(0)) => {}
        other => panic!("期望返回 Int(0)，实际: {:?}", other),
    }

    assert!(interpreter.call_stack.len() <= 2);
}

#[test]
fn test_tail_call_frame_is_marked() {
    let frame = InterpreterFrame {
        function_index: 0,
        function_name: "test".to_string(),
        ip: 0,
        locals: vec![],
        stack_base: 0,
        is_tail_call: true,
    };
    assert!(frame.is_tail_call);

    let default_frame = InterpreterFrame {
        function_index: 0,
        function_name: "test".to_string(),
        ip: 0,
        locals: vec![],
        stack_base: 0,
        is_tail_call: false,
    };
    assert!(!default_frame.is_tail_call);
}

#[test]
fn test_stack_pre_allocation() {
    let interpreter = BytecodeInterpreter::new();
    assert!(interpreter.stack.capacity() >= 256);
    assert!(interpreter.call_stack.capacity() >= 32);
}

#[test]
fn test_basic_execution_with_new_frame_structure() {
    let module = make_simple_module();
    let mut interpreter = BytecodeInterpreter::new();
    let mut host = TestHost::new();

    let result = interpreter.execute(&module, "main", &mut host);

    match result {
        InterpretResult::Return(BytecodeValue::Int(42)) => {}
        other => panic!("期望返回 Int(42)，实际: {:?}", other),
    }

    if let Some(frame) = interpreter.call_stack.last() {
        assert_eq!(frame.function_index, 0);
        assert_eq!(frame.function_name, "main");
        assert!(!frame.is_tail_call);
    }
}

#[test]
fn test_function_call_with_new_frame_structure() {
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
                instructions: vec![BytecodeInstruction::LoadConst { index: 0 }, BytecodeInstruction::Return],
                local_names: vec![],
            },
            BytecodeFunction {
                name: "main".to_string(),
                param_count: 0,
                local_count: 0,
                instructions: vec![
                    BytecodeInstruction::LoadConst { index: 1 },
                    BytecodeInstruction::Call { arg_count: 0 },
                    BytecodeInstruction::Return,
                ],
                local_names: vec![],
            },
        ],
        debug_info: None,
        entry_points: vec![],
        function_index: HashMap::new(),
    };

    let mut interpreter = BytecodeInterpreter::new();
    let mut host = TestHost::new();

    let result = interpreter.execute(&module, "main", &mut host);

    match result {
        InterpretResult::Return(BytecodeValue::Int(10)) => {}
        other => panic!("期望返回 Int(10)，实际: {:?}", other),
    }
}

#[test]
fn test_frame_function_index_access() {
    let module = make_simple_module();
    let mut interpreter = BytecodeInterpreter::new();
    let mut host = TestHost::new();

    interpreter.execute(&module, "main", &mut host);

    if let Some(frame) = interpreter.call_stack.last() {
        assert_eq!(frame.function_index, 0);
        let function = &module.functions[frame.function_index];
        assert_eq!(function.name, "main");
    }
}