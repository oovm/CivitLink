use crate::{
    debug_info::SourceLocation,
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
}

impl BytecodeInterpreter {
    /// 创建新的字节码解释器
    pub fn new() -> Self {
        Self { stack: Vec::new(), call_stack: Vec::new(), running: false }
    }

    /// 执行模块中的指定函数
    pub fn execute<H: Host>(&mut self, module: &BytecodeModule, function_name: &str, host: &mut H) -> InterpretResult {
        let function = match module.find_function(function_name) {
            Some(f) => f.clone(),
            None => {
                return InterpretResult::Error {
                    message: format!("函数未找到: {}", function_name), source_location: None
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
        let initial_depth = self.call_stack.len();

        self.call_stack.push(InterpreterFrame { function_name: function.name, ip: 0, locals, stack_base });

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

            let (function_name, ip) = match self.call_stack.last() {
                Some(f) => (f.function_name.clone(), f.ip),
                None => return InterpretResult::Ok,
            };

            let function = match module.find_function(&function_name) {
                Some(f) => f,
                None => {
                    return InterpretResult::Error {
                        message: format!("函数未找到: {}", function_name), source_location: None
                    };
                }
            };

            let instructions = &function.instructions;
            if ip >= instructions.len() {
                self.call_stack.pop();
                continue;
            }

            let op = instructions[ip].clone();

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
                self.call_stack.push(InterpreterFrame { function_name: function.name, ip: 0, locals, stack_base });

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
