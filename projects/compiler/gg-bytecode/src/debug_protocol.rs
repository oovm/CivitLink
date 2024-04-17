//! 字节码调试协议模块
//! 提供调试器与解释器之间的通信协议定义，包括断点管理、单步执行、调用栈检查等

use std::{cell::RefCell, collections::HashMap, fmt, rc::Rc};

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
    /// 有序列表
    List(Vec<DebugValue>),
    /// 有序键值对对象
    Object(Vec<(String, DebugValue)>),
    /// 映射
    Map(Vec<(DebugValue, DebugValue)>),
}

impl fmt::Display for DebugValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DebugValue::Int(v) => write!(f, "{}", v),
            DebugValue::Float(v) => write!(f, "{}", v),
            DebugValue::Bool(v) => write!(f, "{}", v),
            DebugValue::Str(v) => write!(f, "\"{}\"", v),
            DebugValue::Null => write!(f, "null"),
            DebugValue::List(items) => {
                let formatted: Vec<String> = items.iter().map(|v| format!("{}", v)).collect();
                write!(f, "[{}]", formatted.join(", "))
            }
            DebugValue::Object(fields) => {
                let formatted: Vec<String> = fields.iter().map(|(k, v)| format!("{}: {}", k, v)).collect();
                write!(f, "{{{}}}", formatted.join(", "))
            }
            DebugValue::Map(entries) => {
                let formatted: Vec<String> = entries.iter().map(|(k, v)| format!("{}: {}", k, v)).collect();
                write!(f, "{{{}}}", formatted.join(", "))
            }
        }
    }
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
        Self { breakpoints: HashMap::new(), step_mode: None, call_depth: 0, step_over_depth: 0, step_out_depth: 0 }
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
    breakpoints: HashMap<BreakpointId, Breakpoint>,
    next_breakpoint_id: u32,
    step_mode: Option<StepMode>,
    call_stack_frames: Vec<StackFrameInfo>,
    controller: Option<Rc<RefCell<BasicDebugController>>>,
}

impl BasicDebugProtocol {
    /// 创建新的基础调试协议实例
    pub fn new() -> Self {
        Self {
            breakpoints: HashMap::new(),
            next_breakpoint_id: 0,
            step_mode: None,
            call_stack_frames: Vec::new(),
            controller: None,
        }
    }

    /// 设置关联的调试控制器
    pub fn set_controller(&mut self, controller: Rc<RefCell<BasicDebugController>>) {
        self.controller = Some(controller);
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
        let breakpoint = Breakpoint { id, file: file.to_string(), line, condition };
        if let Some(ref controller) = self.controller {
            controller.borrow_mut().add_file_breakpoint(file, line);
        }
        self.breakpoints.insert(id, breakpoint);
        id
    }

    fn remove_breakpoint(&mut self, id: BreakpointId) -> bool {
        if let Some(bp) = self.breakpoints.remove(&id) {
            if let Some(ref controller) = self.controller {
                controller.borrow_mut().remove_file_breakpoint(&bp.file, bp.line);
            }
            true
        }
        else {
            false
        }
    }

    fn list_breakpoints(&self) -> Vec<&Breakpoint> {
        self.breakpoints.values().collect()
    }

    fn step_over(&mut self) {
        self.step_mode = Some(StepMode::Over);
        if let Some(ref controller) = self.controller {
            controller.borrow_mut().set_step_mode(StepMode::Over);
        }
    }

    fn step_into(&mut self) {
        self.step_mode = Some(StepMode::Into);
        if let Some(ref controller) = self.controller {
            controller.borrow_mut().set_step_mode(StepMode::Into);
        }
    }

    fn step_out(&mut self) {
        self.step_mode = Some(StepMode::Out);
        if let Some(ref controller) = self.controller {
            controller.borrow_mut().set_step_mode(StepMode::Out);
        }
    }

    fn continue_execution(&mut self) {
        self.step_mode = None;
        if let Some(ref controller) = self.controller {
            controller.borrow_mut().clear_step_mode();
        }
    }

    fn pause(&mut self) {
        self.step_mode = Some(StepMode::Into);
        if let Some(ref controller) = self.controller {
            controller.borrow_mut().set_step_mode(StepMode::Into);
        }
    }

    fn call_stack(&self) -> Vec<StackFrameInfo> {
        self.call_stack_frames.clone()
    }

