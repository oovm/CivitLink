//! VM 调试器模块
//! 提供 VmDebugger 结构体，实现 DebugProtocol trait，
//! 支持断点管理、单步执行、调用栈检查等调试功能

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use gg_bytecode::debug_protocol::*;
use gg_bytecode::BytecodeValue;

use crate::Vm;

/// 将 BytecodeValue 转换为 DebugValue
fn bytecode_value_to_debug_value(value: &BytecodeValue) -> DebugValue {
    match value {
        BytecodeValue::Int(i) => DebugValue::Int(*i),
        BytecodeValue::Float(f) => DebugValue::Float(*f),
        BytecodeValue::Bool(b) => DebugValue::Bool(*b),
        BytecodeValue::String(s) => DebugValue::Str(s.clone()),
        BytecodeValue::Null => DebugValue::Null,
        BytecodeValue::Entity(id) => DebugValue::Str(format!("Entity({})", id)),
    }
}

/// VM 调试器
///
/// 实现 DebugProtocol trait，提供断点管理、单步执行、调用栈检查等调试功能。
/// 通过 `Rc<RefCell<Vm>>` 引用附加到虚拟机实例，在附加时创建 BasicDebugController
/// 并设置到解释器中。
pub struct VmDebugger {
    /// 管理的断点集合
    breakpoints: HashMap<BreakpointId, Breakpoint>,
    /// 下一个断点标识符计数器
    next_breakpoint_id: BreakpointId,
    /// 当前单步执行模式
    step_mode: Option<StepMode>,
    /// 缓存的调用栈信息
    call_stack_cache: Vec<StackFrameInfo>,
    /// 缓存的各栈帧局部变量
    local_vars_cache: Vec<Vec<(String, DebugValue)>>,
    /// 调试器是否处于暂停状态
    is_paused: bool,
    /// 附加的虚拟机引用
    vm: Option<Rc<RefCell<Vm>>>,
    /// 调试控制器引用（与解释器共享）
    controller: Option<Rc<RefCell<BasicDebugController>>>,
}

impl VmDebugger {
    /// 创建新的 VM 调试器
    pub fn new() -> Self {
        Self {
            breakpoints: HashMap::new(),
            next_breakpoint_id: BreakpointId(0),
            step_mode: None,
            call_stack_cache: Vec::new(),
            local_vars_cache: Vec::new(),
            is_paused: false,
            vm: None,
            controller: None,
        }
    }

    /// 添加断点
    ///
    /// 在指定文件和行号创建断点，可选地设置条件表达式。
    /// 如果已附加到虚拟机，断点会同步到调试控制器中。
    /// 返回新创建的断点标识符。
    pub fn add_breakpoint(
        &mut self,
        file: &str,
        line: usize,
        condition: Option<String>,
    ) -> BreakpointId {
        let id = BreakpointId(self.next_breakpoint_id.0);
        self.next_breakpoint_id.0 += 1;
        let breakpoint = Breakpoint {
            id,
            file: file.to_string(),
            line,
            condition,
        };

        if let Some(ref controller) = self.controller {
            controller.borrow_mut().add_file_breakpoint(file, line);
        }

        self.breakpoints.insert(id, breakpoint);
        id
    }

    /// 移除断点
    ///
    /// 移除指定标识符的断点。如果已附加到虚拟机，断点也会从调试控制器中移除。
    /// 返回是否成功移除。
    pub fn remove_breakpoint(&mut self, id: BreakpointId) -> bool {
        if let Some(bp) = self.breakpoints.remove(&id) {
            if let Some(ref controller) = self.controller {
                controller.borrow_mut().remove_file_breakpoint(&bp.file, bp.line);
            }
            true
        } else {
            false
        }
    }

    /// 列出所有断点
    ///
    /// 返回所有断点的引用列表。
    pub fn list_breakpoints(&self) -> Vec<&Breakpoint> {
        self.breakpoints.values().collect()
    }

