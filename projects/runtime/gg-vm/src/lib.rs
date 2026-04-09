#![warn(missing_docs)]

//! GG 引擎字节码虚拟机模块
//! 执行 gg-ir 生成的中间表示指令

use gg_ir::IrModule;
use gg_ir::IrValue;
use gg_ir::OpCode;

/// 宿主接口，允许 VM 调用引擎 Rust API
pub trait Host {
    /// 创建实体，返回实体 ID
    fn spawn_entity(&mut self) -> u64;

    /// 销毁实体
    fn despawn_entity(&mut self, entity_id: u64);

    /// 添加组件
    fn add_component(&mut self, entity_id: u64, component_type: &str, value: IrValue);

    /// 获取组件字段值
    fn get_component_field(
        &mut self,
        entity_id: u64,
        component_type: &str,
        field: &str,
    ) -> Option<IrValue>;

    /// 设置组件字段值
    fn set_component_field(
        &mut self,
        entity_id: u64,
        component_type: &str,
        field: &str,
        value: IrValue,
    );

    /// 调用宿主函数
    fn call_host_function(&mut self, name: &str, args: Vec<IrValue>) -> Option<IrValue>;
}

/// 虚拟机执行结果
#[derive(Debug, Clone)]
pub enum VmResult {
    /// 正常完成
    Ok,

    /// 返回值
    Return(IrValue),

    /// 调用宿主函数
    HostCall {
        /// 函数名
        name: String,
        /// 参数
        args: Vec<IrValue>,
    },

    /// 运行时错误
    Error(String),
}

/// 调用栈帧
pub struct CallFrame {
    /// 函数名称
    pub function_name: String,
    /// 指令指针
    pub ip: usize,
    /// 局部变量
    pub locals: Vec<IrValue>,
    /// 栈基指针（栈中该帧的起始位置）
    pub stack_base: usize,
}

/// 字节码虚拟机
pub struct Vm {
    /// 执行栈
    pub stack: Vec<IrValue>,
    /// 调用栈帧
    pub call_stack: Vec<CallFrame>,
    /// 是否正在运行
    pub running: bool,
}

impl Vm {
    /// 创建新的虚拟机
    pub fn new() -> Self {
        Self {
            stack: Vec::new(),
            call_stack: Vec::new(),
            running: false,
        }
    }

    /// 执行模块中的指定函数
    pub fn execute<H: Host>(
        &mut self,
        module: &IrModule,
        function_name: &str,
        host: &mut H,
    ) -> VmResult {
        let function = match module.find_function(function_name) {
            Some(f) => f.clone(),
            None => {
                return VmResult::Error(format!("函数未找到: {}", function_name));
            }
        };

        let mut locals = vec![IrValue::Null; function.local_count];
        for i in 0..function.param_count.min(function.local_count) {
            if let Some(val) = self.stack.pop() {
                locals[i] = val;
            }
        }

        let stack_base = self.stack.len();

        self.call_stack.push(CallFrame {
            function_name: function.name,
            ip: 0,
            locals,
            stack_base,
        });

        self.running = true;
        let result = self.execute_instructions(module, host);
        self.running = false;
        result
    }

    /// 执行当前栈帧的指令序列
    fn execute_instructions<H: Host>(
        &mut self,
        module: &IrModule,
        host: &mut H,
    ) -> VmResult {
        while self.running {
            let (function_name, ip) = match self.call_stack.last() {
                Some(f) => (f.function_name.clone(), f.ip),
                None => return VmResult::Ok,
            };

            let function = match module.find_function(&function_name) {
                Some(f) => f,
                None => return VmResult::Error(format!("函数未找到: {}", function_name)),
            };

            let instructions = &function.instructions;
            if ip >= instructions.len() {
                self.call_stack.pop();
                if self.call_stack.is_empty() {
                    return VmResult::Ok;
                }
                continue;
            }

            let op = instructions[ip].clone();

            match self.call_stack.last_mut() {
                Some(f) => f.ip += 1,
                None => return VmResult::Error("调用栈为空".to_string()),
            };

            match self.dispatch_op(module, host, op) {
                Ok(ControlFlow::Continue) => {}
                Ok(ControlFlow::Jump(addr)) => {
                    if let Some(f) = self.call_stack.last_mut() {
                        f.ip = addr;
                    }
                }
                Ok(ControlFlow::Return(value)) => {
                    self.call_stack.pop();
                    if let Some(value) = value {
                        self.stack.push(value);
                    }
                    if self.call_stack.is_empty() {
                        return VmResult::Ok;
                    }
                }
                Err(vm_result) => return vm_result,
            }
        }

        VmResult::Ok
    }

