use std::{
    cell::RefCell,
    collections::HashMap,
    rc::Rc,
    sync::{Arc, Mutex},
};

use crate::{
    debug_info::SourceLocation,
    debug_protocol::DebugController,
    format::{BytecodeInstruction, BytecodeModule, BytecodeValue},
    host::Host,
    profiler::BytecodeProfiler,
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
    debug_controller: Option<Rc<RefCell<dyn DebugController>>>,
    /// 是否因调试而暂停
    debug_paused: bool,
    /// 调试暂停时的初始调用栈深度
    debug_initial_depth: usize,
    /// 恢复执行后是否跳过首次断点检查
    debug_skip_check: bool,
    /// 函数名到模块函数列表索引的缓存
    function_cache: HashMap<String, usize>,
    /// 可选的性能分析器
    profiler: Option<Arc<Mutex<BytecodeProfiler>>>,
}

impl BytecodeInterpreter {
    /// 创建新的字节码解释器
    pub fn new() -> Self {
        Self {
            stack: Vec::with_capacity(256),
            call_stack: Vec::with_capacity(32),
            running: false,
            debug_controller: None,
            debug_paused: false,
            debug_initial_depth: 0,
            debug_skip_check: false,
            function_cache: HashMap::new(),
            profiler: None,
        }
    }

    /// 创建带性能分析器的字节码解释器
    pub fn with_profiler(profiler: Arc<Mutex<BytecodeProfiler>>) -> Self {
        Self {
            stack: Vec::with_capacity(256),
            call_stack: Vec::with_capacity(32),
            running: false,
            debug_controller: None,
            debug_paused: false,
            debug_initial_depth: 0,
            debug_skip_check: false,
            function_cache: HashMap::new(),
            profiler: Some(profiler),
        }
    }

    /// 设置调试控制器
    pub fn set_debug_controller(&mut self, controller: Option<Rc<RefCell<dyn DebugController>>>) {
        self.debug_controller = controller;
    }

    /// 检查解释器是否因调试而暂停
    pub fn is_debug_paused(&self) -> bool {
        self.debug_paused
    }

    /// 恢复调试暂停的执行
    pub fn resume(&mut self) {
        self.debug_paused = false;
        self.debug_skip_check = true;
    }

    /// 获取调用栈的克隆副本，用于调试检查
    pub fn debug_call_stack(&self) -> Vec<InterpreterFrame> {
        self.call_stack.clone()
    }

    /// 获取指定栈帧的局部变量列表
    pub fn debug_local_variables(&self, frame_index: usize) -> Vec<(String, BytecodeValue)> {
        match self.call_stack.get(frame_index) {
            Some(frame) => frame.locals.iter().enumerate().map(|(i, v)| (format!("local_{}", i), v.clone())).collect(),
            None => Vec::new(),
        }
    }

    /// 确保函数缓存已构建，从模块的 function_index 或函数列表构建
    fn ensure_function_cache(&mut self, module: &BytecodeModule) {
        if !self.function_cache.is_empty() {
            return;
        }
        if !module.function_index.is_empty() {
            for (name, idx) in &module.function_index {
                self.function_cache.insert(name.clone(), *idx);
            }
        }
        else {
            for (i, func) in module.functions.iter().enumerate() {
                self.function_cache.insert(func.name.clone(), i);
            }
        }
    }

    /// 查找函数索引，优先使用缓存，未命中时回退到线性搜索并缓存结果
    fn find_function_index(&mut self, module: &BytecodeModule, name: &str) -> Option<usize> {
        self.ensure_function_cache(module);
        if let Some(&idx) = self.function_cache.get(name) {
            return Some(idx);
        }
        for (i, func) in module.functions.iter().enumerate() {
            if func.name == name {
                self.function_cache.insert(name.to_string(), i);
                return Some(i);
            }
        }
        None
    }

    /// 执行模块中的指定函数
    pub fn execute<H: Host>(&mut self, module: &BytecodeModule, function_name: &str, host: &mut H) -> InterpretResult {
        if self.debug_paused {
            return InterpretResult::Ok;
        }

        let initial_depth = if self.call_stack.is_empty() {
            let function_index = match self.find_function_index(module, function_name) {
                Some(idx) => idx,
                None => {
                    return InterpretResult::Error {
                        message: format!("函数未找到: {}", function_name), source_location: None
                    };
                }
            };

            let function = &module.functions[function_index];
            let mut locals = vec![BytecodeValue::Null; function.local_count as usize];
            for i in 0..function.param_count.min(function.local_count) as usize {
                if let Some(val) = self.stack.pop() {
                    locals[i] = val;
                }
            }

            let stack_base = self.stack.len();
            let depth = self.call_stack.len();

            self.call_stack.push(InterpreterFrame {
                function_index,
                function_name: function.name.clone(),
                ip: 0,
                locals,
                stack_base,
                is_tail_call: false,
            });

            self.debug_initial_depth = depth;
            self.debug_skip_check = false;
            depth
        }
        else {
            self.debug_initial_depth
        };

        self.running = true;
        let result = self.execute_instructions(module, host, initial_depth);
        self.running = false;

        self.enrich_error_with_source_location(module, result)
    }

