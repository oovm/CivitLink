//! 字节码调试协议模块
//! 提供调试器与解释器之间的通信协议定义，包括断点管理、单步执行、调用栈检查等

use std::collections::HashMap;

use crate::debug_info::SourceLocation;

/// 断点唯一标识符
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BreakpointId(
    /// 内部标识值
    pub u32,
);

/// 断点信息
#[derive(Debug, Clone)]
pub struct Breakpoint {
    /// 断点唯一标识符
    pub id: BreakpointId,
    /// 源文件路径
    pub file: String,
    /// 源码行号
    pub line: usize,
    /// 可选的条件表达式
    pub condition: Option<String>,
}

/// 调试事件
#[derive(Debug, Clone)]
pub enum DebugEvent {
    /// 命中断点
    BreakpointHit {
        /// 命中的断点标识符
        breakpoint_id: BreakpointId,
        /// 当前指令指针位置
        ip: usize,
    },
    /// 单步执行完成
    StepComplete {
        /// 当前指令指针位置
        ip: usize,
    },
    /// 执行已暂停
    Paused {
        /// 当前指令指针位置
        ip: usize,
    },
    /// 执行已恢复
    Resumed,
    /// 执行已终止
    Terminated {
        /// 终止结果
        result: DebugTermination,
    },
}

/// 调试终止结果
#[derive(Debug, Clone)]
pub enum DebugTermination {
    /// 正常终止
    Normal,
    /// 错误终止
    Error(String),
}

/// 调用栈帧信息
#[derive(Debug, Clone)]
pub struct StackFrameInfo {
    /// 函数名称
    pub function_name: String,
    /// 源码位置（可选）
    pub source_location: Option<SourceLocation>,
    /// 局部变量列表
    pub local_variables: Vec<(String, DebugValue)>,
}

/// 调试值类型，与 BytecodeValue 对应但面向调试视图
#[derive(Debug, Clone, PartialEq)]
pub enum DebugValue {
    /// 整数
    Int(i64),
    /// 浮点数
    Float(f64),
    /// 布尔值
    Bool(bool),
    /// 字符串
    Str(String),
    /// 空值
    Null,
}

/// 单步执行模式
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StepMode {
    /// 单步进入函数内部
    Into,
    /// 单步跳过函数调用
    Over,
    /// 单步跳出当前函数
    Out,
}

/// 调试控制器 trait（解释器端，控制执行暂停/恢复）
pub trait DebugController {
    /// 检查是否应在当前位置暂停执行
    fn check_breakpoint(&self, ip: usize, source_location: Option<&SourceLocation>) -> bool;

    /// 获取当前单步执行模式
    fn get_step_mode(&self) -> Option<StepMode>;

    /// 单步执行完成时调用
    fn on_step_complete(&mut self, ip: usize);

    /// 进入函数时调用
    fn on_function_call(&mut self, function_name: &str, ip: usize);

    /// 从函数返回时调用
    fn on_function_return(&mut self, ip: usize);
}

/// 调试协议 trait（调试器端接口）
pub trait DebugProtocol {
    /// 在指定文件和行号设置断点，返回断点标识符
    fn set_breakpoint(&mut self, file: &str, line: usize, condition: Option<String>) -> BreakpointId;

    /// 移除指定断点，返回是否成功移除
    fn remove_breakpoint(&mut self, id: BreakpointId) -> bool;

    /// 列出所有断点
    fn list_breakpoints(&self) -> Vec<&Breakpoint>;

    /// 单步跳过
    fn step_over(&mut self);

    /// 单步进入
    fn step_into(&mut self);

    /// 单步跳出
    fn step_out(&mut self);

    /// 继续执行
    fn continue_execution(&mut self);

    /// 暂停执行
    fn pause(&mut self);

    /// 获取调用栈信息
    fn call_stack(&self) -> Vec<StackFrameInfo>;