    fn local_variables(&self, frame_index: usize) -> Vec<(String, DebugValue)> {
        self.call_stack_frames.get(frame_index).map(|f| f.local_variables.clone()).unwrap_or_default()
    }

    fn evaluate(&self, expression: &str) -> Result<DebugValue, String> {
        let expr = expression.trim();

        if expr.contains('[') && expr.ends_with(']') {
            let open_pos = expr.rfind('[').unwrap();
            let base_expr = &expr[..open_pos];
            let index_str = &expr[open_pos + 1..expr.len() - 1];

            let base = self.evaluate(base_expr)?;
            match &base {
                DebugValue::List(items) => {
                    if let Ok(idx) = index_str.parse::<usize>() {
                        if idx < items.len() {
                            return Ok(items[idx].clone());
                        }
                        return Err(format!("索引 {} 超出列表范围 (长度: {})", idx, items.len()));
                    }
                    Err(format!("无效的列表索引: {}", index_str))
                }
                DebugValue::Map(entries) => {
                    let key = self.evaluate(index_str)?;
                    for (k, v) in entries {
                        if *k == key {
                            return Ok(v.clone());
                        }
                    }
                    Err(format!("映射中未找到键: {}", key))
                }
                _ => Err(format!("无法对 {:?} 使用索引访问", base)),
            }
        }
        else if expr.contains('.') {
            let dot_pos = expr.rfind('.').unwrap();
            let base_expr = &expr[..dot_pos];
            let field_name = &expr[dot_pos + 1..];

            let base = self.evaluate(base_expr)?;
            match &base {
                DebugValue::Object(fields) => {
                    for (name, value) in fields {
                        if name == field_name {
                            return Ok(value.clone());
                        }
                    }
                    Err(format!("对象中未找到字段: {}", field_name))
                }
                _ => Err(format!("无法对 {:?} 使用字段访问", base)),
            }
        }
        else {
            if let Some(frame) = self.call_stack_frames.last() {
                for (name, value) in &frame.local_variables {
                    if name == expr {
                        return Ok(value.clone());
                    }
                }
            }

            if expr.contains('+') {
                let parts: Vec<&str> = expr.splitn(2, '+').collect();
                if parts.len() == 2 {
                    let left = self.evaluate(parts[0].trim())?;
                    let right = self.evaluate(parts[1].trim())?;
                    return match (&left, &right) {
                        (DebugValue::Int(a), DebugValue::Int(b)) => Ok(DebugValue::Int(a + b)),
                        (DebugValue::Float(a), DebugValue::Float(b)) => Ok(DebugValue::Float(a + b)),
                        _ => Err(format!("无法对 {:?} 和 {:?} 执行加法", left, right)),
                    };
                }
            }

            if expr.contains('-') {
                let parts: Vec<&str> = expr.splitn(2, '-').collect();
                if parts.len() == 2 {
                    let left = self.evaluate(parts[0].trim())?;
                    let right = self.evaluate(parts[1].trim())?;
                    return match (&left, &right) {
                        (DebugValue::Int(a), DebugValue::Int(b)) => Ok(DebugValue::Int(a - b)),
                        (DebugValue::Float(a), DebugValue::Float(b)) => Ok(DebugValue::Float(a - b)),
                        _ => Err(format!("无法对 {:?} 和 {:?} 执行减法", left, right)),
                    };
                }
            }

            if let Ok(i) = expr.parse::<i64>() {
                return Ok(DebugValue::Int(i));
            }

            if let Ok(f) = expr.parse::<f64>() {
                return Ok(DebugValue::Float(f));
            }

            if expr.starts_with('"') && expr.ends_with('"') && expr.len() >= 2 {
                let s = &expr[1..expr.len() - 1];
                return Ok(DebugValue::Str(s.to_string()));
            }

            if expr == "true" {
                return Ok(DebugValue::Bool(true));
            }
            if expr == "false" {
                return Ok(DebugValue::Bool(false));
            }
            if expr == "null" {
                return Ok(DebugValue::Null);
            }

            Err(format!("无法求值表达式: {}", expression))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_debug_value_list_display() {
        let list = DebugValue::List(vec![DebugValue::Int(1), DebugValue::Int(2), DebugValue::Int(3)]);
        assert_eq!(format!("{}", list), "[1, 2, 3]");

        let empty_list = DebugValue::List(vec![]);
        assert_eq!(format!("{}", empty_list), "[]");

        let mixed_list =
            DebugValue::List(vec![DebugValue::Int(1), DebugValue::Str("hello".to_string()), DebugValue::Bool(true)]);
        assert_eq!(format!("{}", mixed_list), "[1, \"hello\", true]");
    }

    #[test]
    fn test_debug_value_object_display() {
        let obj = DebugValue::Object(vec![
            ("name".to_string(), DebugValue::Str("test".to_string())),
            ("value".to_string(), DebugValue::Int(42)),
        ]);
        assert_eq!(format!("{}", obj), "{name: \"test\", value: 42}");

        let empty_obj = DebugValue::Object(vec![]);
        assert_eq!(format!("{}", empty_obj), "{}");
    }

    #[test]
    fn test_debug_value_map_display() {
        let map = DebugValue::Map(vec![
            (DebugValue::Str("a".to_string()), DebugValue::Int(1)),
            (DebugValue::Str("b".to_string()), DebugValue::Int(2)),
        ]);
        assert_eq!(format!("{}", map), "{\"a\": 1, \"b\": 2}");

        let empty_map = DebugValue::Map(vec![]);
        assert_eq!(format!("{}", empty_map), "{}");
    }

    #[test]
    fn test_evaluate_list_index_access() {
        let mut protocol = BasicDebugProtocol::new();
        protocol.set_call_stack(vec![StackFrameInfo {
            function_name: "test".to_string(),
            source_location: None,
            local_variables: vec![(
                "arr".to_string(),
                DebugValue::List(vec![DebugValue::Int(10), DebugValue::Int(20), DebugValue::Int(30)]),
            )],
        }]);

        assert_eq!(protocol.evaluate("arr[0]").unwrap(), DebugValue::Int(10));
        assert_eq!(protocol.evaluate("arr[1]").unwrap(), DebugValue::Int(20));
        assert_eq!(protocol.evaluate("arr[2]").unwrap(), DebugValue::Int(30));
        assert!(protocol.evaluate("arr[3]").is_err());
        assert!(protocol.evaluate("arr[abc]").is_err());
    }

    #[test]
    fn test_evaluate_object_field_access() {
        let mut protocol = BasicDebugProtocol::new();
        protocol.set_call_stack(vec![StackFrameInfo {
            function_name: "test".to_string(),
            source_location: None,
            local_variables: vec![(
                "obj".to_string(),
                DebugValue::Object(vec![("x".to_string(), DebugValue::Int(100)), ("y".to_string(), DebugValue::Int(200))]),
            )],
        }]);

        assert_eq!(protocol.evaluate("obj.x").unwrap(), DebugValue::Int(100));
        assert_eq!(protocol.evaluate("obj.y").unwrap(), DebugValue::Int(200));
        assert!(protocol.evaluate("obj.z").is_err());
    }

    #[test]
    fn test_evaluate_map_key_access() {
        let mut protocol = BasicDebugProtocol::new();
        protocol.set_call_stack(vec![StackFrameInfo {
            function_name: "test".to_string(),
            source_location: None,
            local_variables: vec![(
                "m".to_string(),
                DebugValue::Map(vec![
                    (DebugValue::Str("key1".to_string()), DebugValue::Int(1)),
                    (DebugValue::Str("key2".to_string()), DebugValue::Int(2)),
                ]),
            )],
        }]);

        assert_eq!(protocol.evaluate("m[\"key1\"]").unwrap(), DebugValue::Int(1));
        assert_eq!(protocol.evaluate("m[\"key2\"]").unwrap(), DebugValue::Int(2));
        assert!(protocol.evaluate("m[\"key3\"]").is_err());
    }

    #[test]
    fn test_evaluate_compound_type_display() {
        let list = DebugValue::List(vec![DebugValue::Int(1), DebugValue::Int(2)]);
        let mut protocol = BasicDebugProtocol::new();
        protocol.set_call_stack(vec![StackFrameInfo {
            function_name: "test".to_string(),
            source_location: None,
            local_variables: vec![("arr".to_string(), list)],
        }]);

        let result = protocol.evaluate("arr").unwrap();
        assert_eq!(format!("{}", result), "[1, 2]");

        let obj = DebugValue::Object(vec![("name".to_string(), DebugValue::Str("hello".to_string()))]);
        protocol.set_call_stack(vec![StackFrameInfo {
            function_name: "test".to_string(),
            source_location: None,
            local_variables: vec![("obj".to_string(), obj)],
        }]);

        let result = protocol.evaluate("obj").unwrap();
        assert_eq!(format!("{}", result), "{name: \"hello\"}");
    }
}