    /// 为错误结果补充源码位置信息
    fn enrich_error_with_source_location(&self, module: &BytecodeModule, result: InterpretResult) -> InterpretResult {
        match result {
            InterpretResult::Error { message, source_location: None } => {
                let source_location = self.resolve_source_location(module);
                InterpretResult::Error { message, source_location }
            }
            other => other,
        }
    }

    /// 从调用栈和调试信息中解析源码位置
    fn resolve_source_location(&self, module: &BytecodeModule) -> Option<SourceLocation> {
        let debug_info = module.debug_info.as_ref()?;
        let frame = self.call_stack.last()?;
        debug_info.lookup(frame.ip as u32).cloned()
    }

    /// 执行当前栈帧的指令序列
    fn execute_instructions<H: Host>(
        &mut self,
        module: &BytecodeModule,
        host: &mut H,
        initial_depth: usize,
    ) -> InterpretResult {
        while self.running {
            if self.call_stack.len() <= initial_depth {
                return InterpretResult::Ok;
            }

            let (function_index, ip) = match self.call_stack.last() {
                Some(f) => (f.function_index, f.ip),
                None => return InterpretResult::Ok,
            };

            let function = match module.functions.get(function_index) {
                Some(f) => f,
                None => {
                    return InterpretResult::Error {
                        message: format!("函数索引越界: {}", function_index),
                        source_location: None,
                    };
                }
            };

            let instructions = &function.instructions;
            if ip >= instructions.len() {
                self.call_stack.pop();
                continue;
            }

            if let Some(ref controller) = self.debug_controller {
                if self.debug_skip_check {
                    self.debug_skip_check = false;
                }
                else {
                    let source_location = module.debug_info.as_ref().and_then(|di| di.lookup(ip as u32));

                    let breakpoint_hit = {
                        let ctrl = controller.borrow();
                        ctrl.check_breakpoint(ip, source_location)
                    };

                    let step_was_active = {
                        let ctrl = controller.borrow();
                        ctrl.get_step_mode().is_some()
                    };

                    {
                        let mut ctrl = controller.borrow_mut();
                        ctrl.on_step_complete(ip);
                    }

                    let step_completed = step_was_active && {
                        let ctrl = controller.borrow();
                        ctrl.get_step_mode().is_none()
                    };

                    if breakpoint_hit || step_completed {
                        self.debug_paused = true;
                        return InterpretResult::Ok;
                    }
                }
            }

            let op = instructions[ip].clone();
            let is_call = matches!(op, BytecodeInstruction::Call { .. } | BytecodeInstruction::TailCall(_));
            let is_return = matches!(op, BytecodeInstruction::Return);

            if let Some(f) = self.call_stack.last_mut() {
                f.ip += 1;
            }
            else {
                return InterpretResult::Error { message: "调用栈为空".to_string(), source_location: None };
            }

            match self.dispatch_op(module, host, op) {
                Ok(ControlFlow::Continue) => {}
                Ok(ControlFlow::Jump(addr)) => {
                    if let Some(f) = self.call_stack.last_mut() {
                        f.ip = addr;
                    }
                }
                Ok(ControlFlow::Return(value)) => {
                    if let Some(ref controller) = self.debug_controller {
                        controller.borrow_mut().on_function_return(ip);
                    }
                    self.call_stack.pop();
                    if self.call_stack.len() <= initial_depth {
                        return InterpretResult::Return(value.unwrap_or(BytecodeValue::Null));
                    }
                    if let Some(value) = value {
                        self.stack.push(value);
                    }
                }
                Err(result) => return result,
            }

            if is_call {
                if let Some(ref controller) = self.debug_controller {
                    if let Some(frame) = self.call_stack.last() {
                        controller.borrow_mut().on_function_call(&frame.function_name, frame.ip);
                    }
                }
                if let Some(ref profiler) = self.profiler {
                    if let Ok(mut p) = profiler.lock() {
                        if p.is_enabled() {
                            if let Some(frame) = self.call_stack.last() {
                                p.on_function_enter(&frame.function_name);
                            }
                        }
                    }
                }
            }

            if is_return {
                if let Some(ref controller) = self.debug_controller {
                    controller.borrow_mut().on_function_return(ip);
                }
                if let Some(ref profiler) = self.profiler {
                    if let Ok(mut p) = profiler.lock() {
                        if p.is_enabled() {
                            if let Some(frame) = self.call_stack.last() {
                                p.on_function_exit(&frame.function_name);
                            }
                        }
                    }
                }
            }
        }

        InterpretResult::Ok
    }