    /// 获取指定栈帧的局部变量
    fn local_variables(&self, frame_index: usize) -> Vec<(String, DebugValue)>;

    /// 在当前上下文中求值表达式
    fn evaluate(&self, expression: &str) -> Result<DebugValue, String>;
}

/// 基础调试控制器实现
pub struct BasicDebugController {
    /// 断点列表（文件路径 -> 行号集合）
    breakpoints: HashMap<String, Vec<usize>>,
    /// 当前单步执行模式
    step_mode: Option<StepMode>,
    /// 当前调用深度
    call_depth: usize,
    /// 单步跳过时的调用深度
    step_over_depth: usize,
    /// 单步跳出时的目标调用深度
    step_out_depth: usize,
}

impl BasicDebugController {
    /// 创建新的基础调试控制器
    pub fn new() -> Self {
        Self {
            breakpoints: HashMap::new(),
            step_mode: None,
            call_depth: 0,
            step_over_depth: 0,
            step_out_depth: 0,
        }
    }

    /// 添加文件行号断点
    pub fn add_file_breakpoint(&mut self, file: &str, line: usize) {
        self.breakpoints.entry(file.to_string()).or_default().push(line);
    }

    /// 移除文件行号断点
    pub fn remove_file_breakpoint(&mut self, file: &str, line: usize) -> bool {
        if let Some(lines) = self.breakpoints.get_mut(file) {
            if let Some(pos) = lines.iter().position(|l| *l == line) {
                lines.remove(pos);
                return true;
            }
        }
        false
    }

    /// 设置单步执行模式
    pub fn set_step_mode(&mut self, mode: StepMode) {
        match mode {
            StepMode::Into => {
                self.step_mode = Some(StepMode::Into);
            }
            StepMode::Over => {
                self.step_over_depth = self.call_depth;
                self.step_mode = Some(StepMode::Over);
            }
            StepMode::Out => {
                self.step_out_depth = self.call_depth.saturating_sub(1);
                self.step_mode = Some(StepMode::Out);
            }
        }
    }

    /// 清除单步执行模式
    pub fn clear_step_mode(&mut self) {
        self.step_mode = None;
    }
}

impl Default for BasicDebugController {
    fn default() -> Self {
        Self::new()
    }
}

impl DebugController for BasicDebugController {
    fn check_breakpoint(&self, _ip: usize, source_location: Option<&SourceLocation>) -> bool {
        if let Some(loc) = source_location {
            if let Some(lines) = self.breakpoints.get(&loc.file) {
                if lines.contains(&(loc.line as usize)) {
                    return true;
                }
            }
        }
        false
    }

    fn get_step_mode(&self) -> Option<StepMode> {
        self.step_mode
    }

    fn on_step_complete(&mut self, _ip: usize) {
        if let Some(mode) = self.step_mode {
            match mode {
                StepMode::Into => {
                    self.step_mode = None;
                }
                StepMode::Over => {
                    if self.call_depth <= self.step_over_depth {
                        self.step_mode = None;
                    }
                }
                StepMode::Out => {
                    if self.call_depth <= self.step_out_depth {
                        self.step_mode = None;
                    }
                }
            }
        }
    }

    fn on_function_call(&mut self, _function_name: &str, _ip: usize) {
        self.call_depth += 1;
    }

    fn on_function_return(&mut self, _ip: usize) {
        self.call_depth = self.call_depth.saturating_sub(1);
    }
}

/// 基础调试协议实现
pub struct BasicDebugProtocol {
    /// 断点集合
    breakpoints: HashMap<BreakpointId, Breakpoint>,
    /// 下一个断点标识符
    next_breakpoint_id: u32,
    /// 当前单步执行模式
    step_mode: Option<StepMode>,
    /// 调用栈信息
    call_stack_frames: Vec<StackFrameInfo>,
}