    /// 分发操作码执行
    fn dispatch_op<H: Host>(
        &mut self,
        module: &IrModule,
        host: &mut H,
        op: OpCode,
    ) -> Result<ControlFlow, VmResult> {
        match op {
            OpCode::LoadConst(idx) => {
                let value = match module.constants.get(idx) {
                    Some(v) => v.clone(),
                    None => {
                        return Err(VmResult::Error(format!(
                            "常量索引越界: {}",
                            idx
                        )));
                    }
                };
                self.stack.push(value);
                Ok(ControlFlow::Continue)
            }

            OpCode::LoadNull => {
                self.stack.push(IrValue::Null);
                Ok(ControlFlow::Continue)
            }

            OpCode::LoadTrue => {
                self.stack.push(IrValue::Bool(true));
                Ok(ControlFlow::Continue)
            }

            OpCode::LoadFalse => {
                self.stack.push(IrValue::Bool(false));
                Ok(ControlFlow::Continue)
            }

            OpCode::LoadLocal(idx) => {
                let frame = match self.call_stack.last() {
                    Some(f) => f,
                    None => return Err(VmResult::Error("调用栈为空".to_string())),
                };
                let value = match frame.locals.get(idx) {
                    Some(v) => v.clone(),
                    None => {
                        return Err(VmResult::Error(format!(
                            "局部变量索引越界: {}",
                            idx
                        )));
                    }
                };
                self.stack.push(value);
                Ok(ControlFlow::Continue)
            }

            OpCode::StoreLocal(idx) => {
                let value = match self.stack.pop() {
                    Some(v) => v,
                    None => return Err(VmResult::Error("栈下溢: StoreLocal".to_string())),
                };
                let frame = match self.call_stack.last_mut() {
                    Some(f) => f,
                    None => return Err(VmResult::Error("调用栈为空".to_string())),
                };
                if idx >= frame.locals.len() {
                    return Err(VmResult::Error(format!(
                        "局部变量索引越界: {}",
                        idx
                    )));
                }
                frame.locals[idx] = value;
                Ok(ControlFlow::Continue)
            }

            OpCode::Add => self.binary_op(|a, b| match (a, b) {
                (IrValue::Int(x), IrValue::Int(y)) => IrValue::Int(x + y),
                (IrValue::Float(x), IrValue::Float(y)) => IrValue::Float(x + y),
                _ => IrValue::Null,
            }),

            OpCode::Sub => self.binary_op(|a, b| match (a, b) {
                (IrValue::Int(x), IrValue::Int(y)) => IrValue::Int(x - y),
                (IrValue::Float(x), IrValue::Float(y)) => IrValue::Float(x - y),
                _ => IrValue::Null,
            }),

            OpCode::Mul => self.binary_op(|a, b| match (a, b) {
                (IrValue::Int(x), IrValue::Int(y)) => IrValue::Int(x * y),
                (IrValue::Float(x), IrValue::Float(y)) => IrValue::Float(x * y),
                _ => IrValue::Null,
            }),

            OpCode::Div => self.binary_op(|a, b| match (a, b) {
                (IrValue::Int(x), IrValue::Int(y)) => {
                    if y == 0 {
                        IrValue::Null
                    } else {
                        IrValue::Int(x / y)
                    }
                }
                (IrValue::Float(x), IrValue::Float(y)) => {
                    if y == 0.0 {
                        IrValue::Null
                    } else {
                        IrValue::Float(x / y)
                    }
                }
                _ => IrValue::Null,
            }),

            OpCode::Mod => self.binary_op(|a, b| match (a, b) {
                (IrValue::Int(x), IrValue::Int(y)) => {
                    if y == 0 {
                        IrValue::Null
                    } else {
                        IrValue::Int(x % y)
                    }
                }
                (IrValue::Float(x), IrValue::Float(y)) => {
                    if y == 0.0 {
                        IrValue::Null
                    } else {
                        IrValue::Float(x % y)
                    }
                }
                _ => IrValue::Null,
            }),

            OpCode::Neg => {
                let value = match self.stack.pop() {
                    Some(v) => v,
                    None => return Err(VmResult::Error("栈下溢: Neg".to_string())),
                };
                let result = match value {
                    IrValue::Int(x) => IrValue::Int(-x),
                    IrValue::Float(x) => IrValue::Float(-x),
                    _ => {
                        return Err(VmResult::Error(format!(
                            "Neg 操作数类型错误: {:?}",
                            value
                        )));
                    }
                };
                self.stack.push(result);
                Ok(ControlFlow::Continue)
            }

            OpCode::Eq => self.compare_op(|a, b| a == b),

            OpCode::Ne => self.compare_op(|a, b| a != b),

            OpCode::Lt => self.compare_op(|a, b| match (a, b) {
                (IrValue::Int(x), IrValue::Int(y)) => x < y,
                (IrValue::Float(x), IrValue::Float(y)) => x < y,
                _ => false,
            }),

            OpCode::Le => self.compare_op(|a, b| match (a, b) {
                (IrValue::Int(x), IrValue::Int(y)) => x <= y,
                (IrValue::Float(x), IrValue::Float(y)) => x <= y,
                _ => false,
            }),

            OpCode::Gt => self.compare_op(|a, b| match (a, b) {
                (IrValue::Int(x), IrValue::Int(y)) => x > y,
                (IrValue::Float(x), IrValue::Float(y)) => x > y,
                _ => false,
            }),

            OpCode::Ge => self.compare_op(|a, b| match (a, b) {
                (IrValue::Int(x), IrValue::Int(y)) => x >= y,
                (IrValue::Float(x), IrValue::Float(y)) => x >= y,
                _ => false,
            }),

            OpCode::And => {
                let b = match self.stack.pop() {
                    Some(IrValue::Bool(v)) => v,
                    Some(v) => {
                        return Err(VmResult::Error(format!(
                            "And 操作数类型错误: {:?}",
                            v
                        )));
                    }
                    None => return Err(VmResult::Error("栈下溢: And".to_string())),
                };
                let a = match self.stack.pop() {
                    Some(IrValue::Bool(v)) => v,
                    Some(v) => {
                        return Err(VmResult::Error(format!(
                            "And 操作数类型错误: {:?}",
                            v
                        )));
                    }
                    None => return Err(VmResult::Error("栈下溢: And".to_string())),
                };
                self.stack.push(IrValue::Bool(a && b));
                Ok(ControlFlow::Continue)
            }

            OpCode::Or => {
                let b = match self.stack.pop() {
                    Some(IrValue::Bool(v)) => v,
                    Some(v) => {
                        return Err(VmResult::Error(format!(
                            "Or 操作数类型错误: {:?}",
                            v
                        )));
                    }
                    None => return Err(VmResult::Error("栈下溢: Or".to_string())),
                };
                let a = match self.stack.pop() {
                    Some(IrValue::Bool(v)) => v,
                    Some(v) => {
                        return Err(VmResult::Error(format!(
                            "Or 操作数类型错误: {:?}",
                            v
                        )));
                    }
                    None => return Err(VmResult::Error("栈下溢: Or".to_string())),
                };
                self.stack.push(IrValue::Bool(a || b));
                Ok(ControlFlow::Continue)
            }

            OpCode::Not => {
                let value = match self.stack.pop() {
                    Some(IrValue::Bool(v)) => v,
                    Some(v) => {
                        return Err(VmResult::Error(format!(
                            "Not 操作数类型错误: {:?}",
                            v
                        )));
                    }
                    None => return Err(VmResult::Error("栈下溢: Not".to_string())),
                };
                self.stack.push(IrValue::Bool(!value));
                Ok(ControlFlow::Continue)
            }

            OpCode::Jump(addr) => Ok(ControlFlow::Jump(addr)),

            OpCode::JumpIfFalse(addr) => {
                let value = match self.stack.pop() {
                    Some(v) => v,
                    None => return Err(VmResult::Error("栈下溢: JumpIfFalse".to_string())),
                };
                match value {
                    IrValue::Bool(false) => Ok(ControlFlow::Jump(addr)),
                    IrValue::Bool(true) => Ok(ControlFlow::Continue),
                    _ => Err(VmResult::Error(format!(
                        "JumpIfFalse 操作数类型错误: {:?}",
                        value
                    ))),
                }
            }

            OpCode::JumpIfTrue(addr) => {
                let value = match self.stack.pop() {
                    Some(v) => v,
                    None => return Err(VmResult::Error("栈下溢: JumpIfTrue".to_string())),
                };
                match value {
                    IrValue::Bool(true) => Ok(ControlFlow::Jump(addr)),
                    IrValue::Bool(false) => Ok(ControlFlow::Continue),
                    _ => Err(VmResult::Error(format!(
                        "JumpIfTrue 操作数类型错误: {:?}",
                        value
                    ))),
                }
            }

            OpCode::Call(arg_count) => {
                let func_name_value = match self.stack.pop() {
                    Some(v) => v,
                    None => return Err(VmResult::Error("栈下溢: Call".to_string())),
                };

                let func_name = match func_name_value {
                    IrValue::String(s) => s,
                    _ => {
                        return Err(VmResult::Error(format!(
                            "Call 函数名必须是字符串: {:?}",
                            func_name_value
                        )));
                    }
                };

                let mut args = Vec::with_capacity(arg_count);
                for _ in 0..arg_count {
                    match self.stack.pop() {
                        Some(v) => args.push(v),
                        None => return Err(VmResult::Error("栈下溢: Call 参数不足".to_string())),
                    }
                }
                args.reverse();

                for arg in args {
                    self.stack.push(arg);
                }

                match self.execute(module, &func_name, host) {
                    VmResult::Ok => Ok(ControlFlow::Continue),
                    VmResult::Return(_) => Ok(ControlFlow::Continue),
                    VmResult::HostCall { name, args } => {
                        Err(VmResult::HostCall { name, args })
                    }
                    VmResult::Error(msg) => Err(VmResult::Error(msg)),
                }
            }

            OpCode::Return => {
                let return_value = self.stack.pop();
                Ok(ControlFlow::Return(return_value))
            }

            OpCode::SpawnEntity => {
                let entity_id = host.spawn_entity();
                self.stack.push(IrValue::Entity(entity_id));
                Ok(ControlFlow::Continue)
            }

            OpCode::DespawnEntity => {
                let entity_id = match self.stack.pop() {
                    Some(IrValue::Entity(id)) => id,
                    Some(v) => {
                        return Err(VmResult::Error(format!(
                            "DespawnEntity 操作数类型错误: {:?}",
                            v
                        )));
                    }
                    None => return Err(VmResult::Error("栈下溢: DespawnEntity".to_string())),
                };
                host.despawn_entity(entity_id);
                Ok(ControlFlow::Continue)
            }

            OpCode::AddComponent(type_name) => {
                let value = match self.stack.pop() {
                    Some(v) => v,
                    None => return Err(VmResult::Error("栈下溢: AddComponent 值".to_string())),
                };
                let entity_id = match self.stack.pop() {
                    Some(IrValue::Entity(id)) => id,
                    Some(v) => {
                        return Err(VmResult::Error(format!(
                            "AddComponent 实体 ID 类型错误: {:?}",
                            v
                        )));
                    }
                    None => {
                        return Err(VmResult::Error(
                            "栈下溢: AddComponent 实体 ID".to_string(),
                        ));
                    }
                };
                host.add_component(entity_id, &type_name, value);
                Ok(ControlFlow::Continue)
            }

            OpCode::GetComponent(type_name) => {
                let entity_id = match self.stack.pop() {
                    Some(IrValue::Entity(id)) => id,
                    Some(v) => {
                        return Err(VmResult::Error(format!(
                            "GetComponent 实体 ID 类型错误: {:?}",
                            v
                        )));
                    }
                    None => {
                        return Err(VmResult::Error(
                            "栈下溢: GetComponent 实体 ID".to_string(),
                        ));
                    }
                };
                let value = host
                    .get_component_field(entity_id, &type_name, "")
                    .unwrap_or(IrValue::Null);
                self.stack.push(value);
                Ok(ControlFlow::Continue)
            }

            OpCode::SetComponent(type_name) => {
                let value = match self.stack.pop() {
                    Some(v) => v,
                    None => return Err(VmResult::Error("栈下溢: SetComponent 值".to_string())),
                };
                let entity_id = match self.stack.pop() {
                    Some(IrValue::Entity(id)) => id,
                    Some(v) => {
                        return Err(VmResult::Error(format!(
                            "SetComponent 实体 ID 类型错误: {:?}",
                            v
                        )));
                    }
                    None => {
                        return Err(VmResult::Error(
                            "栈下溢: SetComponent 实体 ID".to_string(),
                        ));
                    }
                };
                host.set_component_field(entity_id, &type_name, "", value);
                Ok(ControlFlow::Continue)
            }

            OpCode::HostCall(name, arg_count) => {
                let mut args = Vec::with_capacity(arg_count);
                for _ in 0..arg_count {
                    match self.stack.pop() {
                        Some(v) => args.push(v),
                        None => {
                            return Err(VmResult::Error(
                                "栈下溢: HostCall 参数不足".to_string(),
                            ));
                        }
                    }
                }
                args.reverse();

                let result = host.call_host_function(&name, args);
                match result {
                    Some(v) => self.stack.push(v),
                    None => self.stack.push(IrValue::Null),
                }
                Ok(ControlFlow::Continue)
            }

            OpCode::Pop => {
                self.stack.pop();
                Ok(ControlFlow::Continue)
            }

            OpCode::Dup => {
                let value = match self.stack.last() {
                    Some(v) => v.clone(),
                    None => return Err(VmResult::Error("栈下溢: Dup".to_string())),
                };
                self.stack.push(value);
                Ok(ControlFlow::Continue)
            }
        }
    }

