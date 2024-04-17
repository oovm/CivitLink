#![warn(missing_docs)]

//! 内联展开优化 Pass

use crate::{IrModule, IrValue, OpCode, pass::IrPass};
use gg_core::GResult;

/// 内联展开优化 Pass
/// 将小型函数调用内联展开为调用处的指令序列，消除函数调用开销
pub struct InlineExpansionPass {
    /// 最大指令数阈值，超过此值的函数不内联
    max_instruction_count: usize,
}

impl InlineExpansionPass {
    /// 创建新的内联展开 Pass，使用默认阈值
    pub fn new() -> Self {
        Self { max_instruction_count: 20 }
    }

    /// 创建指定阈值的内联展开 Pass
    pub fn with_max_instructions(count: usize) -> Self {
        Self { max_instruction_count: count }
    }
}

impl Default for InlineExpansionPass {
    fn default() -> Self {
        Self::new()
    }
}

impl IrPass for InlineExpansionPass {
    fn name(&self) -> &str {
        "inline_expansion"
    }

    fn run(&self, module: &mut IrModule) -> GResult<bool> {
        let mut any_changed = false;
        let max_iterations = 10;

        for _ in 0..max_iterations {
            let mut changed = false;
            let func_count = module.functions.len();
            for func_idx in 0..func_count {
                if inline_function(module, func_idx, self.max_instruction_count) {
                    changed = true;
                    any_changed = true;
                }
            }
            if !changed {
                break;
            }
        }

        Ok(any_changed)
    }
}

fn inline_function(module: &mut IrModule, func_idx: usize, max_instructions: usize) -> bool {
    let mut changed = false;
    let mut i = 0;

    while i < module.functions[func_idx].instructions.len() {
        if let OpCode::Call(arg_count) = module.functions[func_idx].instructions[i] {
            let name_ip = i.saturating_sub(arg_count + 1);
            let target_name = get_call_target_name(module, func_idx, name_ip);

            if let Some(target_name) = target_name {
                let target_func_idx = module.functions.iter().position(|f| f.name == target_name);

                if let Some(tidx) = target_func_idx {
                    if tidx == func_idx {
                        i += 1;
                        continue;
                    }

                    let target = &module.functions[tidx];

                    if target.instructions.len() > max_instructions {
                        i += 1;
                        continue;
                    }

                    if contains_call(&target.instructions) {
                        i += 1;
                        continue;
                    }

                    let target_param_count = target.param_count;
                    let target_local_count = target.local_count;
                    let extra_locals = target_local_count.saturating_sub(target_param_count);
                    let current_local_count = module.functions[func_idx].local_count;

                    let mut inlined = Vec::new();

                    for j in name_ip..i {
                        if let Some(op) = module.functions[func_idx].instructions.get(j) {
                            if let OpCode::LoadConst(_) = op {
                                continue;
                            }
                            inlined.push(op.clone());
                        }
                    }

                    for target_op in &target.instructions {
                        match target_op {
                            OpCode::LoadLocal(idx) => {
                                inlined.push(OpCode::LoadLocal(*idx));
                            }
                            OpCode::StoreLocal(idx) => {
                                if *idx < target_param_count {
                                    inlined.push(OpCode::StoreLocal(*idx));
                                }
                                else {
                                    inlined.push(OpCode::StoreLocal(*idx - target_param_count + current_local_count));
                                }
                            }
                            OpCode::Return => {}
                            _ => {
                                inlined.push(target_op.clone());
                            }
                        }
                    }

                    let range = name_ip..i + 1;
                    module.functions[func_idx].instructions.splice(range, inlined);
                    module.functions[func_idx].local_count += extra_locals;

                    changed = true;
                    continue;
                }
            }
        }

        i += 1;
    }

    changed
}

fn get_call_target_name(module: &IrModule, func_idx: usize, name_ip: usize) -> Option<String> {
    let instructions = &module.functions[func_idx].instructions;
    if name_ip >= instructions.len() {
        return None;
    }
    if let OpCode::LoadConst(const_idx) = instructions[name_ip] {
        if let Some(IrValue::String(name)) = module.constants.get(const_idx) {
            return Some(name.clone());
        }
    }
    None
}

fn contains_call(instructions: &[OpCode]) -> bool {
    for op in instructions {
        if let OpCode::Call(_) = op {
            return true;
        }
    }
    false
}
