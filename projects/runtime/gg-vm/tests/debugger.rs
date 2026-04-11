use gg_vm::{debugger::{VmDebugger, BreakpointId, StepMode, DebugValue}, Vm};
use gg_bytecode::format::{BytecodeFunction, BytecodeInstruction, BytecodeModule, BytecodeValue};
use gg_bytecode::debug_info::DebugInfo;
use gg_bytecode::SourceLocation;
use gg_bytecode::Host;
use std::rc::Rc;
use std::cell::RefCell;

struct TestHost;

impl Host for TestHost {
    fn spawn_entity(&mut self) -> u64 {
        0
    }

    fn despawn_entity(&mut self, _entity_id: u64) {}

    fn add_component(
        &mut self,
        _entity_id: u64,
        _component_type: &str,
        _value: BytecodeValue,
    ) {
    }

    fn get_component_field(
        &mut self,
        _entity_id: u64,
        _component_type: &str,
        _field: &str,
    ) -> Option<BytecodeValue> {
        None
    }

    fn set_component_field(
        &mut self,
        _entity_id: u64,
        _component_type: &str,
        _field: &str,
        _value: BytecodeValue,
    ) {
    }

    fn call_host_function(
        &mut self,
        _name: &str,
        _args: Vec<BytecodeValue>,
    ) -> Option<BytecodeValue> {
        None
    }
}

/// 测试 VmDebugger 创建
#[test]
fn test_debugger_new() {
    let debugger = VmDebugger::new();
    assert!(debugger.list_breakpoints().is_empty());
    assert!(debugger.step_mode.is_none());
    assert!(debugger.call_stack_cache.is_empty());
    assert!(debugger.local_vars_cache.is_empty());
    assert!(!debugger.is_paused());
    assert!(!debugger.is_attached());
}

/// 测试 VmDebugger 默认值
#[test]
fn test_debugger_default() {
    let debugger = VmDebugger::default();
    assert!(debugger.list_breakpoints().is_empty());
    assert!(!debugger.is_attached());
}

/// 测试添加断点
#[test]
fn test_add_breakpoint() {
    let mut debugger = VmDebugger::new();
    let id1 = debugger.add_breakpoint("test.gg", 10, None);
    assert_eq!(id1, BreakpointId(0));

    let id2 = debugger.add_breakpoint("main.gg", 5, Some("x > 0".to_string()));
    assert_eq!(id2, BreakpointId(1));

    let breakpoints = debugger.list_breakpoints();
    assert_eq!(breakpoints.len(), 2);
}

/// 测试添加断点后断点属性正确
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

/// 测试添加无条件断点
#[test]
fn test_add_breakpoint_no_condition() {
    let mut debugger = VmDebugger::new();
    let id = debugger.add_breakpoint("test.gg", 10, None);

    let bp = debugger.list_breakpoints()[0];
    assert_eq!(bp.id, id);
    assert!(bp.condition.is_none());
}

/// 测试移除断点
#[test]
fn test_remove_breakpoint() {
    let mut debugger = VmDebugger::new();
    let id = debugger.add_breakpoint("test.gg", 10, None);
    assert_eq!(debugger.list_breakpoints().len(), 1);

    assert!(debugger.remove_breakpoint(id));
    assert!(debugger.list_breakpoints().is_empty());
}

/// 测试移除不存在的断点
#[test]
fn test_remove_breakpoint_not_found() {
    let mut debugger = VmDebugger::new();
    assert!(!debugger.remove_breakpoint(BreakpointId(99)));
}

/// 测试重复移除断点
#[test]
fn test_remove_breakpoint_twice() {
    let mut debugger = VmDebugger::new();
    let id = debugger.add_breakpoint("test.gg", 10, None);

    assert!(debugger.remove_breakpoint(id));
    assert!(!debugger.remove_breakpoint(id));
}

/// 测试列出断点
#[test]
fn test_list_breakpoints() {
    let mut debugger = VmDebugger::new();
    debugger.add_breakpoint("a.gg", 1, None);
    debugger.add_breakpoint("b.gg", 2, Some("x".to_string()));
    debugger.add_breakpoint("c.gg", 3, None);

    let list = debugger.list_breakpoints();
    assert_eq!(list.len(), 3);
}

/// 测试空断点列表
#[test]
fn test_list_breakpoints_empty() {
    let debugger = VmDebugger::new();
    assert!(debugger.list_breakpoints().is_empty());
}

/// 测试表达式求值存根
#[test]
fn test_evaluate_not_supported() {
    let debugger = VmDebugger::new();
    let result = debugger.evaluate("1 + 2");
    assert!(result.is_err());
}

/// 测试附加到虚拟机
#[test]
fn test_attach() {
    let vm = Rc::new(RefCell::new(Vm::new()));
    let mut debugger = VmDebugger::new();
    assert!(!debugger.is_attached());

    debugger.attach(vm);
    assert!(debugger.is_attached());
}

/// 测试从虚拟机分离
#[test]
fn test_detach() {
    let vm = Rc::new(RefCell::new(Vm::new()));
    let mut debugger = VmDebugger::new();
    debugger.attach(vm);
    assert!(debugger.is_attached());

    debugger.detach();
    assert!(!debugger.is_attached());
}

/// 测试附加后断点同步到控制器
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

