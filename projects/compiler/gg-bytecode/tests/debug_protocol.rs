use std::{cell::RefCell, rc::Rc};

use gg_bytecode::{
    debug_info::{DebugInfo, SourceLocation},
    debug_protocol::BasicDebugController,
    format::{BytecodeFunction, BytecodeInstruction, BytecodeModule, BytecodeValue},
    host::Host,
    interpreter::{BytecodeInterpreter, InterpretResult},
};

struct MockHost;

impl Host for MockHost {
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

#[test]
fn test_set_debug_controller() {
    let mut interpreter = BytecodeInterpreter::new();
    assert!(!interpreter.is_debug_paused());

    let controller = Rc::new(RefCell::new(BasicDebugController::new()));
    interpreter.set_debug_controller(Some(controller));

    let mut module = BytecodeModule::new("test");
    module.add_function(BytecodeFunction {
        name: "main".to_string(),
        param_count: 0,
        local_count: 0,
        instructions: vec![BytecodeInstruction::Return],
        local_names: vec![],
    });

    let mut host = MockHost;
    let result = interpreter.execute(&module, "main", &mut host);
    assert!(matches!(result, InterpretResult::Return(_)));
    assert!(!interpreter.is_debug_paused());

    interpreter.set_debug_controller(None);
}

#[test]
fn test_breakpoint_hit() {
    let mut module = BytecodeModule::new("test");
    module.add_function(BytecodeFunction {
        name: "main".to_string(),
        param_count: 0,
        local_count: 0,
        instructions: vec![BytecodeInstruction::LoadNull, BytecodeInstruction::Return],
        local_names: vec![],
    });

    let mut debug_info = DebugInfo::new();
    debug_info.add_entry(0, SourceLocation::new("test.gg", 1, 1));
    debug_info.add_entry(1, SourceLocation::new("test.gg", 2, 1));
    module.debug_info = Some(debug_info);

    let controller = Rc::new(RefCell::new(BasicDebugController::new()));
}
