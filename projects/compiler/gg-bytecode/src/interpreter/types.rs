use std::{
    cell::RefCell,
    collections::HashMap,
    rc::Rc,
    sync::{Arc, Mutex},
};

use crate::{
    debug_info::SourceLocation, debug_protocol::DebugController, format::BytecodeValue, host::Host, profiler::BytecodeProfiler,
};

/// 解释器执行结果
#[derive(Debug, Clone)]
pub enum InterpretResult {
    /// 正常完成
    Ok,
    /// 返回值
    Return(BytecodeValue),
    /// 运行时错误
    Error {
        /// 错误消息
        message: String,
        /// 源码位置（可选）
        source_location: Option<SourceLocation>,
    },
}

/// 调用栈帧
#[derive(Clone)]
pub struct InterpreterFrame {
    /// 函数在模块函数列表中的索引，用于快速访问函数数据
    pub function_index: usize,
    /// 函数名称，用于调试
    pub function_name: String,
    /// 指令指针
    pub ip: usize,
    /// 局部变量
    pub locals: Vec<BytecodeValue>,
    /// 栈基指针
    pub stack_base: usize,
    /// 是否通过尾调用优化创建的栈帧
    pub is_tail_call: bool,
}

/// 字节码解释器
pub struct BytecodeInterpreter {
    /// 执行栈
    pub stack: Vec<BytecodeValue>,
    /// 调用栈帧
    pub call_stack: Vec<InterpreterFrame>,
    /// 是否正在运行
    pub running: bool,
    /// 调试控制器
    pub(crate) debug_controller: Option<Rc<RefCell<dyn DebugController>>>,
    /// 是否因调试而暂停
    pub(crate) debug_paused: bool,
    /// 调试暂停时的初始调用栈深度
    pub(crate) debug_initial_depth: usize,
    /// 恢复执行后是否跳过首次断点检查
    pub(crate) debug_skip_check: bool,
    /// 函数名到模块函数列表索引的缓存
    pub function_cache: HashMap<String, usize>,
    /// 可选的性能分析器
    pub(crate) profiler: Option<Arc<Mutex<BytecodeProfiler>>>,
}

/// 指令执行控制流
pub enum ControlFlow {
    /// 继续执行下一条指令
    Continue,
    /// 跳转到指定地址
    Jump(usize),
    /// 从函数返回
    Return(Option<BytecodeValue>),
}
