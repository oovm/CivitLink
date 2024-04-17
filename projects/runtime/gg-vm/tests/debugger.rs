use gg_bytecode::{
    DebugProtocol, Host, SourceLocation,
    debug_info::DebugInfo,
    debug_protocol::DebugValue,
    format::{BytecodeFunction, BytecodeInstruction, BytecodeModule, BytecodeValue},
};
use gg_vm::{Vm, debugger::VmDebugger};
use std::{cell::RefCell, rc::Rc};

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

#[test]
fn test_debugger_new() {
    let debugger = VmDebugger::new();
    assert!(debugger.list_breakpoints().is_empty());
    assert!(!debugger.is_paused());
    assert!(!debugger.is_attached());
}

#[test]
fn test_debugger_default() {
    let debugger = VmDebugger::default();
    assert!(debugger.list_breakpoints().is_empty());
    assert!(!debugger.is_attached());
}

#[test]
fn test_add_breakpoint() {
    let mut debugger = VmDebugger::new();
    let id1 = debugger.add_breakpoint("test.gg", 10, None);
    let id2 = debugger.add_breakpoint("main.gg", 5, Some("x > 0".to_string()));
    let breakpoints = debugger.list_breakpoints();
    assert_eq!(breakpoints.len(), 2);
}

#[test]
fn test_add_breakpoint_properties() {
    let mut debugger = VmDebugger::new();
    let id = debugger.add_breakpoint("test.gg", 42, Some("x == 1".to_string()));
    let breakpoints = debugger.list_breakpoints();
    assert_eq!(breakpoints.len(), 1);
    let bp = breakpoints[0];
    assert_eq!(bp.id, id);
    assert_eq!(bp.file, "test.gg");
    assert_eq!(bp.line, 42);
    assert_eq!(bp.condition.as_deref(), Some("x == 1"));
}

#[test]
fn test_add_breakpoint_no_condition() {
    let mut debugger = VmDebugger::new();
    let id = debugger.add_breakpoint("test.gg", 10, None);
    let bp = debugger.list_breakpoints()[0];
    assert_eq!(bp.id, id);
    assert!(bp.condition.is_none());
}

#[test]
fn test_remove_breakpoint() {
    let mut debugger = VmDebugger::new();
    let id = debugger.add_breakpoint("test.gg", 10, None);
    assert_eq!(debugger.list_breakpoints().len(), 1);
    assert!(debugger.remove_breakpoint(id));
    assert!(debugger.list_breakpoints().is_empty());
}

#[test]
fn test_remove_breakpoint_not_found() {
    let mut debugger = VmDebugger::new();
    assert!(!debugger.remove_breakpoint(gg_bytecode::debug_protocol::BreakpointId(99)));
}

#[test]
fn test_remove_breakpoint_twice() {
    let mut debugger = VmDebugger::new();
    let id = debugger.add_breakpoint("test.gg", 10, None);
    assert!(debugger.remove_breakpoint(id));
    assert!(!debugger.remove_breakpoint(id));
}

#[test]
fn test_list_breakpoints() {
    let mut debugger = VmDebugger::new();
    debugger.add_breakpoint("a.gg", 1, None);
    debugger.add_breakpoint("b.gg", 2, Some("x".to_string()));
    debugger.add_breakpoint("c.gg", 3, None);
    assert_eq!(debugger.list_breakpoints().len(), 3);
}

#[test]
fn test_list_breakpoints_empty() {
    let debugger = VmDebugger::new();
    assert!(debugger.list_breakpoints().is_empty());
}

#[test]
fn test_evaluate_empty_expression() {
    let debugger = VmDebugger::new();
    let result = debugger.evaluate("");
    assert!(result.is_err());
}

#[test]
fn test_evaluate_integer_literal() {
    let debugger = VmDebugger::new();
    let result = debugger.evaluate("42");
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), DebugValue::Int(42));
}

#[test]
fn test_evaluate_float_literal() {
    let debugger = VmDebugger::new();
    let result = debugger.evaluate("3.14");
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), DebugValue::Float(3.14));
}

