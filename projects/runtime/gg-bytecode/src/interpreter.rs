use std::cell::RefCell;
use std::rc::Rc;

use crate::{
    debug_info::SourceLocation,
    debug_protocol::DebugController,
    format::{BytecodeInstruction, BytecodeModule, BytecodeValue},
    host::Host,
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
    /// 函数名称
    pub function_name: String,
    /// 指令指针
    pub ip: usize,
    /// 局部变量
    pub locals: Vec<BytecodeValue>,
    /// 栈基指针
    pub stack_base: usize,
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
}

impl BytecodeInterpreter {
    /// 创建新的字节码解释器
    pub fn new() -> Self {
        Self {
            stack: Vec::new(),
            call_stack: Vec::new(),
            running: false,
            debug_controller: None,
            debug_paused: false,
            debug_initial_depth: 0,
            debug_skip_check: false,
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
            Some(frame) => frame
                .locals
                .iter()
                .enumerate()
                .map(|(i, v)| (format!("local_{}", i), v.clone()))
                .collect(),
            None => Vec::new(),
        }
    }

    /// 执行模块中的指定函数
    pub fn execute<H: Host>(
        &mut self,
        module: &BytecodeModule,
        function_name: &str,
        host: &mut H,
    ) -> InterpretResult {
        if self.debug_paused {
            return InterpretResult::Ok;
        }

        let initial_depth = if self.call_stack.is_empty() {
            let function = match module.find_function(function_name) {
                Some(f) => f.clone(),
                None => {
                    return InterpretResult::Error {
                        message: format!("函数未找到: {}", function_name),
                        source_location: None,
                    };
                }
            };

            let mut locals = vec![BytecodeValue::Null; function.local_count as usize];
            for i in 0..function.param_count.min(function.local_count) as usize {
                if let Some(val) = self.stack.pop() {
                    locals[i] = val;
                }
            }

            let stack_base = self.stack.len();
            let depth = self.call_stack.len();

            self.call_stack.push(InterpreterFrame {
                function_name: function.name,
                ip: 0,
                locals,
                stack_base,
            });

            self.debug_initial_depth = depth;
            self.debug_skip_check = false;
            depth
        } else {
            self.debug_initial_depth
        };

        self.running = true;
        let result = self.execute_instructions(module, host, initial_depth);
        self.running = false;

        self.enrich_error_with_source_location(module, result)
    }

