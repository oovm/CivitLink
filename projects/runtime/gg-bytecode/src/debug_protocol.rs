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