#[test]
fn test_evaluate_bool_literal() {
    let debugger = VmDebugger::new();
    assert_eq!(debugger.evaluate("true").unwrap(), DebugValue::Bool(true));
    assert_eq!(debugger.evaluate("false").unwrap(), DebugValue::Bool(false));
}

#[test]
fn test_evaluate_unknown_variable() {
    let debugger = VmDebugger::new();
    let result = debugger.evaluate("unknown_var");
    assert!(result.is_err());
}

#[test]
fn test_watch_add_remove() {
    use gg_vm::WatchId;
    let mut debugger = VmDebugger::new();
    let id = debugger.add_watch("x".to_string());
    assert_eq!(id, WatchId(0));
    let id2 = debugger.add_watch("y".to_string());
    assert_eq!(id2, WatchId(1));
    assert!(debugger.remove_watch(id));
    assert!(!debugger.remove_watch(id));
    assert!(debugger.remove_watch(id2));
}

#[test]
fn test_watch_evaluate() {
    let mut debugger = VmDebugger::new();
    debugger.add_watch("42".to_string());
    debugger.add_watch("unknown".to_string());
    let results = debugger.evaluate_watches();
    assert_eq!(results.len(), 2);
}

#[test]
fn test_attach() {
    let vm = Rc::new(RefCell::new(Vm::new()));
    let mut debugger = VmDebugger::new();
    assert!(!debugger.is_attached());
    debugger.attach(vm);
    assert!(debugger.is_attached());
}

#[test]
fn test_detach() {
    let vm = Rc::new(RefCell::new(Vm::new()));
    let mut debugger = VmDebugger::new();
    debugger.attach(vm);
    assert!(debugger.is_attached());
    debugger.detach();
    assert!(!debugger.is_attached());
}

#[test]
fn test_attach_syncs_breakpoints() {
    let mut debugger = VmDebugger::new();
    debugger.add_breakpoint("test.gg", 10, None);
    debugger.add_breakpoint("test.gg", 20, None);
    let vm = Rc::new(RefCell::new(Vm::new()));
    debugger.attach(vm);
    assert!(debugger.is_attached());
    assert_eq!(debugger.list_breakpoints().len(), 2);
}

#[test]
fn test_add_breakpoint_after_attach() {
    let vm = Rc::new(RefCell::new(Vm::new()));
    let mut debugger = VmDebugger::new();
    debugger.attach(vm);
    let id = debugger.add_breakpoint("test.gg", 10, None);
    assert_eq!(debugger.list_breakpoints().len(), 1);
}

#[test]
fn test_remove_breakpoint_after_attach() {
    let vm = Rc::new(RefCell::new(Vm::new()));
    let mut debugger = VmDebugger::new();
    debugger.attach(vm);
    let id = debugger.add_breakpoint("test.gg", 10, None);
    assert!(debugger.remove_breakpoint(id));
    assert!(debugger.list_breakpoints().is_empty());
}

#[test]
fn test_pause_sets_flag() {
    let vm = Rc::new(RefCell::new(Vm::new()));
    let mut debugger = VmDebugger::new();
    debugger.attach(vm);
    assert!(!debugger.is_paused());
    debugger.pause();
    assert!(debugger.is_paused());
}

#[test]
fn test_detach_clears_state() {
    let vm = Rc::new(RefCell::new(Vm::new()));
    let mut debugger = VmDebugger::new();
    debugger.add_breakpoint("test.gg", 10, None);
    debugger.attach(vm);
    debugger.pause();
    debugger.detach();
    assert!(!debugger.is_attached());
    assert!(!debugger.is_paused());
    assert!(debugger.call_stack().is_empty());
    assert!(debugger.local_variables(0).is_empty());
}

#[test]
fn test_operations_without_attach() {
    let mut debugger = VmDebugger::new();
    debugger.step_over();
    debugger.step_into();
    debugger.step_out();
    debugger.continue_execution();
    debugger.pause();
    assert!(debugger.call_stack().is_empty());
    assert!(debugger.local_variables(0).is_empty());
    assert!(debugger.evaluate("x").is_err());
}

