#![warn(missing_docs)]

//! 公共子表达式消除优化 Pass

use crate::{IrModule, OpCode, pass::IrPass};
use gg_core::GResult;

/// 公共子表达式消除优化 Pass
/// 识别重复的相同计算模式，将后续出现替换为首次计算结果的存储和加载
pub struct CommonSubexprElimPass;

impl CommonSubexprElimPass {
    /// 创建新的公共子表达式消除 Pass
    pub fn new() -> Self {
        Self
    }
}

impl Default for CommonSubexprElimPass {
    fn default() -> Self {
        Self::new()
    }
}

impl IrPass for CommonSubexprElimPass {
    fn name(&self) -> &str {
        "common_subexpr_elim"
    }

    fn run(&self, module: &mut IrModule) -> GResult<bool> {
        let mut any_changed = false;
        for func in &mut module.functions {
            if elim_common_subexpr(func) {
                any_changed = true;
            }
        }
        Ok(any_changed)
    }
}

fn elim_common_subexpr(func: &mut crate::IrFunction) -> bool {
    let pattern_len = 3;
    if func.instructions.len() < pattern_len * 2 {
        return false;
    }

    let mut temp_local_start = func.local_count;
    let mut replacements: Vec<(usize, usize, usize)> = Vec::new();

    let mut i = 0;
    while i + pattern_len <= func.instructions.len() {
        let pattern = &func.instructions[i..i + pattern_len];

        if !is_pure_pattern(pattern) {
            i += 1;
            continue;
        }

        let mut j = i + pattern_len;
        while j + pattern_len <= func.instructions.len() {
            let candidate = &func.instructions[j..j + pattern_len];

            if patterns_equal(pattern, candidate) && !has_side_effect_between(&func.instructions, i + pattern_len, j) {
                let temp_idx = temp_local_start;
                temp_local_start += 1;

                replacements.push((i, j, temp_idx));
                break;
            }
            j += 1;
        }

        i += 1;
    }

    if replacements.is_empty() {
        return false;
    }

    for (first_pos, second_pos, temp_idx) in replacements.iter().rev() {
        let insert_pos = first_pos + pattern_len;

        func.instructions.splice(insert_pos..insert_pos, std::iter::once(OpCode::StoreLocal(*temp_idx)));

        let adjusted_second = second_pos + 1;
        func.instructions.splice(adjusted_second..adjusted_second + pattern_len, std::iter::once(OpCode::LoadLocal(*temp_idx)));
    }

    func.local_count = temp_local_start;

    true
}

fn is_pure_pattern(pattern: &[OpCode]) -> bool {
    if pattern.len() != 3 {
        return false;
    }

    match &pattern[0] {
        OpCode::LoadLocal(_) | OpCode::LoadConst(_) => {}
        _ => return false,
    }

    match &pattern[1] {
        OpCode::LoadLocal(_) | OpCode::LoadConst(_) => {}
        _ => return false,
    }

    matches!(
        &pattern[2],
        OpCode::Add
            | OpCode::Sub
            | OpCode::Mul
            | OpCode::Div
            | OpCode::Mod
            | OpCode::Eq
            | OpCode::Ne
            | OpCode::Lt
            | OpCode::Le
            | OpCode::Gt
            | OpCode::Ge
    )
}

fn patterns_equal(a: &[OpCode], b: &[OpCode]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    for (x, y) in a.iter().zip(b.iter()) {
        if !opcodes_equal(x, y) {
            return false;
        }
    }
    true
}

fn opcodes_equal(a: &OpCode, b: &OpCode) -> bool {
    match (a, b) {
        (OpCode::LoadLocal(i), OpCode::LoadLocal(j)) => i == j,
        (OpCode::LoadConst(i), OpCode::LoadConst(j)) => i == j,
        (OpCode::Add, OpCode::Add) => true,
        (OpCode::Sub, OpCode::Sub) => true,
        (OpCode::Mul, OpCode::Mul) => true,
        (OpCode::Div, OpCode::Div) => true,
        (OpCode::Mod, OpCode::Mod) => true,
        (OpCode::Eq, OpCode::Eq) => true,
        (OpCode::Ne, OpCode::Ne) => true,
        (OpCode::Lt, OpCode::Lt) => true,
        (OpCode::Le, OpCode::Le) => true,
        (OpCode::Gt, OpCode::Gt) => true,
        (OpCode::Ge, OpCode::Ge) => true,
        _ => false,
    }
}

fn has_side_effect_between(instructions: &[OpCode], start: usize, end: usize) -> bool {
    for i in start..end {
        match &instructions[i] {
            OpCode::StoreLocal(_) => return true,
            OpCode::Call(_) | OpCode::TailCall(_) => return true,
            OpCode::SpawnEntity | OpCode::DespawnEntity => return true,
            OpCode::AddComponent(_) | OpCode::SetComponent(_) => return true,
            OpCode::HostCall(_, _) => return true,
            OpCode::SetField(_) | OpCode::SetIndex => return true,
            OpCode::Jump(_) | OpCode::JumpIfFalse(_) | OpCode::JumpIfTrue(_) => return true,
            OpCode::Return => return true,
            _ => {}
        }
    }
    false
}