    /// 分发操作码执行
    fn dispatch_op<H: Host>(
        &mut self,
        module: &BytecodeModule,
        host: &mut H,
        op: BytecodeInstruction,
    ) -> Result<ControlFlow, InterpretResult> {
        match op {
            BytecodeInstruction::LoadConst { index } => {
                let value = match module.constants.get(index as usize) {
                    Some(v) => v.clone(),
                    None => {
                        return Err(InterpretResult::Error {
                            message: format!("常量索引越界: {}", index),
                            source_location: None,
                        });
                    }
                };
                self.stack.push(value);
                Ok(ControlFlow::Continue)
            }

            BytecodeInstruction::LoadConstInt { index } => {
                let value = match module.int_constants.get(index as usize) {
                    Some(v) => *v,
                    None => {
                        return Err(InterpretResult::Error {
                            message: format!("整数常量索引越界: {}", index),
                            source_location: None,
                        });
                    }
                };
                self.stack.push(BytecodeValue::Int(value));
                Ok(ControlFlow::Continue)
            }

            BytecodeInstruction::LoadConstFloat { index } => {
                let value = match module.float_constants.get(index as usize) {
                    Some(v) => *v,
                    None => {
                        return Err(InterpretResult::Error {
                            message: format!("浮点常量索引越界: {}", index),
                            source_location: None,
                        });
                    }
                };
                self.stack.push(BytecodeValue::Float(value));
                Ok(ControlFlow::Continue)
            }

            BytecodeInstruction::LoadConstString { index } => {
                let value = match module.string_constants.get(index as usize) {
                    Some(v) => v.clone(),
                    None => {
                        return Err(InterpretResult::Error {
                            message: format!("字符串常量索引越界: {}", index),
                            source_location: None,
                        });
                    }
                };
                self.stack.push(BytecodeValue::String(value));
                Ok(ControlFlow::Continue)
            }

            BytecodeInstruction::LoadConstEntity { index } => {
                let value = match module.entity_constants.get(index as usize) {
                    Some(v) => *v,
                    None => {
                        return Err(InterpretResult::Error {
                            message: format!("实体常量索引越界: {}", index),
                            source_location: None,
                        });
                    }
                };
                self.stack.push(BytecodeValue::Entity(value));
                Ok(ControlFlow::Continue)
            }

            BytecodeInstruction::LoadConstBool { index } => {
                let value = match module.bool_constants.get(index as usize) {
                    Some(v) => *v,
                    None => {
                        return Err(InterpretResult::Error {
                            message: format!("布尔常量索引越界: {}", index),
                            source_location: None,
                        });
                    }
                };
                self.stack.push(BytecodeValue::Bool(value));
                Ok(ControlFlow::Continue)
            }

            BytecodeInstruction::LoadNull => {
                self.stack.push(BytecodeValue::Null);
                Ok(ControlFlow::Continue)
            }

            BytecodeInstruction::LoadTrue => {
                self.stack.push(BytecodeValue::Bool(true));
                Ok(ControlFlow::Continue)
            }

            BytecodeInstruction::LoadFalse => {
                self.stack.push(BytecodeValue::Bool(false));
                Ok(ControlFlow::Continue)
            }

            BytecodeInstruction::LoadLocal { index } => {
                let frame = match self.call_stack.last() {
                    Some(f) => f,
                    None => {
                        return Err(InterpretResult::Error { message: "调用栈为空".to_string(), source_location: None });
                    }
                };
                let value = match frame.locals.get(index as usize) {
                    Some(v) => v.clone(),
                    None => {
                        return Err(InterpretResult::Error {
                            message: format!("局部变量索引越界: {}", index),
                            source_location: None,
                        });
                    }
                };
                self.stack.push(value);
                Ok(ControlFlow::Continue)
            }

            BytecodeInstruction::StoreLocal { index } => {
                let value = match self.stack.pop() {
                    Some(v) => v,
                    None => {
                        return Err(InterpretResult::Error {
                            message: "栈下溢: StoreLocal".to_string(),
                            source_location: None,
                        });
                    }
                };
                let frame = match self.call_stack.last_mut() {
                    Some(f) => f,
                    None => {
                        return Err(InterpretResult::Error { message: "调用栈为空".to_string(), source_location: None });
                    }
                };
                if index as usize >= frame.locals.len() {
                    return Err(InterpretResult::Error {
                        message: format!("局部变量索引越界: {}", index),
                        source_location: None,
                    });
                }
                frame.locals[index as usize] = value;
                Ok(ControlFlow::Continue)
            }

            BytecodeInstruction::Add => self.binary_op(|a, b| match (a, b) {
                (BytecodeValue::Int(x), BytecodeValue::Int(y)) => BytecodeValue::Int(x + y),
                (BytecodeValue::Float(x), BytecodeValue::Float(y)) => BytecodeValue::Float(x + y),
                _ => BytecodeValue::Null,
            }),

            BytecodeInstruction::Sub => self.binary_op(|a, b| match (a, b) {
                (BytecodeValue::Int(x), BytecodeValue::Int(y)) => BytecodeValue::Int(x - y),
                (BytecodeValue::Float(x), BytecodeValue::Float(y)) => BytecodeValue::Float(x - y),
                _ => BytecodeValue::Null,
            }),

            BytecodeInstruction::Mul => self.binary_op(|a, b| match (a, b) {
                (BytecodeValue::Int(x), BytecodeValue::Int(y)) => BytecodeValue::Int(x * y),
                (BytecodeValue::Float(x), BytecodeValue::Float(y)) => BytecodeValue::Float(x * y),
                _ => BytecodeValue::Null,
            }),

            BytecodeInstruction::Div => self.binary_op(|a, b| match (a, b) {
                (BytecodeValue::Int(x), BytecodeValue::Int(y)) => {
                    if y == 0 {
                        BytecodeValue::Null
                    }
                    else {
                        BytecodeValue::Int(x / y)
                    }
                }
                (BytecodeValue::Float(x), BytecodeValue::Float(y)) => {
                    if y == 0.0 {
                        BytecodeValue::Null
                    }
                    else {
                        BytecodeValue::Float(x / y)
                    }
                }
                _ => BytecodeValue::Null,
            }),

            BytecodeInstruction::Mod => self.binary_op(|a, b| match (a, b) {
                (BytecodeValue::Int(x), BytecodeValue::Int(y)) => {
                    if y == 0 {
                        BytecodeValue::Null
                    }
                    else {
                        BytecodeValue::Int(x % y)
                    }
                }
                (BytecodeValue::Float(x), BytecodeValue::Float(y)) => {
                    if y == 0.0 {
                        BytecodeValue::Null
                    }
                    else {
                        BytecodeValue::Float(x % y)
                    }
                }
                _ => BytecodeValue::Null,
            }),

            BytecodeInstruction::Neg => {
                let value = match self.stack.pop() {
                    Some(v) => v,
                    None => {
                        return Err(InterpretResult::Error { message: "栈下溢: Neg".to_string(), source_location: None });
                    }
                };
                let result = match value {
                    BytecodeValue::Int(x) => BytecodeValue::Int(-x),
                    BytecodeValue::Float(x) => BytecodeValue::Float(-x),
                    _ => {
                        return Err(InterpretResult::Error {
                            message: format!("Neg 操作数类型错误: {:?}", value),
                            source_location: None,
                        });
                    }
                };
                self.stack.push(result);
                Ok(ControlFlow::Continue)
            }

            BytecodeInstruction::Eq => self.compare_op(|a, b| a == b),

            BytecodeInstruction::Ne => self.compare_op(|a, b| a != b),

            BytecodeInstruction::Lt => self.compare_op(|a, b| match (a, b) {
                (BytecodeValue::Int(x), BytecodeValue::Int(y)) => x < y,
                (BytecodeValue::Float(x), BytecodeValue::Float(y)) => x < y,
                _ => false,
            }),

            BytecodeInstruction::Le => self.compare_op(|a, b| match (a, b) {
                (BytecodeValue::Int(x), BytecodeValue::Int(y)) => x <= y,
                (BytecodeValue::Float(x), BytecodeValue::Float(y)) => x <= y,
                _ => false,
            }),

            BytecodeInstruction::Gt => self.compare_op(|a, b| match (a, b) {
                (BytecodeValue::Int(x), BytecodeValue::Int(y)) => x > y,
                (BytecodeValue::Float(x), BytecodeValue::Float(y)) => x > y,
                _ => false,
            }),

            BytecodeInstruction::Ge => self.compare_op(|a, b| match (a, b) {
                (BytecodeValue::Int(x), BytecodeValue::Int(y)) => x >= y,
                (BytecodeValue::Float(x), BytecodeValue::Float(y)) => x >= y,
                _ => false,
            }),

            BytecodeInstruction::And => {
                let b = match self.stack.pop() {
                    Some(BytecodeValue::Bool(v)) => v,
                    Some(v) => {
                        return Err(InterpretResult::Error {
                            message: format!("And 操作数类型错误: {:?}", v),
                            source_location: None,
                        });
                    }
                    None => {
                        return Err(InterpretResult::Error { message: "栈下溢: And".to_string(), source_location: None });
                    }
                };
                let a = match self.stack.pop() {
                    Some(BytecodeValue::Bool(v)) => v,
                    Some(v) => {
                        return Err(InterpretResult::Error {
                            message: format!("And 操作数类型错误: {:?}", v),
                            source_location: None,
                        });
                    }
                    None => {
                        return Err(InterpretResult::Error { message: "栈下溢: And".to_string(), source_location: None });
                    }
                };
                self.stack.push(BytecodeValue::Bool(a && b));
                Ok(ControlFlow::Continue)
            }

            BytecodeInstruction::Or => {
                let b = match self.stack.pop() {
                    Some(BytecodeValue::Bool(v)) => v,
                    Some(v) => {
                        return Err(InterpretResult::Error {
                            message: format!("Or 操作数类型错误: {:?}", v),
                            source_location: None,
                        });
                    }
                    None => {
                        return Err(InterpretResult::Error { message: "栈下溢: Or".to_string(), source_location: None });
                    }
                };
                let a = match self.stack.pop() {
                    Some(BytecodeValue::Bool(v)) => v,
                    Some(v) => {
                        return Err(InterpretResult::Error {
                            message: format!("Or 操作数类型错误: {:?}", v),
                            source_location: None,
                        });
                    }
                    None => {
                        return Err(InterpretResult::Error { message: "栈下溢: Or".to_string(), source_location: None });
                    }
                };
                self.stack.push(BytecodeValue::Bool(a || b));
                Ok(ControlFlow::Continue)
            }

            BytecodeInstruction::Not => {
                let value = match self.stack.pop() {
                    Some(BytecodeValue::Bool(v)) => v,
                    Some(v) => {
                        return Err(InterpretResult::Error {
                            message: format!("Not 操作数类型错误: {:?}", v),
                            source_location: None,
                        });
                    }
                    None => {
                        return Err(InterpretResult::Error { message: "栈下溢: Not".to_string(), source_location: None });
                    }
                };
                self.stack.push(BytecodeValue::Bool(!value));
                Ok(ControlFlow::Continue)
            }

            BytecodeInstruction::Jump { address } => Ok(ControlFlow::Jump(address as usize)),

            BytecodeInstruction::JumpIfFalse { address } => {
                let value = match self.stack.pop() {
                    Some(v) => v,
                    None => {
                        return Err(InterpretResult::Error {
                            message: "栈下溢: JumpIfFalse".to_string(),
                            source_location: None,
                        });
                    }
                };
                match value {
                    BytecodeValue::Bool(false) => Ok(ControlFlow::Jump(address as usize)),
                    BytecodeValue::Bool(true) => Ok(ControlFlow::Continue),
                    _ => Err(InterpretResult::Error {
                        message: format!("JumpIfFalse 操作数类型错误: {:?}", value),
                        source_location: None,
                    }),
                }
            }

            BytecodeInstruction::JumpIfTrue { address } => {
                let value = match self.stack.pop() {
                    Some(v) => v,
                    None => {
                        return Err(InterpretResult::Error {
                            message: "栈下溢: JumpIfTrue".to_string(),
                            source_location: None,
                        });
                    }
                };
                match value {
                    BytecodeValue::Bool(true) => Ok(ControlFlow::Jump(address as usize)),
                    BytecodeValue::Bool(false) => Ok(ControlFlow::Continue),
                    _ => Err(InterpretResult::Error {
                        message: format!("JumpIfTrue 操作数类型错误: {:?}", value),
                        source_location: None,
                    }),
                }
            }

            BytecodeInstruction::Call { arg_count } => {
                let func_name_value = match self.stack.pop() {
                    Some(v) => v,
                    None => {
                        return Err(InterpretResult::Error { message: "栈下溢: Call".to_string(), source_location: None });
                    }
                };

                let func_name = match func_name_value {
                    BytecodeValue::String(s) => s,
                    _ => {
                        return Err(InterpretResult::Error {
                            message: format!("Call 函数名必须是字符串: {:?}", func_name_value),
                            source_location: None,
                        });
                    }
                };

                let function_index = match self.find_function_index(module, &func_name) {
                    Some(idx) => idx,
                    None => {
                        return Err(InterpretResult::Error {
                            message: format!("函数未找到: {}", func_name),
                            source_location: None,
                        });
                    }
                };

                let function = &module.functions[function_index];
                let mut locals = vec![BytecodeValue::Null; function.local_count as usize];
                for i in 0..arg_count.min(function.local_count) as usize {
                    if let Some(val) = self.stack.pop() {
                        locals[i] = val;
                    }
                }

                let stack_base = self.stack.len();
                self.call_stack.push(InterpreterFrame {
                    function_index,
                    function_name: function.name.clone(),
                    ip: 0,
                    locals,
                    stack_base,
                    is_tail_call: false,
                });

                Ok(ControlFlow::Continue)
            }

            BytecodeInstruction::TailCall(arg_count) => {
                let func_name_value = match self.stack.pop() {
                    Some(v) => v,
                    None => {
                        return Err(InterpretResult::Error {
                            message: "栈下溢: TailCall".to_string(), source_location: None
                        });
                    }
                };

                let func_name = match func_name_value {
                    BytecodeValue::String(s) => s,
                    _ => {
                        return Err(InterpretResult::Error {
                            message: format!("TailCall 函数名必须是字符串: {:?}", func_name_value),
                            source_location: None,
                        });
                    }
                };

                let function_index = match self.find_function_index(module, &func_name) {
                    Some(idx) => idx,
                    None => {
                        return Err(InterpretResult::Error {
                            message: format!("函数未找到: {}", func_name),
                            source_location: None,
                        });
                    }
                };

                let function = &module.functions[function_index];
                let mut locals = vec![BytecodeValue::Null; function.local_count as usize];
                for i in 0..arg_count.min(function.local_count) as usize {
                    if let Some(val) = self.stack.pop() {
                        locals[i] = val;
                    }
                }

                if let Some(frame) = self.call_stack.last_mut() {
                    frame.function_index = function_index;
                    frame.function_name = function.name.clone();
                    frame.ip = 0;
                    frame.locals = locals;
                    frame.is_tail_call = true;
                }

                Ok(ControlFlow::Continue)
            }

            BytecodeInstruction::Return => {
                let return_value = self.stack.pop();
                Ok(ControlFlow::Return(return_value))
            }

            BytecodeInstruction::SpawnEntity => {
                let entity_id = host.spawn_entity();
                self.stack.push(BytecodeValue::Entity(entity_id));
                Ok(ControlFlow::Continue)
            }

            BytecodeInstruction::DespawnEntity => {
                let entity_id = match self.stack.pop() {
                    Some(BytecodeValue::Entity(id)) => id,
                    Some(v) => {
                        return Err(InterpretResult::Error {
                            message: format!("DespawnEntity 操作数类型错误: {:?}", v),
                            source_location: None,
                        });
                    }
                    None => {
                        return Err(InterpretResult::Error {
                            message: "栈下溢: DespawnEntity".to_string(),
                            source_location: None,
                        });
                    }
                };
                host.despawn_entity(entity_id);
                Ok(ControlFlow::Continue)
            }

            BytecodeInstruction::AddComponent { type_name_index } => {
                let type_name = match module.string_pool.get(type_name_index as usize) {
                    Some(s) => s.clone(),
                    None => {
                        return Err(InterpretResult::Error {
                            message: format!("字符串池索引越界: {}", type_name_index),
                            source_location: None,
                        });
                    }
                };
                let value = match self.stack.pop() {
                    Some(v) => v,
                    None => {
                        return Err(InterpretResult::Error {
                            message: "栈下溢: AddComponent 值".to_string(),
                            source_location: None,
                        });
                    }
                };
                let entity_id = match self.stack.pop() {
                    Some(BytecodeValue::Entity(id)) => id,
                    Some(v) => {
                        return Err(InterpretResult::Error {
                            message: format!("AddComponent 实体 ID 类型错误: {:?}", v),
                            source_location: None,
                        });
                    }
                    None => {
                        return Err(InterpretResult::Error {
                            message: "栈下溢: AddComponent 实体 ID".to_string(),
                            source_location: None,
                        });
                    }
                };
                host.add_component(entity_id, &type_name, value);
                Ok(ControlFlow::Continue)
            }

            BytecodeInstruction::GetComponent { type_name_index } => {
                let type_name = match module.string_pool.get(type_name_index as usize) {
                    Some(s) => s.clone(),
                    None => {
                        return Err(InterpretResult::Error {
                            message: format!("字符串池索引越界: {}", type_name_index),
                            source_location: None,
                        });
                    }
                };
                let entity_id = match self.stack.pop() {
                    Some(BytecodeValue::Entity(id)) => id,
                    Some(v) => {
                        return Err(InterpretResult::Error {
                            message: format!("GetComponent 实体 ID 类型错误: {:?}", v),
                            source_location: None,
                        });
                    }
                    None => {
                        return Err(InterpretResult::Error {
                            message: "栈下溢: GetComponent 实体 ID".to_string(),
                            source_location: None,
                        });
                    }
                };
                let value = host.get_component_field(entity_id, &type_name, "").unwrap_or(BytecodeValue::Null);
                self.stack.push(value);
                Ok(ControlFlow::Continue)
            }

            BytecodeInstruction::SetComponent { type_name_index } => {
                let type_name = match module.string_pool.get(type_name_index as usize) {
                    Some(s) => s.clone(),
                    None => {
                        return Err(InterpretResult::Error {
                            message: format!("字符串池索引越界: {}", type_name_index),
                            source_location: None,
                        });
                    }
                };
                let value = match self.stack.pop() {
                    Some(v) => v,
                    None => {
                        return Err(InterpretResult::Error {
                            message: "栈下溢: SetComponent 值".to_string(),
                            source_location: None,
                        });
                    }
                };
                let entity_id = match self.stack.pop() {
                    Some(BytecodeValue::Entity(id)) => id,
                    Some(v) => {
                        return Err(InterpretResult::Error {
                            message: format!("SetComponent 实体 ID 类型错误: {:?}", v),
                            source_location: None,
                        });
                    }
                    None => {
                        return Err(InterpretResult::Error {
                            message: "栈下溢: SetComponent 实体 ID".to_string(),
                            source_location: None,
                        });
                    }
                };
                host.set_component_field(entity_id, &type_name, "", value);
                Ok(ControlFlow::Continue)
            }

            BytecodeInstruction::HostCall { name_index, arg_count } => {
                let name = match module.string_pool.get(name_index as usize) {
                    Some(s) => s.clone(),
                    None => {
                        return Err(InterpretResult::Error {
                            message: format!("字符串池索引越界: {}", name_index),
                            source_location: None,
                        });
                    }
                };
                let mut args = Vec::with_capacity(arg_count as usize);
                for _ in 0..arg_count {
                    match self.stack.pop() {
                        Some(v) => args.push(v),
                        None => {
                            return Err(InterpretResult::Error {
                                message: "栈下溢: HostCall 参数不足".to_string(),
                                source_location: None,
                            });
                        }
                    }
                }
                args.reverse();

                let result = host.call_host_function(&name, args);
                match result {
                    Some(v) => self.stack.push(v),
                    None => self.stack.push(BytecodeValue::Null),
                }
                Ok(ControlFlow::Continue)
            }

            BytecodeInstruction::Pop => {
                self.stack.pop();
                Ok(ControlFlow::Continue)
            }

            BytecodeInstruction::Dup => {
                let value = match self.stack.last() {
                    Some(v) => v.clone(),
                    None => {
                        return Err(InterpretResult::Error { message: "栈下溢: Dup".to_string(), source_location: None });
                    }
                };
                self.stack.push(value);
                Ok(ControlFlow::Continue)
            }

            BytecodeInstruction::GetField { name_index } => {
                let field_name = match module.string_pool.get(name_index as usize) {
                    Some(s) => s.clone(),
                    None => {
                        return Err(InterpretResult::Error {
                            message: format!("字符串池索引越界: {}", name_index),
                            source_location: None,
                        });
                    }
                };
                let obj = match self.stack.pop() {
                    Some(v) => v,
                    None => {
                        return Err(InterpretResult::Error {
                            message: "栈下溢: GetField".to_string(), source_location: None
                        });
                    }
                };
                match obj {
                    BytecodeValue::Object(fields) => {
                        let value = fields
                            .iter()
                            .find(|(k, _)| k == &field_name)
                            .map(|(_, v)| v.clone())
                            .unwrap_or(BytecodeValue::Null);
                        self.stack.push(value);
                    }
                    _ => {
                        return Err(InterpretResult::Error {
                            message: format!("GetField 操作数类型错误: 期望 Object, 实际 {:?}", obj),
                            source_location: None,
                        });
                    }
                }
                Ok(ControlFlow::Continue)
            }
            BytecodeInstruction::SetField { name_index } => {
                let field_name = match module.string_pool.get(name_index as usize) {
                    Some(s) => s.clone(),
                    None => {
                        return Err(InterpretResult::Error {
                            message: format!("字符串池索引越界: {}", name_index),
                            source_location: None,
                        });
                    }
                };
                let value = match self.stack.pop() {
                    Some(v) => v,
                    None => {
                        return Err(InterpretResult::Error {
                            message: "栈下溢: SetField 值".to_string(),
                            source_location: None,
                        });
                    }
                };
                let obj = match self.stack.pop() {
                    Some(v) => v,
                    None => {
                        return Err(InterpretResult::Error {
                            message: "栈下溢: SetField 对象".to_string(),
                            source_location: None,
                        });
                    }
                };
                match obj {
                    BytecodeValue::Object(mut fields) => {
                        if let Some(entry) = fields.iter_mut().find(|(k, _)| k == &field_name) {
                            entry.1 = value;
                        }
                        else {
                            fields.push((field_name, value));
                        }
                        self.stack.push(BytecodeValue::Object(fields));
                    }
                    _ => {
                        return Err(InterpretResult::Error {
                            message: format!("SetField 操作数类型错误: 期望 Object, 实际 {:?}", obj),
                            source_location: None,
                        });
                    }
                }
                Ok(ControlFlow::Continue)
            }
            BytecodeInstruction::GetIndex => {
                let index = match self.stack.pop() {
                    Some(v) => v,
                    None => {
                        return Err(InterpretResult::Error {
                            message: "栈下溢: GetIndex 索引".to_string(),
                            source_location: None,
                        });
                    }
                };
                let container = match self.stack.pop() {
                    Some(v) => v,
                    None => {
                        return Err(InterpretResult::Error {
                            message: "栈下溢: GetIndex 容器".to_string(),
                            source_location: None,
                        });
                    }
                };
                match container {
                    BytecodeValue::List(items) => match index {
                        BytecodeValue::Int(i) => {
                            let value = items.get(i as usize).cloned().unwrap_or(BytecodeValue::Null);
                            self.stack.push(value);
                        }
                        _ => {
                            return Err(InterpretResult::Error {
                                message: format!("GetIndex 列表索引类型错误: {:?}", index),
                                source_location: None,
                            });
                        }
                    },
                    BytecodeValue::Map(entries) => {
                        let value =
                            entries.iter().find(|(k, _)| *k == index).map(|(_, v)| v.clone()).unwrap_or(BytecodeValue::Null);
                        self.stack.push(value);
                    }
                    _ => {
                        return Err(InterpretResult::Error {
                            message: format!("GetIndex 操作数类型错误: 期望 List 或 Map, 实际 {:?}", container),
                            source_location: None,
                        });
                    }
                }
                Ok(ControlFlow::Continue)
            }
            BytecodeInstruction::SetIndex => {
                let value = match self.stack.pop() {
                    Some(v) => v,
                    None => {
                        return Err(InterpretResult::Error {
                            message: "栈下溢: SetIndex 值".to_string(),
                            source_location: None,
                        });
                    }
                };
                let index = match self.stack.pop() {
                    Some(v) => v,
                    None => {
                        return Err(InterpretResult::Error {
                            message: "栈下溢: SetIndex 索引".to_string(),
                            source_location: None,
                        });
                    }
                };
                let container = match self.stack.pop() {
                    Some(v) => v,
                    None => {
                        return Err(InterpretResult::Error {
                            message: "栈下溢: SetIndex 容器".to_string(),
                            source_location: None,
                        });
                    }
                };
                match container {
                    BytecodeValue::List(mut items) => match index {
                        BytecodeValue::Int(i) => {
                            let idx = i as usize;
                            if idx < items.len() {
                                items[idx] = value;
                            }
                            else {
                                return Err(InterpretResult::Error {
                                    message: format!("SetIndex 列表索引越界: {}", i),
                                    source_location: None,
                                });
                            }
                            self.stack.push(BytecodeValue::List(items));
                        }
                        _ => {
                            return Err(InterpretResult::Error {
                                message: format!("SetIndex 列表索引类型错误: {:?}", index),
                                source_location: None,
                            });
                        }
                    },
                    BytecodeValue::Map(mut entries) => {
                        if let Some(entry) = entries.iter_mut().find(|(k, _)| *k == index) {
                            entry.1 = value;
                        }
                        else {
                            entries.push((index, value));
                        }
                        self.stack.push(BytecodeValue::Map(entries));
                    }
                    _ => {
                        return Err(InterpretResult::Error {
                            message: format!("SetIndex 操作数类型错误: 期望 List 或 Map, 实际 {:?}", container),
                            source_location: None,
                        });
                    }
                }
                Ok(ControlFlow::Continue)
            }
            BytecodeInstruction::NewObject { field_count } => {
                let mut fields = Vec::with_capacity(field_count as usize);
                for _ in 0..field_count {
                    let value = match self.stack.pop() {
                        Some(v) => v,
                        None => {
                            return Err(InterpretResult::Error {
                                message: "栈下溢: NewObject 值".to_string(),
                                source_location: None,
                            });
                        }
                    };
                    let key = match self.stack.pop() {
                        Some(BytecodeValue::String(s)) => s,
                        Some(v) => {
                            return Err(InterpretResult::Error {
                                message: format!("NewObject 字段名必须是字符串: {:?}", v),
                                source_location: None,
                            });
                        }
                        None => {
                            return Err(InterpretResult::Error {
                                message: "栈下溢: NewObject 键".to_string(),
                                source_location: None,
                            });
                        }
                    };
                    fields.push((key, value));
                }
                fields.reverse();
                self.stack.push(BytecodeValue::Object(fields));
                Ok(ControlFlow::Continue)
            }
            BytecodeInstruction::NewList { element_count } => {
                let mut items = Vec::with_capacity(element_count as usize);
                for _ in 0..element_count {
                    match self.stack.pop() {
                        Some(v) => items.push(v),
                        None => {
                            return Err(InterpretResult::Error {
                                message: "栈下溢: NewList 元素".to_string(),
                                source_location: None,
                            });
                        }
                    }
                }
                items.reverse();
                self.stack.push(BytecodeValue::List(items));
                Ok(ControlFlow::Continue)
            }
            BytecodeInstruction::NewMap { pair_count } => {
                let mut entries = Vec::with_capacity(pair_count as usize);
                for _ in 0..pair_count {
                    let value = match self.stack.pop() {
                        Some(v) => v,
                        None => {
                            return Err(InterpretResult::Error {
                                message: "栈下溢: NewMap 值".to_string(),
                                source_location: None,
                            });
                        }
                    };
                    let key = match self.stack.pop() {
                        Some(v) => v,
                        None => {
                            return Err(InterpretResult::Error {
                                message: "栈下溢: NewMap 键".to_string(),
                                source_location: None,
                            });
                        }
                    };
                    entries.push((key, value));
                }
                entries.reverse();
                self.stack.push(BytecodeValue::Map(entries));
                Ok(ControlFlow::Continue)
            }
            BytecodeInstruction::StringConcat { count } => {
                let mut parts = Vec::with_capacity(count as usize);
                for _ in 0..count {
                    match self.stack.pop() {
                        Some(v) => parts.push(v),
                        None => {
                            return Err(InterpretResult::Error {
                                message: "栈下溢: StringConcat".to_string(),
                                source_location: None,
                            });
                        }
                    }
                }
                parts.reverse();
                let result: String = parts
                    .iter()
                    .map(|v| match v {
                        BytecodeValue::String(s) => s.clone(),
                        BytecodeValue::Int(i) => i.to_string(),
                        BytecodeValue::Float(f) => f.to_string(),
                        BytecodeValue::Bool(b) => b.to_string(),
                        BytecodeValue::Entity(id) => id.to_string(),
                        BytecodeValue::Null => "null".to_string(),
                        BytecodeValue::List(_) => "[list]".to_string(),
                        BytecodeValue::Object(_) => "[object]".to_string(),
                        BytecodeValue::Map(_) => "[map]".to_string(),
                    })
                    .collect();
                self.stack.push(BytecodeValue::String(result));
                Ok(ControlFlow::Continue)
            }
        }
    }