    /// 单步跳过
    ///
    /// 设置单步跳过模式并恢复虚拟机执行。
    pub fn step_over(&mut self) {
        self.step_mode = Some(StepMode::Over);
        self.is_paused = false;
        if let Some(ref controller) = self.controller {
            controller.borrow_mut().set_step_mode(StepMode::Over);
        }
        self.resume_vm();
    }

    /// 单步进入
    ///
    /// 设置单步进入模式并恢复虚拟机执行。
    pub fn step_into(&mut self) {
        self.step_mode = Some(StepMode::Into);
        self.is_paused = false;
        if let Some(ref controller) = self.controller {
            controller.borrow_mut().set_step_mode(StepMode::Into);
        }
        self.resume_vm();
    }

    /// 单步跳出
    ///
    /// 设置单步跳出模式并恢复虚拟机执行。
    pub fn step_out(&mut self) {
        self.step_mode = Some(StepMode::Out);
        self.is_paused = false;
        if let Some(ref controller) = self.controller {
            controller.borrow_mut().set_step_mode(StepMode::Out);
        }
        self.resume_vm();
    }

    /// 继续执行
    ///
    /// 清除单步执行模式并恢复虚拟机执行。
    pub fn continue_execution(&mut self) {
        self.step_mode = None;
        self.is_paused = false;
        if let Some(ref controller) = self.controller {
            controller.borrow_mut().clear_step_mode();
        }
        self.resume_vm();
    }

    /// 暂停执行
    ///
    /// 设置暂停标志，虚拟机将在下一条指令处暂停。
    pub fn pause(&mut self) {
        self.is_paused = true;
        if let Some(ref controller) = self.controller {
            controller.borrow_mut().set_step_mode(StepMode::Into);
        }
    }

    /// 获取调用栈信息
    ///
    /// 返回缓存的调用栈信息。
    pub fn call_stack(&self) -> Vec<StackFrameInfo> {
        self.call_stack_cache.clone()
    }

    /// 获取指定栈帧的局部变量
    ///
    /// 返回缓存的指定栈帧局部变量列表。
    /// 如果帧索引越界，返回空列表。
    pub fn local_variables(&self, frame_index: usize) -> Vec<(String, DebugValue)> {
        self.local_vars_cache
            .get(frame_index)
            .cloned()
            .unwrap_or_default()
    }

    /// 在当前上下文中求值表达式
    ///
    /// 当前为存根实现，始终返回错误。
    pub fn evaluate(&self, _expression: &str) -> Result<DebugValue, String> {
        Err("表达式求值尚未支持".to_string())
    }

    /// 附加到虚拟机
    ///
    /// 存储虚拟机引用，创建 BasicDebugController 并设置到解释器中。
    /// 已有的断点会同步到调试控制器中。
    pub fn attach(&mut self, vm: Rc<RefCell<Vm>>) {
        let controller = Rc::new(RefCell::new(BasicDebugController::new()));

        for bp in self.breakpoints.values() {
            controller.borrow_mut().add_file_breakpoint(&bp.file, bp.line);
        }

        vm.borrow_mut()
            .set_debug_controller(Some(controller.clone()));

        self.vm = Some(vm);
        self.controller = Some(controller);
    }

    /// 从虚拟机分离
    ///
    /// 移除虚拟机引用，清除调试控制器和调试状态。
    pub fn detach(&mut self) {
        if let Some(ref vm) = self.vm {
            vm.borrow_mut().set_debug_controller(None);
        }
        self.vm = None;
        self.controller = None;
        self.step_mode = None;
        self.is_paused = false;
        self.call_stack_cache.clear();
        self.local_vars_cache.clear();
    }

    /// 检查调试器是否处于暂停状态
    pub fn is_paused(&self) -> bool {
        self.is_paused
    }

    /// 检查是否已附加到虚拟机
    pub fn is_attached(&self) -> bool {
        self.vm.is_some()
    }