#[test]
fn test_local_variables_out_of_bounds() {
    let debugger = VmDebugger::new();
    assert!(debugger.local_variables(0).is_empty());
    assert!(debugger.local_variables(99).is_empty());
}

#[test]
fn test_debug_protocol_trait() {
    let mut debugger = VmDebugger::new();
    let id = debugger.set_breakpoint("test.gg", 10, None);
    let list = debugger.list_breakpoints();
    assert_eq!(list.len(), 1);
    assert!(debugger.remove_breakpoint(id));
    assert!(debugger.list_breakpoints().is_empty());
}

#[test]
fn test_refresh_debug_state_not_paused() {
    let vm = Rc::new(RefCell::new(Vm::new()));
    let mut debugger = VmDebugger::new();
    debugger.attach(vm);
    debugger.refresh_debug_state();
    assert!(!debugger.is_paused());
    assert!(debugger.call_stack().is_empty());
}

#[test]
fn test_refresh_debug_state_paused() {
    let mut module = BytecodeModule {
        name: "test".to_string(),
        version: 1,
        constants: vec![],
        int_constants: vec![],
        float_constants: vec![],
        string_constants: vec![],
        entity_constants: vec![],
        bool_constants: vec![],
        string_pool: vec![],
        functions: vec![BytecodeFunction {
            name: "main".to_string(),
            param_count: 0,
            local_count: 2,
            local_names: vec![],
            instructions: vec![BytecodeInstruction::LoadNull, BytecodeInstruction::Return],
        }],
        debug_info: None,
        entry_points: vec![],
        function_index: std::collections::HashMap::new(),
    };

    let mut debug_info = DebugInfo::new();
    debug_info.add_entry(0, SourceLocation::new("test.gg", 1, 1));
    debug_info.add_entry(1, SourceLocation::new("test.gg", 2, 1));
    module.debug_info = Some(debug_info);

    let vm = Rc::new(RefCell::new(Vm::new()));
    let mut debugger = VmDebugger::new();
    debugger.add_breakpoint("test.gg", 1, None);
    debugger.attach(vm.clone());

    let mut host = TestHost;
    vm.borrow_mut().execute(&module, "main", &mut host);

    debugger.refresh_debug_state();
    assert!(debugger.is_paused());

    let stack = debugger.call_stack();
    assert_eq!(stack.len(), 1);
    assert_eq!(stack[0].function_name, "main");

    let vars = debugger.local_variables(0);
    assert_eq!(vars.len(), 2);
}

#[test]
fn test_full_debug_flow() {
    let mut module = BytecodeModule {
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
            local_count: 1,
            local_names: vec![],
            instructions: vec![
                BytecodeInstruction::LoadConst { index: 0 },
                BytecodeInstruction::StoreLocal { index: 0 },
                BytecodeInstruction::Return,
            ],
        }],
        debug_info: None,
        entry_points: vec![],
        function_index: std::collections::HashMap::new(),
    };

    let mut debug_info = DebugInfo::new();
    debug_info.add_entry(0, SourceLocation::new("test.gg", 1, 1));
    debug_info.add_entry(1, SourceLocation::new("test.gg", 2, 1));
    debug_info.add_entry(2, SourceLocation::new("test.gg", 3, 1));
    module.debug_info = Some(debug_info);

    let vm = Rc::new(RefCell::new(Vm::new()));
    let mut debugger = VmDebugger::new();
    debugger.add_breakpoint("test.gg", 3, None);
    debugger.attach(vm.clone());

    let mut host = TestHost;
    vm.borrow_mut().execute(&module, "main", &mut host);

    debugger.refresh_debug_state();
    assert!(debugger.is_paused());

    let vars = debugger.local_variables(0);
    assert_eq!(vars.len(), 1);
    assert_eq!(vars[0].1, DebugValue::Int(42));

    debugger.continue_execution();
    assert!(!debugger.is_paused());
}