    /// 执行二元算术操作
    fn binary_op<F>(&mut self, op: F) -> Result<ControlFlow, InterpretResult>
    where
        F: FnOnce(BytecodeValue, BytecodeValue) -> BytecodeValue,
    {
        let b = match self.stack.pop() {
            Some(v) => v,
            None => {
                return Err(InterpretResult::Error {
                    message: "栈下溢: 二元操作右操作数".to_string(), source_location: None
                });
            }
        };
        let a = match self.stack.pop() {
            Some(v) => v,
            None => {
                return Err(InterpretResult::Error {
                    message: "栈下溢: 二元操作左操作数".to_string(), source_location: None
                });
            }
        };
        let result = op(a, b);
        self.stack.push(result);
        Ok(ControlFlow::Continue)
    }

    /// 执行比较操作
    fn compare_op<F>(&mut self, op: F) -> Result<ControlFlow, InterpretResult>
    where
        F: FnOnce(&BytecodeValue, &BytecodeValue) -> bool,
    {
        let b = match self.stack.pop() {
            Some(v) => v,
            None => {
                return Err(InterpretResult::Error {
                    message: "栈下溢: 比较操作右操作数".to_string(), source_location: None
                });
            }
        };
        let a = match self.stack.pop() {
            Some(v) => v,
            None => {
                return Err(InterpretResult::Error {
                    message: "栈下溢: 比较操作左操作数".to_string(), source_location: None
                });
            }
        };
        let result = op(&a, &b);
        self.stack.push(BytecodeValue::Bool(result));
        Ok(ControlFlow::Continue)
    }
}

impl Default for BytecodeInterpreter {
    fn default() -> Self {
        Self::new()
    }
}

/// 指令执行控制流
enum ControlFlow {
    /// 继续执行下一条指令
    Continue,
    /// 跳转到指定地址
    Jump(usize),
    /// 从函数返回
    Return(Option<BytecodeValue>),
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        format::{BytecodeFunction, BytecodeInstruction, BytecodeModule, BytecodeValue},
        host::Host,
    };
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
}