impl BasicDebugProtocol {
    /// 创建新的基础调试协议实例
    pub fn new() -> Self {
        Self {
            breakpoints: HashMap::new(),
            next_breakpoint_id: 0,
            step_mode: None,
            call_stack_frames: Vec::new(),
        }
    }

    /// 设置调用栈信息
    pub fn set_call_stack(&mut self, frames: Vec<StackFrameInfo>) {
        self.call_stack_frames = frames;
    }
}

impl Default for BasicDebugProtocol {
    fn default() -> Self {
        Self::new()
    }
}

impl DebugProtocol for BasicDebugProtocol {
    fn set_breakpoint(&mut self, file: &str, line: usize, condition: Option<String>) -> BreakpointId {
        let id = BreakpointId(self.next_breakpoint_id);
        self.next_breakpoint_id += 1;
        let breakpoint = Breakpoint {
            id,
            file: file.to_string(),
            line,
            condition,
        };
        self.breakpoints.insert(id, breakpoint);
        id
    }

    fn remove_breakpoint(&mut self, id: BreakpointId) -> bool {
        self.breakpoints.remove(&id).is_some()
    }

    fn list_breakpoints(&self) -> Vec<&Breakpoint> {
        self.breakpoints.values().collect()
    }

    fn step_over(&mut self) {
        self.step_mode = Some(StepMode::Over);
    }

    fn step_into(&mut self) {
        self.step_mode = Some(StepMode::Into);
    }

    fn step_out(&mut self) {
        self.step_mode = Some(StepMode::Out);
    }

    fn continue_execution(&mut self) {
        self.step_mode = None;
    }

    fn pause(&mut self) {
        self.step_mode = Some(StepMode::Into);
    }

    fn call_stack(&self) -> Vec<StackFrameInfo> {
        self.call_stack_frames.clone()
    }

    fn local_variables(&self, frame_index: usize) -> Vec<(String, DebugValue)> {
        self.call_stack_frames
            .get(frame_index)
            .map(|f| f.local_variables.clone())
            .unwrap_or_default()
    }

