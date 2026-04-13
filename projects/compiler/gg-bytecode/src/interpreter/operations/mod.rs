use crate::{
    format::{BytecodeInstruction, BytecodeModule, BytecodeValue},
    host::Host,
};

use super::{BytecodeInterpreter, ControlFlow, InterpretResult};

impl BytecodeInterpreter {
    /// 分发操作码执行
    pub(crate) fn dispatch_op<H: Host>(
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
                self.call_stack.push(super::InterpreterFrame {
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
    pub(crate) fn binary_op<F>(&mut self, op: F) -> Result<ControlFlow, InterpretResult>
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
    pub(crate) fn compare_op<F>(&mut self, op: F) -> Result<ControlFlow, InterpretResult>
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