    /// 执行二元算术操作
    fn binary_op<F>(&mut self, op: F) -> Result<ControlFlow, VmResult>
    where
        F: FnOnce(IrValue, IrValue) -> IrValue,
    {
        let b = match self.stack.pop() {
            Some(v) => v,
            None => return Err(VmResult::Error("栈下溢: 二元操作右操作数".to_string())),
        };
        let a = match self.stack.pop() {
            Some(v) => v,
            None => return Err(VmResult::Error("栈下溢: 二元操作左操作数".to_string())),
        };
        let result = op(a, b);
        self.stack.push(result);
        Ok(ControlFlow::Continue)
    }

    /// 执行比较操作
    fn compare_op<F>(&mut self, op: F) -> Result<ControlFlow, VmResult>
    where
        F: FnOnce(&IrValue, &IrValue) -> bool,
    {
        let b = match self.stack.pop() {
            Some(v) => v,
            None => return Err(VmResult::Error("栈下溢: 比较操作右操作数".to_string())),
        };
        let a = match self.stack.pop() {
            Some(v) => v,
            None => return Err(VmResult::Error("栈下溢: 比较操作左操作数".to_string())),
        };
        let result = op(&a, &b);
        self.stack.push(IrValue::Bool(result));
        Ok(ControlFlow::Continue)
    }
}

impl Default for Vm {
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
    Return(Option<IrValue>),
}
