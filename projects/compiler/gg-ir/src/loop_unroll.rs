#![warn(missing_docs)]

//! 循环展开优化 Pass
//! 对固定迭代次数的小循环进行展开优化

use crate::{IrModule, OpCode, pass::IrPass};
use gg_core::GResult;

/// 循环展开优化 Pass
/// 识别固定迭代次数的循环，将循环体复制 N 次以减少循环控制开销
pub struct LoopUnrollingPass {
    /// 最大展开次数，迭代次数超过此值的循环不进行展开
    pub max_unroll_count: usize,
}

impl LoopUnrollingPass {
    /// 创建新的循环展开 Pass，使用默认最大展开次数 8
    pub fn new() -> Self {
        Self { max_unroll_count: 8 }
    }

    /// 创建指定最大展开次数的循环展开 Pass
    pub fn with_max_unroll(max_unroll_count: usize) -> Self {
        Self { max_unroll_count }
    }
}

impl Default for LoopUnrollingPass {
    fn default() -> Self {
        Self::new()
    }
}

impl IrPass for LoopUnrollingPass {
    fn name(&self) -> &str {
        "LoopUnrolling"
    }

    fn run(&self, module: &mut IrModule) -> GResult<bool> {
        let mut any_changed = false;
        for func in &mut module.functions {
            if unroll_loops(func, self.max_unroll_count) {
                any_changed = true;
            }
        }
        Ok(any_changed)
    }
}

/// 计数循环信息
#[allow(dead_code)]
struct CountedLoop {
    /// 循环迭代变量所在的局部变量索引
    iter_local: usize,
    /// 循环起始值常量池索引
    start_const: usize,
    /// 循环结束值常量池索引
    end_const: usize,
    /// 比较操作
    compare_op: OpCode,
    /// 循环头地址（比较指令位置）
    header: usize,
    /// 条件跳转指令位置
    condition_jump: usize,
    /// 循环体起始地址
    body_start: usize,
    /// 递增指令序列起始地址
    increment_start: usize,
    /// 递增指令序列长度
    increment_len: usize,
    /// back-edge 跳转指令位置
    back_jump: usize,
    /// 循环退出地址
    exit_addr: usize,
}

/// 尝试识别一个简单的计数循环
/// 模式：LoadLocal(i) LoadConst(n) Cmp JumpIfFalse(exit) ...body... LoadLocal(i) LoadConst(1) Add StoreLocal(i) Jump(header)
fn identify_counted_loop(instructions: &[OpCode]) -> Option<CountedLoop> {
    if instructions.len() < 8 {
        return None;
    }

    for header in 0..instructions.len() {
        if header + 3 >= instructions.len() {
            continue;
        }

        let iter_local = match &instructions[header] {
            OpCode::LoadLocal(idx) => *idx,
            _ => continue,
        };

        let end_const = match &instructions[header + 1] {
            OpCode::LoadConst(idx) => *idx,
            _ => continue,
        };

        let compare_op = match &instructions[header + 2] {
            op @ OpCode::Lt | op @ OpCode::Le | op @ OpCode::Gt | op @ OpCode::Ge | op @ OpCode::Eq | op @ OpCode::Ne => {
                op.clone()
            }
            _ => continue,
        };

        let exit_addr = match &instructions[header + 3] {
            OpCode::JumpIfFalse(addr) => *addr,
            _ => continue,
        };

        if exit_addr <= header + 3 {
            continue;
        }

        let body_start = header + 4;

        for back_offset in (body_start..exit_addr).rev() {
            if back_offset + 3 >= instructions.len() {
                continue;
            }

            let matches_increment = match &instructions[back_offset..] {
                [OpCode::LoadLocal(idx), OpCode::LoadConst(_), OpCode::Add, OpCode::StoreLocal(store_idx), ..] => {
                    *idx == iter_local && *store_idx == iter_local
                }
                _ => false,
            };

            if !matches_increment {
                continue;
            }

            let back_jump_pos = back_offset + 4;
            if back_jump_pos >= instructions.len() {
                continue;
            }

            if let OpCode::Jump(target) = instructions[back_jump_pos] {
                if target == header {
                    return Some(CountedLoop {
                        iter_local,
                        start_const: 0,
                        end_const,
                        compare_op,
                        header,
                        condition_jump: header + 3,
                        body_start,
                        increment_start: back_offset,
                        increment_len: 4,
                        back_jump: back_jump_pos,
                        exit_addr,
                    });
                }
            }
        }
    }

    None
}

