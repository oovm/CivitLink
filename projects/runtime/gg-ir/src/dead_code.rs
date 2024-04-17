//! 死代码消除优化 Pass 模块
//! 移除不可达的代码块和无用的指令

use std::collections::HashSet;

use crate::{IrModule, OpCode, pass::IrPass};
use gg_core::GResult;

/// 死代码消除优化 Pass
/// 移除不可达的代码块和无用的指令
pub struct DeadCodeElimPass;

impl IrPass for DeadCodeElimPass {
    fn name(&self) -> &str {
        "dead_code_elim"
    }

    fn run(&self, module: &mut IrModule) -> GResult<bool> {
        let mut any_changed = false;
        for func in &mut module.functions {
            if elim_function(func) {
                any_changed = true;
            }
        }
        Ok(any_changed)
    }
}

/// 对单个函数执行死代码消除，返回是否做了修改
fn elim_function(func: &mut crate::IrFunction) -> bool {
    let original_len = func.instructions.len();

    // 阶段 1: 冗余指令消除
    elim_redundant(&mut func.instructions);

    // 阶段 2: 不可达代码消除
    let reachable = compute_reachable(&func.instructions);

    // 在删除前构建旧索引到新索引的映射
    let index_map = build_index_map(&func.instructions, &reachable);

    // 移除不可达指令
    remove_unreachable(&mut func.instructions, &reachable);

    // 使用映射修复跳转地址
    fixup_jumps(&mut func.instructions, &index_map);

    func.instructions.len() != original_len
}

/// 消除冗余指令模式
fn elim_redundant(instructions: &mut Vec<OpCode>) {
    let mut i = 0;
    while i < instructions.len() {
        if i + 1 < instructions.len() {
            if let (OpCode::LoadLocal(a), OpCode::StoreLocal(b)) = (&instructions[i], &instructions[i + 1]) {
                if a == b {
                    instructions.remove(i);
                    instructions.remove(i);
                    continue;
                }
            }
        }
        i += 1;
    }
}

/// 计算可达指令集合
fn compute_reachable(instructions: &[OpCode]) -> HashSet<usize> {
    let mut reachable = HashSet::new();
    let mut worklist = vec![0usize];

    while let Some(ip) = worklist.pop() {
        if ip >= instructions.len() || reachable.contains(&ip) {
            continue;
        }
        reachable.insert(ip);

        match &instructions[ip] {
            OpCode::Jump(target) => {
                worklist.push(*target);
            }
            OpCode::JumpIfFalse(target) | OpCode::JumpIfTrue(target) => {
                worklist.push(ip + 1);
                worklist.push(*target);
            }
            OpCode::Return => {}
            OpCode::Call(_) => {
                worklist.push(ip + 1);
            }
            _ => {
                worklist.push(ip + 1);
            }
        }
    }

    reachable
}

/// 构建旧索引到新索引的映射表
/// 不可达指令映射到 usize::MAX
fn build_index_map(instructions: &[OpCode], reachable: &HashSet<usize>) -> Vec<usize> {
    let mut map = vec![0usize; instructions.len()];
    let mut new_idx = 0usize;
    for old_idx in 0..instructions.len() {
        if reachable.contains(&old_idx) {
            map[old_idx] = new_idx;
            new_idx += 1;
        }
        else {
            map[old_idx] = usize::MAX;
        }
    }
    map
}

/// 移除不可达指令
fn remove_unreachable(instructions: &mut Vec<OpCode>, reachable: &HashSet<usize>) {
    let mut i = 0;
    instructions.retain(|_| {
        let keep = reachable.contains(&i);
        i += 1;
        keep
    });
}

/// 使用索引映射修复跳转地址
fn fixup_jumps(instructions: &mut [OpCode], index_map: &[usize]) {
    for instr in instructions.iter_mut() {
        match instr {
            OpCode::Jump(target) => {
                if *target < index_map.len() && index_map[*target] != usize::MAX {
                    *target = index_map[*target];
                }
            }
            OpCode::JumpIfFalse(target) | OpCode::JumpIfTrue(target) => {
                if *target < index_map.len() && index_map[*target] != usize::MAX {
                    *target = index_map[*target];
                }
            }
            _ => {}
        }
    }
}