    fn evaluate(&self, _expression: &str) -> Result<DebugValue, String> {
        Err("表达式求值尚未实现".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_breakpoint_id_equality() {
        let id1 = BreakpointId(1);
        let id2 = BreakpointId(1);
        let id3 = BreakpointId(2);
        assert_eq!(id1, id2);
        assert_ne!(id1, id3);
    }

    #[test]
    fn test_breakpoint_id_hash() {
        use std::collections::HashSet;
        let mut set = HashSet::new();
        set.insert(BreakpointId(1));
        set.insert(BreakpointId(1));
        set.insert(BreakpointId(2));
        assert_eq!(set.len(), 2);
    }

    #[test]
    fn test_breakpoint_creation() {
        let bp = Breakpoint {
            id: BreakpointId(42),
            file: "test.gg".to_string(),
            line: 10,
            condition: Some("x > 0".to_string()),
        };
        assert_eq!(bp.id, BreakpointId(42));
        assert_eq!(bp.file, "test.gg");
        assert_eq!(bp.line, 10);
        assert_eq!(bp.condition.as_deref(), Some("x > 0"));
    }

    #[test]
    fn test_breakpoint_without_condition() {
        let bp = Breakpoint {
            id: BreakpointId(1),
            file: "main.gg".to_string(),
            line: 5,
            condition: None,
        };
        assert!(bp.condition.is_none());
    }

    #[test]
    fn test_debug_event_breakpoint_hit() {
        let event = DebugEvent::BreakpointHit {
            breakpoint_id: BreakpointId(1),
            ip: 42,
        };
        if let DebugEvent::BreakpointHit { breakpoint_id, ip } = event {
            assert_eq!(breakpoint_id, BreakpointId(1));
            assert_eq!(ip, 42);
        } else {
            panic!("期望 BreakpointHit 变体");
        }
    }

    #[test]
    fn test_debug_event_step_complete() {
        let event = DebugEvent::StepComplete { ip: 10 };
        if let DebugEvent::StepComplete { ip } = event {
            assert_eq!(ip, 10);
        } else {
            panic!("期望 StepComplete 变体");
        }
    }

    #[test]
    fn test_debug_event_paused() {
        let event = DebugEvent::Paused { ip: 5 };
        if let DebugEvent::Paused { ip } = event {
            assert_eq!(ip, 5);
        } else {
            panic!("期望 Paused 变体");
        }
    }

    #[test]
    fn test_debug_event_resumed() {
        let event = DebugEvent::Resumed;
        assert!(matches!(event, DebugEvent::Resumed));
    }

    #[test]
    fn test_debug_event_terminated_normal() {
        let event = DebugEvent::Terminated {
            result: DebugTermination::Normal,
        };
        if let DebugEvent::Terminated { result } = event {
            assert!(matches!(result, DebugTermination::Normal));
        } else {
            panic!("期望 Terminated 变体");
        }
    }

    #[test]
    fn test_debug_event_terminated_error() {
        let event = DebugEvent::Terminated {
            result: DebugTermination::Error("运行时错误".to_string()),
        };
        if let DebugEvent::Terminated { result: DebugTermination::Error(msg) } = event {
            assert_eq!(msg, "运行时错误");
        } else {
            panic!("期望 Terminated(Error) 变体");
        }
    }

    #[test]
    fn test_debug_termination_variants() {
        let normal = DebugTermination::Normal;
        let error = DebugTermination::Error("测试错误".to_string());
        assert!(matches!(normal, DebugTermination::Normal));
        assert!(matches!(error, DebugTermination::Error(_)));
    }

    #[test]
    fn test_stack_frame_info() {
        let frame = StackFrameInfo {
            function_name: "main".to_string(),
            source_location: Some(SourceLocation::new("test.gg", 10, 5)),
            local_variables: vec![
                ("x".to_string(), DebugValue::Int(42)),
                ("y".to_string(), DebugValue::Float(3.14)),
            ],
        };
        assert_eq!(frame.function_name, "main");
        assert!(frame.source_location.is_some());
        assert_eq!(frame.local_variables.len(), 2);
    }

    #[test]
    fn test_stack_frame_info_no_source_location() {
        let frame = StackFrameInfo {
            function_name: "foo".to_string(),
            source_location: None,
            local_variables: vec![],
        };
        assert!(frame.source_location.is_none());
        assert!(frame.local_variables.is_empty());
    }

    #[test]
    fn test_debug_value_int() {
        let val = DebugValue::Int(42);
        assert_eq!(val, DebugValue::Int(42));
        assert_ne!(val, DebugValue::Int(0));
    }

    #[test]
    fn test_debug_value_float() {
        let val = DebugValue::Float(3.14);
        assert_eq!(val, DebugValue::Float(3.14));
    }

    #[test]
    fn test_debug_value_bool() {
        assert_eq!(DebugValue::Bool(true), DebugValue::Bool(true));
        assert_ne!(DebugValue::Bool(true), DebugValue::Bool(false));
    }

    #[test]
    fn test_debug_value_str() {
        let val = DebugValue::Str("hello".to_string());
        assert_eq!(val, DebugValue::Str("hello".to_string()));
    }

    #[test]
    fn test_debug_value_null() {
        assert_eq!(DebugValue::Null, DebugValue::Null);
    }

    #[test]
    fn test_debug_value_different_types_not_equal() {
        assert_ne!(DebugValue::Int(0), DebugValue::Bool(false));
        assert_ne!(DebugValue::Int(1), DebugValue::Float(1.0));
        assert_ne!(DebugValue::Str("true".to_string()), DebugValue::Bool(true));
    }

    #[test]
    fn test_step_mode_variants() {
        let into_mode = StepMode::Into;
        let over_mode = StepMode::Over;
        let out_mode = StepMode::Out;
        assert_ne!(into_mode, over_mode);
        assert_ne!(over_mode, out_mode);
        assert_ne!(into_mode, out_mode);
    }

    #[test]
    fn test_basic_debug_controller_new() {
        let controller = BasicDebugController::new();
        assert!(controller.get_step_mode().is_none());
    }

    #[test]
    fn test_basic_debug_controller_check_breakpoint_no_location() {
        let controller = BasicDebugController::new();
        assert!(!controller.check_breakpoint(0, None));
    }

    #[test]
    fn test_basic_debug_controller_check_breakpoint_with_location() {
        let mut controller = BasicDebugController::new();
        controller.add_file_breakpoint("test.gg", 10);
        let loc = SourceLocation::new("test.gg", 10, 1);
        assert!(controller.check_breakpoint(5, Some(&loc)));
    }

    #[test]
    fn test_basic_debug_controller_check_breakpoint_wrong_line() {
        let mut controller = BasicDebugController::new();
        controller.add_file_breakpoint("test.gg", 10);
        let loc = SourceLocation::new("test.gg", 5, 1);
        assert!(!controller.check_breakpoint(5, Some(&loc)));
    }

    #[test]
    fn test_basic_debug_controller_check_breakpoint_wrong_file() {
        let mut controller = BasicDebugController::new();
        controller.add_file_breakpoint("test.gg", 10);
        let loc = SourceLocation::new("other.gg", 10, 1);
        assert!(!controller.check_breakpoint(5, Some(&loc)));
    }

    #[test]
    fn test_basic_debug_controller_step_into() {
        let mut controller = BasicDebugController::new();
        controller.set_step_mode(StepMode::Into);
        assert_eq!(controller.get_step_mode(), Some(StepMode::Into));
        controller.on_step_complete(0);
        assert!(controller.get_step_mode().is_none());
    }

    #[test]
    fn test_basic_debug_controller_step_over_same_depth() {
        let mut controller = BasicDebugController::new();
        controller.set_step_mode(StepMode::Over);
        assert_eq!(controller.get_step_mode(), Some(StepMode::Over));
        controller.on_step_complete(0);
        assert!(controller.get_step_mode().is_none());
    }

    #[test]
    fn test_basic_debug_controller_step_over_deeper_call() {
        let mut controller = BasicDebugController::new();
        controller.set_step_mode(StepMode::Over);
        controller.on_function_call("inner", 0);
        assert_eq!(controller.get_step_mode(), Some(StepMode::Over));
        controller.on_step_complete(1);
        assert_eq!(controller.get_step_mode(), Some(StepMode::Over));
        controller.on_function_return(2);
        controller.on_step_complete(3);
        assert!(controller.get_step_mode().is_none());
    }

    #[test]
    fn test_basic_debug_controller_step_out() {
        let mut controller = BasicDebugController::new();
        controller.on_function_call("outer", 0);
        controller.set_step_mode(StepMode::Out);
        controller.on_function_call("inner", 1);
        assert_eq!(controller.get_step_mode(), Some(StepMode::Out));
        controller.on_step_complete(2);
        assert_eq!(controller.get_step_mode(), Some(StepMode::Out));
        controller.on_function_return(3);
        controller.on_step_complete(4);
        assert_eq!(controller.get_step_mode(), Some(StepMode::Out));
        controller.on_function_return(5);
        controller.on_step_complete(6);
        assert!(controller.get_step_mode().is_none());
    }

    #[test]
    fn test_basic_debug_controller_function_call_depth() {
        let mut controller = BasicDebugController::new();
        controller.on_function_call("f1", 0);
        controller.on_function_call("f2", 1);
        controller.on_function_return(2);
        controller.on_function_call("f3", 3);
    }

    #[test]
    fn test_basic_debug_controller_remove_file_breakpoint() {
        let mut controller = BasicDebugController::new();
        controller.add_file_breakpoint("test.gg", 10);
        assert!(controller.remove_file_breakpoint("test.gg", 10));
        assert!(!controller.remove_file_breakpoint("test.gg", 10));
        assert!(!controller.remove_file_breakpoint("other.gg", 10));
    }

    #[test]
    fn test_basic_debug_controller_clear_step_mode() {
        let mut controller = BasicDebugController::new();
        controller.set_step_mode(StepMode::Into);
        assert_eq!(controller.get_step_mode(), Some(StepMode::Into));
        controller.clear_step_mode();
        assert!(controller.get_step_mode().is_none());
    }

    #[test]
    fn test_basic_debug_protocol_set_breakpoint() {
        let mut protocol = BasicDebugProtocol::new();
        let id = protocol.set_breakpoint("test.gg", 10, None);
        assert_eq!(id, BreakpointId(0));
        let id2 = protocol.set_breakpoint("test.gg", 20, Some("x > 0".to_string()));
        assert_eq!(id2, BreakpointId(1));
    }

    #[test]
    fn test_basic_debug_protocol_remove_breakpoint() {
        let mut protocol = BasicDebugProtocol::new();
        let id = protocol.set_breakpoint("test.gg", 10, None);
        assert!(protocol.remove_breakpoint(id));
        assert!(!protocol.remove_breakpoint(id));
    }

    #[test]
    fn test_basic_debug_protocol_list_breakpoints() {
        let mut protocol = BasicDebugProtocol::new();
        protocol.set_breakpoint("a.gg", 1, None);
        protocol.set_breakpoint("b.gg", 2, Some("x".to_string()));
        let list = protocol.list_breakpoints();
        assert_eq!(list.len(), 2);
    }

    #[test]
    fn test_basic_debug_protocol_list_breakpoints_empty() {
        let protocol = BasicDebugProtocol::new();
        assert!(protocol.list_breakpoints().is_empty());
    }

    #[test]
    fn test_basic_debug_protocol_step_modes() {
        let mut protocol = BasicDebugProtocol::new();
        protocol.step_into();
        protocol.step_over();
        protocol.step_out();
        protocol.continue_execution();
        protocol.pause();
    }

    #[test]
    fn test_basic_debug_protocol_call_stack() {
        let mut protocol = BasicDebugProtocol::new();
        let frames = vec![StackFrameInfo {
            function_name: "main".to_string(),
            source_location: None,
            local_variables: vec![("x".to_string(), DebugValue::Int(1))],
        }];
        protocol.set_call_stack(frames);
        let stack = protocol.call_stack();
        assert_eq!(stack.len(), 1);
        assert_eq!(stack[0].function_name, "main");
    }

    #[test]
    fn test_basic_debug_protocol_local_variables() {
        let mut protocol = BasicDebugProtocol::new();
        let frames = vec![
            StackFrameInfo {
                function_name: "main".to_string(),
                source_location: None,
                local_variables: vec![("x".to_string(), DebugValue::Int(1))],
            },
            StackFrameInfo {
                function_name: "foo".to_string(),
                source_location: None,
                local_variables: vec![
                    ("a".to_string(), DebugValue::Bool(true)),
                    ("b".to_string(), DebugValue::Null),
                ],
            },
        ];
        protocol.set_call_stack(frames);
        let vars0 = protocol.local_variables(0);
        assert_eq!(vars0.len(), 1);
        assert_eq!(vars0[0].0, "x");
        let vars1 = protocol.local_variables(1);
        assert_eq!(vars1.len(), 2);
        let vars_out = protocol.local_variables(99);
        assert!(vars_out.is_empty());
    }

    #[test]
    fn test_basic_debug_protocol_evaluate_not_implemented() {
        let protocol = BasicDebugProtocol::new();
        let result = protocol.evaluate("1 + 2");
        assert!(result.is_err());
    }
}