    /// 刷新调试状态
    ///
    /// 如果虚拟机处于暂停状态，从解释器刷新调用栈和局部变量缓存。
    /// 应在虚拟机执行后调用此方法以更新调试信息。
    pub fn refresh_debug_state(&mut self) {
        if let Some(ref vm) = self.vm {
            let vm_ref = vm.borrow();
            if vm_ref.is_debug_paused() {
                self.is_paused = true;
                let frames = vm_ref.debug_call_stack();

                self.call_stack_cache = frames
                    .iter()
                    .map(|frame| StackFrameInfo {
                        function_name: frame.function_name.clone(),
                        source_location: None,
                        local_variables: frame
                            .locals
                            .iter()
                            .enumerate()
                            .map(|(i, v)| {
                                (format!("local_{}", i), bytecode_value_to_debug_value(v))
                            })
                            .collect(),
                    })
                    .collect();

                self.local_vars_cache = (0..frames.len())
                    .map(|i| {
                        let vars = vm_ref.debug_local_variables(i);
                        vars.into_iter()
                            .map(|(name, v)| (name, bytecode_value_to_debug_value(&v)))
                            .collect()
                    })
                    .collect();
            } else {
                self.is_paused = false;
            }
        }
    }

    /// 恢复虚拟机执行
    fn resume_vm(&mut self) {
        if let Some(ref vm) = self.vm {
            vm.borrow_mut().resume();
        }
    }
}

impl Default for VmDebugger {
    fn default() -> Self {
        Self::new()
    }
}

impl DebugProtocol for VmDebugger {
    fn set_breakpoint(
        &mut self,
        file: &str,
        line: usize,
        condition: Option<String>,
    ) -> BreakpointId {
        self.add_breakpoint(file, line, condition)
    }

    fn remove_breakpoint(&mut self, id: BreakpointId) -> bool {
        self.remove_breakpoint(id)
    }

    fn list_breakpoints(&self) -> Vec<&Breakpoint> {
        self.list_breakpoints()
    }

    fn step_over(&mut self) {
        self.step_over();
    }

    fn step_into(&mut self) {
        self.step_into();
    }

    fn step_out(&mut self) {
        self.step_out();
    }

    fn continue_execution(&mut self) {
        self.continue_execution();
    }

    fn pause(&mut self) {
        self.pause();
    }

    fn call_stack(&self) -> Vec<StackFrameInfo> {
        self.call_stack()
    }

    fn local_variables(&self, frame_index: usize) -> Vec<(String, DebugValue)> {
        self.local_variables(frame_index)
    }

    fn evaluate(&self, expression: &str) -> Result<DebugValue, String> {
        self.evaluate(expression)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gg_bytecode::format::{BytecodeFunction, BytecodeInstruction, BytecodeModule};
    use gg_bytecode::Host;

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
        use gg_bytecode::debug_info::DebugInfo;
        use gg_bytecode::SourceLocation;

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
        use gg_bytecode::debug_info::DebugInfo;
        use gg_bytecode::SourceLocation;

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
            bytecode_value_to_debug_value(&BytecodeValue::Int(42)),
            DebugValue::Int(42)
        );
        assert_eq!(
            bytecode_value_to_debug_value(&BytecodeValue::Float(3.14)),
            DebugValue::Float(3.14)
        );
        assert_eq!(
            bytecode_value_to_debug_value(&BytecodeValue::Bool(true)),
            DebugValue::Bool(true)
        );
        assert_eq!(
            bytecode_value_to_debug_value(&BytecodeValue::String("hello".to_string())),
            DebugValue::Str("hello".to_string())
        );
        assert_eq!(
            bytecode_value_to_debug_value(&BytecodeValue::Null),
            DebugValue::Null
        );
        assert!(matches!(
            bytecode_value_to_debug_value(&BytecodeValue::Entity(1)),
            DebugValue::Str(_)
        ));
    }
}