    /// 为错误结果补充源码位置信息
    fn enrich_error_with_source_location(
        &self,
        module: &BytecodeModule,
        result: InterpretResult,
    ) -> InterpretResult {
        match result {
            InterpretResult::Error {
                message,
                source_location: None,
            } => {
                let source_location = self.resolve_source_location(module);
                InterpretResult::Error {
                    message,
                    source_location,
                }
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

            let (function_name, ip) = match self.call_stack.last() {
                Some(f) => (f.function_name.clone(), f.ip),
                None => return InterpretResult::Ok,
            };

            let function = match module.find_function(&function_name) {
                Some(f) => f,
                None => {
                    return InterpretResult::Error {
                        message: format!("函数未找到: {}", function_name),
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
                } else {
                    let source_location = module
                        .debug_info
                        .as_ref()
                        .and_then(|di| di.lookup(ip as u32));

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

                    let step_completed = step_was_active
                        && {
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
            let is_call = matches!(op, BytecodeInstruction::Call { .. });
            let is_return = matches!(op, BytecodeInstruction::Return);

            if let Some(f) = self.call_stack.last_mut() {
                f.ip += 1;
            } else {
                return InterpretResult::Error {
                    message: "调用栈为空".to_string(),
                    source_location: None,
                };
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
                        controller
                            .borrow_mut()
                            .on_function_call(&frame.function_name, frame.ip);
                    }
                }
            }

            if is_return {
                if let Some(ref controller) = self.debug_controller {
                    controller.borrow_mut().on_function_return(ip);
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
                        return Err(InterpretResult::Error {
                            message: "调用栈为空".to_string(),
                            source_location: None,
                        });
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
                        return Err(InterpretResult::Error {
                            message: "调用栈为空".to_string(),
                            source_location: None,
                        });
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
                    } else {
                        BytecodeValue::Int(x / y)
                    }
                }
                (BytecodeValue::Float(x), BytecodeValue::Float(y)) => {
                    if y == 0.0 {
                        BytecodeValue::Null
                    } else {
                        BytecodeValue::Float(x / y)
                    }
                }
                _ => BytecodeValue::Null,
            }),

            BytecodeInstruction::Mod => self.binary_op(|a, b| match (a, b) {
                (BytecodeValue::Int(x), BytecodeValue::Int(y)) => {
                    if y == 0 {
                        BytecodeValue::Null
                    } else {
                        BytecodeValue::Int(x % y)
                    }
                }
                (BytecodeValue::Float(x), BytecodeValue::Float(y)) => {
                    if y == 0.0 {
                        BytecodeValue::Null
                    } else {
                        BytecodeValue::Float(x % y)
                    }
                }
                _ => BytecodeValue::Null,
            }),

            BytecodeInstruction::Neg => {
                let value = match self.stack.pop() {
                    Some(v) => v,
                    None => {
                        return Err(InterpretResult::Error {
                            message: "栈下溢: Neg".to_string(),
                            source_location: None,
                        });
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
                        return Err(InterpretResult::Error {
                            message: "栈下溢: And".to_string(),
                            source_location: None,
                        });
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
                        return Err(InterpretResult::Error {
                            message: "栈下溢: And".to_string(),
                            source_location: None,
                        });
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
                        return Err(InterpretResult::Error {
                            message: "栈下溢: Or".to_string(),
                            source_location: None,
                        });
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
                        return Err(InterpretResult::Error {
                            message: "栈下溢: Or".to_string(),
                            source_location: None,
                        });
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
                        return Err(InterpretResult::Error {
                            message: "栈下溢: Not".to_string(),
                            source_location: None,
                        });
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
                        return Err(InterpretResult::Error {
                            message: "栈下溢: Call".to_string(),
                            source_location: None,
                        });
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

                let function = match module.find_function(&func_name) {
                    Some(f) => f.clone(),
                    None => {
                        return Err(InterpretResult::Error {
                            message: format!("函数未找到: {}", func_name),
                            source_location: None,
                        });
                    }
                };

                let mut locals = vec![BytecodeValue::Null; function.local_count as usize];
                for i in 0..arg_count.min(function.local_count) as usize {
                    if let Some(val) = self.stack.pop() {
                        locals[i] = val;
                    }
                }

                let stack_base = self.stack.len();
                self.call_stack.push(InterpreterFrame {
                    function_name: function.name,
                    ip: 0,
                    locals,
                    stack_base,
                });

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
                let value = host
                    .get_component_field(entity_id, &type_name, "")
                    .unwrap_or(BytecodeValue::Null);
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

            BytecodeInstruction::HostCall {
                name_index,
                arg_count,
            } => {
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
                        return Err(InterpretResult::Error {
                            message: "栈下溢: Dup".to_string(),
                            source_location: None,
                        });
                    }
                };
                self.stack.push(value);
                Ok(ControlFlow::Continue)
            }

            BytecodeInstruction::GetField { .. } => Ok(ControlFlow::Continue),
            BytecodeInstruction::SetField { .. } => {
                self.stack.pop();
                Ok(ControlFlow::Continue)
            }
            BytecodeInstruction::GetIndex => {
                self.stack.pop();
                Ok(ControlFlow::Continue)
            }
            BytecodeInstruction::SetIndex => {
                self.stack.pop();
                self.stack.pop();
                self.stack.pop();
                Ok(ControlFlow::Continue)
            }
            BytecodeInstruction::NewObject { field_count } => {
                for _ in 0..field_count {
                    self.stack.pop();
                }
                self.stack.push(BytecodeValue::Null);
                Ok(ControlFlow::Continue)
            }
            BytecodeInstruction::NewList { element_count } => {
                for _ in 0..element_count {
                    self.stack.pop();
                }
                self.stack.push(BytecodeValue::Null);
                Ok(ControlFlow::Continue)
            }
            BytecodeInstruction::NewMap { pair_count } => {
                for _ in 0..pair_count * 2 {
                    self.stack.pop();
                }
                self.stack.push(BytecodeValue::Null);
                Ok(ControlFlow::Continue)
            }
            BytecodeInstruction::StringConcat { count } => {
                for _ in 0..count {
                    self.stack.pop();
                }
                self.stack.push(BytecodeValue::Null);
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
                    message: "栈下溢: 二元操作右操作数".to_string(),
                    source_location: None,
                });
            }
        };
        let a = match self.stack.pop() {
            Some(v) => v,
            None => {
                return Err(InterpretResult::Error {
                    message: "栈下溢: 二元操作左操作数".to_string(),
                    source_location: None,
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
                    message: "栈下溢: 比较操作右操作数".to_string(),
                    source_location: None,
                });
            }
        };
        let a = match self.stack.pop() {
            Some(v) => v,
            None => {
                return Err(InterpretResult::Error {
                    message: "栈下溢: 比较操作左操作数".to_string(),
                    source_location: None,
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
    use crate::debug_info::DebugInfo;
    use crate::debug_protocol::{BasicDebugController, StepMode};
    use crate::format::{BytecodeFunction, BytecodeModule, BytecodeValue};
    use crate::host::Host;

    struct MockHost;

    impl Host for MockHost {
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
            instructions: vec![
                BytecodeInstruction::LoadNull,
                BytecodeInstruction::Return,
            ],
        });

        let mut debug_info = DebugInfo::new();
        debug_info.add_entry(0, SourceLocation::new("test.gg", 1, 1));
        debug_info.add_entry(1, SourceLocation::new("test.gg", 2, 1));
        module.debug_info = Some(debug_info);

        let controller = Rc::new(RefCell::new(BasicDebugController::new()));
        controller.borrow_mut().add_file_breakpoint("test.gg", 1);

        let mut interpreter = BytecodeInterpreter::new();
        interpreter.set_debug_controller(Some(controller));

        let mut host = MockHost;
        let result = interpreter.execute(&module, "main", &mut host);

        assert!(matches!(result, InterpretResult::Ok));
        assert!(interpreter.is_debug_paused());

        let stack = interpreter.debug_call_stack();
        assert_eq!(stack.len(), 1);
        assert_eq!(stack[0].function_name, "main");
        assert_eq!(stack[0].ip, 0);
    }

    #[test]
    fn test_step_over_mode() {
        let mut module = BytecodeModule::new("test");
        module.add_function(BytecodeFunction {
            name: "main".to_string(),
            param_count: 0,
            local_count: 0,
            instructions: vec![
                BytecodeInstruction::LoadNull,
                BytecodeInstruction::LoadNull,
                BytecodeInstruction::Return,
            ],
        });

        let controller = Rc::new(RefCell::new(BasicDebugController::new()));
        controller.borrow_mut().set_step_mode(StepMode::Over);

        let mut interpreter = BytecodeInterpreter::new();
        interpreter.set_debug_controller(Some(controller));

        let mut host = MockHost;
        let result = interpreter.execute(&module, "main", &mut host);

        assert!(matches!(result, InterpretResult::Ok));
        assert!(interpreter.is_debug_paused());
    }

    #[test]
    fn test_debug_call_stack() {
        let mut module = BytecodeModule::new("test");
        module.add_function(BytecodeFunction {
            name: "main".to_string(),
            param_count: 0,
            local_count: 2,
            instructions: vec![
                BytecodeInstruction::LoadNull,
                BytecodeInstruction::Return,
            ],
        });

        let mut debug_info = DebugInfo::new();
        debug_info.add_entry(0, SourceLocation::new("test.gg", 1, 1));
        module.debug_info = Some(debug_info);

        let controller = Rc::new(RefCell::new(BasicDebugController::new()));
        controller.borrow_mut().add_file_breakpoint("test.gg", 1);

        let mut interpreter = BytecodeInterpreter::new();
        interpreter.set_debug_controller(Some(controller));

        let mut host = MockHost;
        interpreter.execute(&module, "main", &mut host);

        assert!(interpreter.is_debug_paused());

        let stack = interpreter.debug_call_stack();
        assert_eq!(stack.len(), 1);
        assert_eq!(stack[0].function_name, "main");
        assert_eq!(stack[0].ip, 0);
        assert_eq!(stack[0].locals.len(), 2);
    }

    #[test]
    fn test_debug_local_variables() {
        let mut module = BytecodeModule::new("test");
        module.add_constant(BytecodeValue::Int(42));
        module.add_function(BytecodeFunction {
            name: "main".to_string(),
            param_count: 0,
            local_count: 2,
            instructions: vec![
                BytecodeInstruction::LoadConst { index: 0 },
                BytecodeInstruction::StoreLocal { index: 0 },
                BytecodeInstruction::Return,
            ],
        });

        let mut debug_info = DebugInfo::new();
        debug_info.add_entry(0, SourceLocation::new("test.gg", 1, 1));
        debug_info.add_entry(1, SourceLocation::new("test.gg", 2, 1));
        debug_info.add_entry(2, SourceLocation::new("test.gg", 3, 1));
        module.debug_info = Some(debug_info);

        let controller = Rc::new(RefCell::new(BasicDebugController::new()));
        controller.borrow_mut().add_file_breakpoint("test.gg", 3);

        let mut interpreter = BytecodeInterpreter::new();
        interpreter.set_debug_controller(Some(controller));

        let mut host = MockHost;
        interpreter.execute(&module, "main", &mut host);

        assert!(interpreter.is_debug_paused());

        let vars = interpreter.debug_local_variables(0);
        assert_eq!(vars.len(), 2);
        assert_eq!(vars[0].0, "local_0");
        assert_eq!(vars[0].1, BytecodeValue::Int(42));
        assert_eq!(vars[1].0, "local_1");
        assert_eq!(vars[1].1, BytecodeValue::Null);

        let empty_vars = interpreter.debug_local_variables(99);
        assert!(empty_vars.is_empty());
    }

    #[test]
    fn test_resume_after_pause() {
        let mut module = BytecodeModule::new("test");
        module.add_function(BytecodeFunction {
            name: "main".to_string(),
            param_count: 0,
            local_count: 0,
            instructions: vec![
                BytecodeInstruction::LoadNull,
                BytecodeInstruction::Return,
            ],
        });

        let mut debug_info = DebugInfo::new();
        debug_info.add_entry(0, SourceLocation::new("test.gg", 1, 1));
        debug_info.add_entry(1, SourceLocation::new("test.gg", 2, 1));
        module.debug_info = Some(debug_info);

        let controller = Rc::new(RefCell::new(BasicDebugController::new()));
        controller.borrow_mut().add_file_breakpoint("test.gg", 1);

        let mut interpreter = BytecodeInterpreter::new();
        interpreter.set_debug_controller(Some(controller));

        let mut host = MockHost;
        interpreter.execute(&module, "main", &mut host);

        assert!(interpreter.is_debug_paused());

        interpreter.resume();
        assert!(!interpreter.is_debug_paused());

        let result = interpreter.execute(&module, "main", &mut host);
        assert!(!interpreter.is_debug_paused());
        assert!(matches!(result, InterpretResult::Return(_)));
    }

    #[test]
    fn test_no_debug_controller() {
        let mut module = BytecodeModule::new("test");
        module.add_function(BytecodeFunction {
            name: "main".to_string(),
            param_count: 0,
            local_count: 0,
            instructions: vec![
                BytecodeInstruction::LoadNull,
                BytecodeInstruction::Return,
            ],
        });

        let mut interpreter = BytecodeInterpreter::new();

        let mut host = MockHost;
        let result = interpreter.execute(&module, "main", &mut host);

        assert!(!interpreter.is_debug_paused());
        assert!(matches!(result, InterpretResult::Return(_)));
    }

    #[test]
    fn test_step_into_mode() {
        let mut module = BytecodeModule::new("test");
        module.add_function(BytecodeFunction {
            name: "main".to_string(),
            param_count: 0,
            local_count: 0,
            instructions: vec![
                BytecodeInstruction::LoadNull,
                BytecodeInstruction::Return,
            ],
        });

        let controller = Rc::new(RefCell::new(BasicDebugController::new()));
        controller.borrow_mut().set_step_mode(StepMode::Into);

        let mut interpreter = BytecodeInterpreter::new();
        interpreter.set_debug_controller(Some(controller));

        let mut host = MockHost;
        let result = interpreter.execute(&module, "main", &mut host);

        assert!(matches!(result, InterpretResult::Ok));
        assert!(interpreter.is_debug_paused());
    }
}
