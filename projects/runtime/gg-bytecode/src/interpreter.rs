use crate::format::{
    BytecodeInstruction, BytecodeModule, BytecodeValue,
};
use crate::host::Host;

/// 解释器执行结果
#[derive(Debug, Clone)]
pub enum InterpretResult {
    /// 正常完成
    Ok,
    /// 返回值
    Return(BytecodeValue),
    /// 运行时错误
    Error(String),
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
        Self {
            stack: Vec::new(),
            call_stack: Vec::new(),
            running: false,
        }
    }

    /// 执行模块中的指定函数
    pub fn execute<H: Host>(
        &mut self,
        module: &BytecodeModule,
        function_name: &str,
        host: &mut H,
    ) -> InterpretResult {
        let function = match module.find_function(function_name) {
            Some(f) => f.clone(),
            None => {
                return InterpretResult::Error(format!("函数未找到: {}", function_name));
            }
        };

        let mut locals = vec![BytecodeValue::Null; function.local_count as usize];
        for i in 0..function.param_count.min(function.local_count) as usize {
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

        self.running = true;
        let result = self.execute_instructions(module, host);
        self.running = false;
        result
    }

    /// 执行当前栈帧的指令序列
    fn execute_instructions<H: Host>(
        &mut self,
        module: &BytecodeModule,
        host: &mut H,
    ) -> InterpretResult {
        while self.running {
            let (function_name, ip) = match self.call_stack.last() {
                Some(f) => (f.function_name.clone(), f.ip),
                None => return InterpretResult::Ok,
            };

            let function = match module.find_function(&function_name) {
                Some(f) => f,
                None => {
                    return InterpretResult::Error(format!("函数未找到: {}", function_name));
                }
            };

            let instructions = &function.instructions;
            if ip >= instructions.len() {
                self.call_stack.pop();
                if self.call_stack.is_empty() {
                    return InterpretResult::Ok;
                }
                continue;
            }

            let op = instructions[ip].clone();

            if let Some(f) = self.call_stack.last_mut() {
                f.ip += 1;
            } else {
                return InterpretResult::Error("调用栈为空".to_string());
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
                    if let Some(value) = value {
                        self.stack.push(value);
                    }
                    if self.call_stack.is_empty() {
                        return InterpretResult::Ok;
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
                        return Err(InterpretResult::Error(format!(
                            "常量索引越界: {}",
                            index
                        )));
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
                    None => return Err(InterpretResult::Error("调用栈为空".to_string())),
                };
                let value = match frame.locals.get(index as usize) {
                    Some(v) => v.clone(),
                    None => {
                        return Err(InterpretResult::Error(format!(
                            "局部变量索引越界: {}",
                            index
                        )));
                    }
                };
                self.stack.push(value);
                Ok(ControlFlow::Continue)
            }

            BytecodeInstruction::StoreLocal { index } => {
                let value = match self.stack.pop() {
                    Some(v) => v,
                    None => {
                        return Err(InterpretResult::Error(
                            "栈下溢: StoreLocal".to_string(),
                        ));
                    }
                };
                let frame = match self.call_stack.last_mut() {
                    Some(f) => f,
                    None => return Err(InterpretResult::Error("调用栈为空".to_string())),
                };
                if index as usize >= frame.locals.len() {
                    return Err(InterpretResult::Error(format!(
                        "局部变量索引越界: {}",
                        index
                    )));
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
                        return Err(InterpretResult::Error(
                            "栈下溢: Neg".to_string(),
                        ));
                    }
                };
                let result = match value {
                    BytecodeValue::Int(x) => BytecodeValue::Int(-x),
                    BytecodeValue::Float(x) => BytecodeValue::Float(-x),
                    _ => {
                        return Err(InterpretResult::Error(format!(
                            "Neg 操作数类型错误: {:?}",
                            value
                        )));
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
                        return Err(InterpretResult::Error(format!(
                            "And 操作数类型错误: {:?}",
                            v
                        )));
                    }
                    None => {
                        return Err(InterpretResult::Error(
                            "栈下溢: And".to_string(),
                        ));
                    }
                };
                let a = match self.stack.pop() {
                    Some(BytecodeValue::Bool(v)) => v,
                    Some(v) => {
                        return Err(InterpretResult::Error(format!(
                            "And 操作数类型错误: {:?}",
                            v
                        )));
                    }
                    None => {
                        return Err(InterpretResult::Error(
                            "栈下溢: And".to_string(),
                        ));
                    }
                };
                self.stack.push(BytecodeValue::Bool(a && b));
                Ok(ControlFlow::Continue)
            }

            BytecodeInstruction::Or => {
                let b = match self.stack.pop() {
                    Some(BytecodeValue::Bool(v)) => v,
                    Some(v) => {
                        return Err(InterpretResult::Error(format!(
                            "Or 操作数类型错误: {:?}",
                            v
                        )));
                    }
                    None => {
                        return Err(InterpretResult::Error(
                            "栈下溢: Or".to_string(),
                        ));
                    }
                };
                let a = match self.stack.pop() {
                    Some(BytecodeValue::Bool(v)) => v,
                    Some(v) => {
                        return Err(InterpretResult::Error(format!(
                            "Or 操作数类型错误: {:?}",
                            v
                        )));
                    }
                    None => {
                        return Err(InterpretResult::Error(
                            "栈下溢: Or".to_string(),
                        ));
                    }
                };
                self.stack.push(BytecodeValue::Bool(a || b));
                Ok(ControlFlow::Continue)
            }

            BytecodeInstruction::Not => {
                let value = match self.stack.pop() {
                    Some(BytecodeValue::Bool(v)) => v,
                    Some(v) => {
                        return Err(InterpretResult::Error(format!(
                            "Not 操作数类型错误: {:?}",
                            v
                        )));
                    }
                    None => {
                        return Err(InterpretResult::Error(
                            "栈下溢: Not".to_string(),
                        ));
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
                        return Err(InterpretResult::Error(
                            "栈下溢: JumpIfFalse".to_string(),
                        ));
                    }
                };
                match value {
                    BytecodeValue::Bool(false) => Ok(ControlFlow::Jump(address as usize)),
                    BytecodeValue::Bool(true) => Ok(ControlFlow::Continue),
                    _ => Err(InterpretResult::Error(format!(
                        "JumpIfFalse 操作数类型错误: {:?}",
                        value
                    ))),
                }
            }

            BytecodeInstruction::JumpIfTrue { address } => {
                let value = match self.stack.pop() {
                    Some(v) => v,
                    None => {
                        return Err(InterpretResult::Error(
                            "栈下溢: JumpIfTrue".to_string(),
                        ));
                    }
                };
                match value {
                    BytecodeValue::Bool(true) => Ok(ControlFlow::Jump(address as usize)),
                    BytecodeValue::Bool(false) => Ok(ControlFlow::Continue),
                    _ => Err(InterpretResult::Error(format!(
                        "JumpIfTrue 操作数类型错误: {:?}",
                        value
                    ))),
                }
            }

            BytecodeInstruction::Call { arg_count } => {
                let func_name_value = match self.stack.pop() {
                    Some(v) => v,
                    None => {
                        return Err(InterpretResult::Error(
                            "栈下溢: Call".to_string(),
                        ));
                    }
                };

                let func_name = match func_name_value {
                    BytecodeValue::String(s) => s,
                    _ => {
                        return Err(InterpretResult::Error(format!(
                            "Call 函数名必须是字符串: {:?}",
                            func_name_value
                        )));
                    }
                };

                let mut args = Vec::with_capacity(arg_count as usize);
                for _ in 0..arg_count {
                    match self.stack.pop() {
                        Some(v) => args.push(v),
                        None => {
                            return Err(InterpretResult::Error(
                                "栈下溢: Call 参数不足".to_string(),
                            ));
                        }
                    }
                }
                args.reverse();

                for arg in args {
                    self.stack.push(arg);
                }

                match self.execute(module, &func_name, host) {
                    InterpretResult::Ok => Ok(ControlFlow::Continue),
                    InterpretResult::Return(_) => Ok(ControlFlow::Continue),
                    InterpretResult::Error(msg) => Err(InterpretResult::Error(msg)),
                }
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
                        return Err(InterpretResult::Error(format!(
                            "DespawnEntity 操作数类型错误: {:?}",
                            v
                        )));
                    }
                    None => {
                        return Err(InterpretResult::Error(
                            "栈下溢: DespawnEntity".to_string(),
                        ));
                    }
                };
                host.despawn_entity(entity_id);
                Ok(ControlFlow::Continue)
            }

            BytecodeInstruction::AddComponent { type_name_index } => {
                let type_name = match module.string_pool.get(type_name_index as usize) {
                    Some(s) => s.clone(),
                    None => {
                        return Err(InterpretResult::Error(format!(
                            "字符串池索引越界: {}",
                            type_name_index
                        )));
                    }
                };
                let value = match self.stack.pop() {
                    Some(v) => v,
                    None => {
                        return Err(InterpretResult::Error(
                            "栈下溢: AddComponent 值".to_string(),
                        ));
                    }
                };
                let entity_id = match self.stack.pop() {
                    Some(BytecodeValue::Entity(id)) => id,
                    Some(v) => {
                        return Err(InterpretResult::Error(format!(
                            "AddComponent 实体 ID 类型错误: {:?}",
                            v
                        )));
                    }
                    None => {
                        return Err(InterpretResult::Error(
                            "栈下溢: AddComponent 实体 ID".to_string(),
                        ));
                    }
                };
                host.add_component(entity_id, &type_name, value);
                Ok(ControlFlow::Continue)
            }

            BytecodeInstruction::GetComponent { type_name_index } => {
                let type_name = match module.string_pool.get(type_name_index as usize) {
                    Some(s) => s.clone(),
                    None => {
                        return Err(InterpretResult::Error(format!(
                            "字符串池索引越界: {}",
                            type_name_index
                        )));
                    }
                };
                let entity_id = match self.stack.pop() {
                    Some(BytecodeValue::Entity(id)) => id,
                    Some(v) => {
                        return Err(InterpretResult::Error(format!(
                            "GetComponent 实体 ID 类型错误: {:?}",
                            v
                        )));
                    }
                    None => {
                        return Err(InterpretResult::Error(
                            "栈下溢: GetComponent 实体 ID".to_string(),
                        ));
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
                        return Err(InterpretResult::Error(format!(
                            "字符串池索引越界: {}",
                            type_name_index
                        )));
                    }
                };
                let value = match self.stack.pop() {
                    Some(v) => v,
                    None => {
                        return Err(InterpretResult::Error(
                            "栈下溢: SetComponent 值".to_string(),
                        ));
                    }
                };
                let entity_id = match self.stack.pop() {
                    Some(BytecodeValue::Entity(id)) => id,
                    Some(v) => {
                        return Err(InterpretResult::Error(format!(
                            "SetComponent 实体 ID 类型错误: {:?}",
                            v
                        )));
                    }
                    None => {
                        return Err(InterpretResult::Error(
                            "栈下溢: SetComponent 实体 ID".to_string(),
                        ));
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
                        return Err(InterpretResult::Error(format!(
                            "字符串池索引越界: {}",
                            name_index
                        )));
                    }
                };
                let mut args = Vec::with_capacity(arg_count as usize);
                for _ in 0..arg_count {
                    match self.stack.pop() {
                        Some(v) => args.push(v),
                        None => {
                            return Err(InterpretResult::Error(
                                "栈下溢: HostCall 参数不足".to_string(),
                            ));
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
                        return Err(InterpretResult::Error(
                            "栈下溢: Dup".to_string(),
                        ));
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
                return Err(InterpretResult::Error(
                    "栈下溢: 二元操作右操作数".to_string(),
                ));
            }
        };
        let a = match self.stack.pop() {
            Some(v) => v,
            None => {
                return Err(InterpretResult::Error(
                    "栈下溢: 二元操作左操作数".to_string(),
                ));
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
                return Err(InterpretResult::Error(
                    "栈下溢: 比较操作右操作数".to_string(),
                ));
            }
        };
        let a = match self.stack.pop() {
            Some(v) => v,
            None => {
                return Err(InterpretResult::Error(
                    "栈下溢: 比较操作左操作数".to_string(),
                ));
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
    use crate::format::{BytecodeFunction, BytecodeInstruction, BytecodeModule, BytecodeValue};
    use crate::host::Host;

    /// 测试用宿主实现
    struct TestHost {
        /// 已创建的实体列表
        entities: Vec<u64>,
        /// 操作日志
        log: Vec<String>,
    }

    impl TestHost {
        /// 创建新的测试宿主
        fn new() -> Self {
            Self {
                entities: Vec::new(),
                log: Vec::new(),
            }
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
            self.log.push(format!(
                "add_component({}, {}, {:?})",
                entity_id, component_type, value
            ));
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
            entity_id: u64,
            component_type: &str,
            field: &str,
            value: BytecodeValue,
        ) {
            self.log.push(format!(
                "set_component_field({}, {}, {}, {:?})",
                entity_id, component_type, field, value
            ));
        }

        fn call_host_function(
            &mut self,
            name: &str,
            args: Vec<BytecodeValue>,
        ) -> Option<BytecodeValue> {
            self.log.push(format!("call_host_function({}, {:?})", name, args));
            match name {
                "print" => None,
                "spawn_entity" => {
                    let id = self.entities.len() as u64;
                    self.entities.push(id);
                    Some(BytecodeValue::Entity(id))
                }
                _ => None,
            }
        }
    }

    /// 测试基本算术：返回 42 的简单函数
    #[test]
    fn test_basic_arithmetic_return_42() {
        let module = BytecodeModule {
            name: "test".to_string(),
            version: 1,
            constants: vec![BytecodeValue::Int(42)],
            string_pool: vec![],
            functions: vec![BytecodeFunction {
                name: "main".to_string(),
                param_count: 0,
                local_count: 0,
                instructions: vec![
                    BytecodeInstruction::LoadConst { index: 0 },
                    BytecodeInstruction::Return,
                ],
            }],
        };

        let mut interpreter = BytecodeInterpreter::new();
        let mut host = TestHost::new();
        let result = interpreter.execute(&module, "main", &mut host);

        match result {
            InterpretResult::Return(BytecodeValue::Int(42)) => {}
            other => panic!("期望返回 Int(42)，实际: {:?}", other),
        }
    }

    /// 测试函数调用：一个函数调用另一个函数
    #[test]
    fn test_function_call() {
        let module = BytecodeModule {
            name: "test".to_string(),
            version: 1,
            constants: vec![
                BytecodeValue::Int(10),
                BytecodeValue::String("helper".to_string()),
            ],
            string_pool: vec![],
            functions: vec![
                BytecodeFunction {
                    name: "helper".to_string(),
                    param_count: 0,
                    local_count: 0,
                    instructions: vec![
                        BytecodeInstruction::LoadConst { index: 0 },
                        BytecodeInstruction::Return,
                    ],
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
                },
            ],
        };

        let mut interpreter = BytecodeInterpreter::new();
        let mut host = TestHost::new();
        let result = interpreter.execute(&module, "main", &mut host);

        match result {
            InterpretResult::Return(BytecodeValue::Int(10)) => {}
            other => panic!("期望返回 Int(10)，实际: {:?}", other),
        }
    }

    /// 测试条件执行：if/else 分支
    #[test]
    fn test_conditional_execution() {
        let module = BytecodeModule {
            name: "test".to_string(),
            version: 1,
            constants: vec![BytecodeValue::Int(1), BytecodeValue::Int(2)],
            string_pool: vec![],
            functions: vec![BytecodeFunction {
                name: "main".to_string(),
                param_count: 0,
                local_count: 0,
                instructions: vec![
                    BytecodeInstruction::LoadTrue,
                    BytecodeInstruction::JumpIfFalse { address: 5 },
                    BytecodeInstruction::LoadConst { index: 0 },
                    BytecodeInstruction::Jump { address: 6 },
                    BytecodeInstruction::LoadConst { index: 1 },
                    BytecodeInstruction::Return,
                ],
            }],
        };

        let mut interpreter = BytecodeInterpreter::new();
        let mut host = TestHost::new();
        let result = interpreter.execute(&module, "main", &mut host);

        match result {
            InterpretResult::Return(BytecodeValue::Int(1)) => {}
            other => panic!("期望返回 Int(1)（true 分支），实际: {:?}", other),
        }
    }

    /// 测试宿主交互：spawn_entity、add_component、print
    #[test]
    fn test_host_interaction() {
        let module = BytecodeModule {
            name: "test".to_string(),
            version: 1,
            constants: vec![BytecodeValue::String("Position".to_string())],
            string_pool: vec!["Position".to_string(), "print".to_string()],
            functions: vec![BytecodeFunction {
                name: "main".to_string(),
                param_count: 0,
                local_count: 1,
                instructions: vec![
                    BytecodeInstruction::SpawnEntity,
                    BytecodeInstruction::StoreLocal { index: 0 },
                    BytecodeInstruction::LoadLocal { index: 0 },
                    BytecodeInstruction::LoadConst { index: 0 },
                    BytecodeInstruction::AddComponent { type_name_index: 0 },
                    BytecodeInstruction::HostCall {
                        name_index: 1,
                        arg_count: 0,
                    },
                    BytecodeInstruction::Pop,
                    BytecodeInstruction::Return,
                ],
            }],
        };

        let mut interpreter = BytecodeInterpreter::new();
        let mut host = TestHost::new();
        let result = interpreter.execute(&module, "main", &mut host);

        assert!(matches!(result, InterpretResult::Return(_)));
        assert_eq!(host.entities.len(), 1);
        assert!(host.log.iter().any(|l| l.starts_with("add_component")));
    }
}
