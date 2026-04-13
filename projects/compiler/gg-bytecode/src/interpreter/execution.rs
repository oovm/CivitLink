use crate::{debug_info::SourceLocation, format::{BytecodeModule, BytecodeInstruction, BytecodeValue}, host::Host};

use super::{BytecodeInterpreter, ControlFlow, InterpretResult};

impl BytecodeInterpreter {
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

            self.call_stack.push(super::InterpreterFrame {
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
}