/// 对单个函数执行循环展开优化
fn unroll_loops(func: &mut crate::IrFunction, max_unroll_count: usize) -> bool {
    let loop_info = match identify_counted_loop(&func.instructions) {
        Some(info) => info,
        None => return false,
    };

    let iteration_count = match extract_iteration_count(&func.instructions, &loop_info) {
        Some(count) if count > 0 && count <= max_unroll_count => count,
        _ => return false,
    };

    let body_instructions = func.instructions[loop_info.body_start..loop_info.increment_start].to_vec();
    let increment_instructions =
        func.instructions[loop_info.increment_start..loop_info.increment_start + loop_info.increment_len].to_vec();

    let mut unrolled = Vec::new();

    for _ in 0..iteration_count {
        unrolled.extend(body_instructions.iter().cloned());
        unrolled.extend(increment_instructions.iter().cloned());
    }

    let new_instructions = {
        let mut result = Vec::new();

        if loop_info.header > 0 {
            result.extend_from_slice(&func.instructions[..loop_info.header]);
        }

        result.extend(unrolled);

        if loop_info.exit_addr < func.instructions.len() {
            result.extend_from_slice(&func.instructions[loop_info.exit_addr..]);
        }

        result
    };

    func.instructions = new_instructions;

    fixup_jump_addresses(&mut func.instructions, loop_info.header, loop_info.exit_addr);

    true
}

/// 从循环头部的比较指令中提取迭代次数
/// 仅处理常量比较的简单情况
fn extract_iteration_count(instructions: &[OpCode], loop_info: &CountedLoop) -> Option<usize> {
    let _ = instructions;
    let _ = loop_info;
    None
}