/// 测试附加后添加断点
#[test]
fn test_add_breakpoint_after_attach() {
    let vm = Rc::new(RefCell::new(Vm::new()));
    let mut debugger = VmDebugger::new();
    debugger.attach(vm);

    let id = debugger.add_breakpoint("test.gg", 10, None);
    assert_eq!(id, BreakpointId(0));
    assert_eq!(debugger.list_breakpoints().len(), 1);
}

/// 测试附加后移除断点
#[test]
fn test_remove_breakpoint_after_attach() {
    let vm = Rc::new(RefCell::new(Vm::new()));
    let mut debugger = VmDebugger::new();
    debugger.attach(vm);

    let id = debugger.add_breakpoint("test.gg", 10, None);
    assert!(debugger.remove_breakpoint(id));
    assert!(debugger.list_breakpoints().is_empty());
}

/// 测试暂停状态
#[test]
fn test_pause_sets_flag() {
    let vm = Rc::new(RefCell::new(Vm::new()));
    let mut debugger = VmDebugger::new();
    debugger.attach(vm);

    assert!(!debugger.is_paused());
    debugger.pause();
    assert!(debugger.is_paused());
}

/// 测试分离后状态清除
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

/// 测试未附加时操作安全
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

/// 测试越界局部变量查询
#[test]
fn test_local_variables_out_of_bounds() {
    let debugger = VmDebugger::new();
    assert!(debugger.local_variables(0).is_empty());
    assert!(debugger.local_variables(99).is_empty());
}

/// 测试 DebugProtocol trait 实现
#[test]
fn test_debug_protocol_trait() {
    let mut debugger = VmDebugger::new();
    let id = debugger.set_breakpoint("test.gg", 10, None);
    assert_eq!(id, BreakpointId(0));

    let list = debugger.list_breakpoints();
    assert_eq!(list.len(), 1);

    assert!(debugger.remove_breakpoint(id));
    assert!(debugger.list_breakpoints().is_empty());
}

/// 测试刷新调试状态（未暂停）
#[test]
fn test_refresh_debug_state_not_paused() {
    let vm = Rc::new(RefCell::new(Vm::new()));
    let mut debugger = VmDebugger::new();
    debugger.attach(vm);

    debugger.refresh_debug_state();
    assert!(!debugger.is_paused());
    assert!(debugger.call_stack().is_empty());
}

/// 测试刷新调试状态（命中断点后暂停）
#[test]
fn test_refresh_debug_state_paused() {
    let mut module = BytecodeModule {
        name: "test".to_string(),
        version: 1,
        constants: vec![],
        string_pool: vec![],
        functions: vec![BytecodeFunction {
            name: "main".to_string(),
            param_count: 0,
            local_count: 2,
            instructions: vec![
                BytecodeInstruction::LoadNull,
                BytecodeInstruction::Return,
            ],
        }],
        debug_info: None,
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

/// 测试完整调试流程：附加、执行、暂停、刷新、继续
#[test]
fn test_full_debug_flow() {
    let mut module = BytecodeModule {
        name: "test".to_string(),
        version: 1,
        constants: vec![BytecodeValue::Int(42)],
        string_pool: vec![],
        functions: vec![BytecodeFunction {
            name: "main".to_string(),
            param_count: 0,
            local_count: 1,
            instructions: vec![
                BytecodeInstruction::LoadConst { index: 0 },
                BytecodeInstruction::StoreLocal { index: 0 },
                BytecodeInstruction::Return,
            ],
        }],
        debug_info: None,
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

/// 测试单步执行模式设置
#[test]
fn test_step_modes() {
    let vm = Rc::new(RefCell::new(Vm::new()));
    let mut debugger = VmDebugger::new();
    debugger.attach(vm);

    debugger.step_into();
    assert_eq!(debugger.step_mode, Some(StepMode::Into));

    debugger.step_over();
    assert_eq!(debugger.step_mode, Some(StepMode::Over));

    debugger.step_out();
    assert_eq!(debugger.step_mode, Some(StepMode::Out));

    debugger.continue_execution();
    assert!(debugger.step_mode.is_none());
}

/// 测试 BytecodeValue 到 DebugValue 的转换
#[test]
fn test_bytecode_value_to_debug_value() {
    assert_eq!(
        gg_vm::debugger::bytecode_value_to_debug_value(&BytecodeValue::Int(42)),
        DebugValue::Int(42)
    );
    assert_eq!(
        gg_vm::debugger::bytecode_value_to_debug_value(&BytecodeValue::Float(3.14)),
        DebugValue::Float(3.14)
    );
    assert_eq!(
        gg_vm::debugger::bytecode_value_to_debug_value(&BytecodeValue::Bool(true)),
        DebugValue::Bool(true)
    );
    assert_eq!(
        gg_vm::debugger::bytecode_value_to_debug_value(&BytecodeValue::String("hello".to_string())),
        DebugValue::Str("hello".to_string())
    );
    assert_eq!(
        gg_vm::debugger::bytecode_value_to_debug_value(&BytecodeValue::Null),
        DebugValue::Null
    );
    assert!(matches!(
        gg_vm::debugger::bytecode_value_to_debug_value(&BytecodeValue::Entity(1)),
        DebugValue::Str(_)
    ));
}