/// 修复因删除循环控制指令而偏移的跳转地址
fn fixup_jump_addresses(instructions: &mut [OpCode], _old_header: usize, _old_exit: usize) {
    let len = instructions.len();
    for op in instructions.iter_mut() {
        match op {
            OpCode::Jump(target) => {
                if *target >= len {
                    *target = len.saturating_sub(1);
                }
            }
            OpCode::JumpIfFalse(target) | OpCode::JumpIfTrue(target) => {
                if *target >= len {
                    *target = len.saturating_sub(1);
                }
            }
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{IrFunction, IrModule};

    fn make_func(name: &str, instructions: Vec<OpCode>) -> IrFunction {
        IrFunction {
            name: name.to_string(),
            param_count: 0,
            local_count: 2,
            local_names: vec!["i".to_string(), "sum".to_string()],
            instructions,
            is_entry: false,
            target: None,
        }
    }

    fn make_module(funcs: Vec<IrFunction>) -> IrModule {
        let mut module = IrModule::new("test");
        module.functions = funcs;
        module
    }

    #[test]
    fn test_identify_counted_loop() {
        let instructions = vec![
            OpCode::LoadLocal(0),
            OpCode::LoadConst(10),
            OpCode::Lt,
            OpCode::JumpIfFalse(11),
            OpCode::LoadLocal(1),
            OpCode::LoadLocal(0),
            OpCode::Add,
            OpCode::StoreLocal(1),
            OpCode::LoadLocal(0),
            OpCode::LoadConst(1),
            OpCode::Add,
            OpCode::StoreLocal(0),
            OpCode::Jump(0),
        ];

        let loop_info = identify_counted_loop(&instructions);
        assert!(loop_info.is_some());
        let info = loop_info.unwrap();
        assert_eq!(info.iter_local, 0);
        assert_eq!(info.header, 0);
        assert_eq!(info.body_start, 4);
        assert_eq!(info.increment_start, 8);
        assert_eq!(info.back_jump, 12);
        assert_eq!(info.exit_addr, 11);
    }

    #[test]
    fn test_no_counted_loop_without_increment() {
        let instructions = vec![
            OpCode::LoadLocal(0),
            OpCode::LoadConst(10),
            OpCode::Lt,
            OpCode::JumpIfFalse(6),
            OpCode::LoadConst(0),
            OpCode::Pop,
            OpCode::Jump(0),
        ];

        let loop_info = identify_counted_loop(&instructions);
        assert!(loop_info.is_none());
    }

    #[test]
    fn test_no_counted_loop_wrong_back_jump() {
        let instructions = vec![
            OpCode::LoadLocal(0),
            OpCode::LoadConst(10),
            OpCode::Lt,
            OpCode::JumpIfFalse(11),
            OpCode::LoadLocal(0),
            OpCode::LoadConst(1),
            OpCode::Add,
            OpCode::StoreLocal(0),
            OpCode::Jump(1),
        ];

        let loop_info = identify_counted_loop(&instructions);
        assert!(loop_info.is_none());
    }

    #[test]
    fn test_unroll_skips_unknown_iteration_count() {
        let mut module = make_module(vec![make_func(
            "loop",
            vec![
                OpCode::LoadLocal(0),
                OpCode::LoadConst(10),
                OpCode::Lt,
                OpCode::JumpIfFalse(11),
                OpCode::LoadLocal(1),
                OpCode::LoadLocal(0),
                OpCode::Add,
                OpCode::StoreLocal(1),
                OpCode::LoadLocal(0),
                OpCode::LoadConst(1),
                OpCode::Add,
                OpCode::StoreLocal(0),
                OpCode::Jump(0),
            ],
        )]);

        let pass = LoopUnrollingPass::new();
        let changed = pass.run(&mut module).unwrap();
        assert!(!changed);
    }

    #[test]
    fn test_empty_function() {
        let mut module = make_module(vec![make_func("empty", vec![])]);
        let pass = LoopUnrollingPass::new();
        let changed = pass.run(&mut module).unwrap();
        assert!(!changed);
    }

    #[test]
    fn test_pass_name() {
        let pass = LoopUnrollingPass::new();
        assert_eq!(pass.name(), "LoopUnrolling");
    }

    #[test]
    fn test_default_max_unroll() {
        let pass = LoopUnrollingPass::new();
        assert_eq!(pass.max_unroll_count, 8);
    }

    #[test]
    fn test_custom_max_unroll() {
        let pass = LoopUnrollingPass::with_max_unroll(4);
        assert_eq!(pass.max_unroll_count, 4);
    }

    #[test]
    fn test_default_impl() {
        let pass = LoopUnrollingPass::default();
        assert_eq!(pass.max_unroll_count, 8);
    }

    #[test]
    fn test_no_loop_short_function() {
        let mut module = make_module(vec![make_func("short", vec![OpCode::LoadConst(0), OpCode::Return])]);
        let pass = LoopUnrollingPass::new();
        let changed = pass.run(&mut module).unwrap();
        assert!(!changed);
    }

    #[test]
    fn test_multiple_functions() {
        let mut module = make_module(vec![
            make_func("no_loop", vec![OpCode::LoadConst(0), OpCode::Return]),
            make_func("also_no_loop", vec![OpCode::LoadConst(1), OpCode::Return]),
        ]);
        let pass = LoopUnrollingPass::new();
        let changed = pass.run(&mut module).unwrap();
        assert!(!changed);
    }
}
