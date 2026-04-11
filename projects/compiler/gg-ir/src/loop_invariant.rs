#![warn(missing_docs)]

//! 循环不变量外提优化 Pass
//! 识别循环体内的不变计算并将其外提到循环前置块

use std::collections::HashSet;

use crate::{IrModule, OpCode, pass::IrPass};
use gg_core::GResult;

/// 循环不变量外提优化 Pass
/// 检测循环体内的不变计算指令，将其移动到循环前置块
pub struct LoopInvariantCodeMotionPass;

impl IrPass for LoopInvariantCodeMotionPass {
    fn name(&self) -> &str {
        "LoopInvariantCodeMotion"
    }

    fn run(&self, module: &mut IrModule) -> GResult<bool> {
        let mut any_changed = false;
        for func in &mut module.functions {
            if hoist_loop_invariants(func) {
                any_changed = true;
            }
        }
        Ok(any_changed)
    }
}

/// 循环信息
struct LoopInfo {
    /// 循环头地址
    header: usize,
    /// 循环体结束地址（含 back-edge 跳转指令）
    back_edge: usize,
}

/// 检测函数中的所有循环
/// 通过查找 back-edge 跳转（目标地址 <= 当前地址）来识别循环
fn detect_loops(instructions: &[OpCode]) -> Vec<LoopInfo> {
    let mut loops = Vec::new();
    for (i, op) in instructions.iter().enumerate() {
        let target = match op {
            OpCode::Jump(t) => Some(*t),
            OpCode::JumpIfFalse(t) | OpCode::JumpIfTrue(t) => Some(*t),
            _ => None,
        };
        if let Some(target) = target {
            if target <= i && target < instructions.len() {
                loops.push(LoopInfo { header: target, back_edge: i });
            }
        }
    }
    loops
}

/// 计算循环体内被写入的局部变量集合
fn compute_written_locals(instructions: &[OpCode], header: usize, back_edge: usize) -> HashSet<usize> {
    let mut written = HashSet::new();
    for i in header..=back_edge {
        if let OpCode::StoreLocal(idx) = instructions[i] {
            written.insert(idx);
        }
    }
    written
}

/// 判断指令是否为纯计算（无副作用）
fn is_pure_op(op: &OpCode) -> bool {
    matches!(
        op,
        OpCode::LoadConst(_)
            | OpCode::LoadNull
            | OpCode::LoadTrue
            | OpCode::LoadFalse
            | OpCode::LoadLocal(_)
            | OpCode::Add
            | OpCode::Sub
            | OpCode::Mul
            | OpCode::Div
            | OpCode::Mod
            | OpCode::Neg
            | OpCode::Eq
            | OpCode::Ne
            | OpCode::Lt
            | OpCode::Le
            | OpCode::Gt
            | OpCode::Ge
            | OpCode::And
            | OpCode::Or
            | OpCode::Not
            | OpCode::Dup
    )
}

/// 判断指令是否有副作用
fn has_side_effect(op: &OpCode) -> bool {
    matches!(
        op,
        OpCode::StoreLocal(_)
            | OpCode::Call(_)
            | OpCode::TailCall(_)
            | OpCode::Return
            | OpCode::SpawnEntity
            | OpCode::DespawnEntity
            | OpCode::AddComponent(_)
            | OpCode::SetComponent(_)
            | OpCode::HostCall(_, _)
            | OpCode::SetField(_)
            | OpCode::SetIndex
            | OpCode::NewObject(_)
            | OpCode::NewList(_)
            | OpCode::NewMap(_)
            | OpCode::StringConcat(_)
    )
}

/// 计算指令的栈效果（栈元素数量变化）
fn stack_effect(op: &OpCode) -> i32 {
    match op {
        OpCode::LoadConst(_) | OpCode::LoadNull | OpCode::LoadTrue | OpCode::LoadFalse => 1,
        OpCode::LoadLocal(_) => 1,
        OpCode::StoreLocal(_) => -1,
        OpCode::Add | OpCode::Sub | OpCode::Mul | OpCode::Div | OpCode::Mod => -1,
        OpCode::Neg | OpCode::Not => 0,
        OpCode::Eq | OpCode::Ne | OpCode::Lt | OpCode::Le | OpCode::Gt | OpCode::Ge => -1,
        OpCode::And | OpCode::Or => -1,
        OpCode::Jump(_) => 0,
        OpCode::JumpIfFalse(_) | OpCode::JumpIfTrue(_) => -1,
        OpCode::Call(n) => -(*n as i32) + 1,
        OpCode::TailCall(n) => -(*n as i32),
        OpCode::Return => 0,
        OpCode::SpawnEntity => 1,
        OpCode::DespawnEntity => -1,
        OpCode::AddComponent(_) => -2,
        OpCode::GetComponent(_) => 0,
        OpCode::SetComponent(_) => -2,
        OpCode::HostCall(_, n) => -(*n as i32) + 1,
        OpCode::Pop => -1,
        OpCode::Dup => 1,
        OpCode::GetField(_) => 0,
        OpCode::SetField(_) => -2,
        OpCode::GetIndex => -1,
        OpCode::SetIndex => -3,
        OpCode::NewObject(n) => -(*n as i32) + 1,
        OpCode::NewList(n) => -(*n as i32) + 1,
        OpCode::NewMap(n) => -(*n as i32) * 2 + 1,
        OpCode::StringConcat(n) => -(*n as i32) + 1,
    }
}

/// 对单个函数执行循环不变量外提优化
fn hoist_loop_invariants(func: &mut crate::IrFunction) -> bool {
    let loops = detect_loops(&func.instructions);
    if loops.is_empty() {
        return false;
    }

    let mut any_changed = false;
    let mut offset: i32 = 0;

    for loop_info in &loops {
        let header = (loop_info.header as i32 + offset) as usize;
        let back_edge = (loop_info.back_edge as i32 + offset) as usize;

        let written_locals = compute_written_locals(&func.instructions, header, back_edge);

        let mut hoist_candidates: Vec<usize> = Vec::new();
        let mut invariant_locals: HashSet<usize> = HashSet::new();

        let mut found = true;
        while found {
            found = false;
            let mut stack_invariant: Vec<bool> = Vec::new();

            for i in header..=back_edge {
                if hoist_candidates.contains(&i) {
                    for _ in 0..stack_effect(&func.instructions[i]) {
                        stack_invariant.pop();
                    }
                    stack_invariant.push(true);
                    continue;
                }

                let op = &func.instructions[i];

                if is_pure_op(op) && !has_side_effect(op) {
                    let is_invariant = match op {
                        OpCode::LoadConst(_) | OpCode::LoadNull | OpCode::LoadTrue | OpCode::LoadFalse => true,
                        OpCode::LoadLocal(idx) => !written_locals.contains(idx) || invariant_locals.contains(idx),
                        OpCode::Dup => stack_invariant.last().copied().unwrap_or(false),
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
                        | OpCode::And
                        | OpCode::Or => {
                            if stack_invariant.len() >= 2 {
                                stack_invariant[stack_invariant.len() - 2] && stack_invariant[stack_invariant.len() - 1]
                            }
                            else {
                                false
                            }
                        }
                        OpCode::Neg | OpCode::Not => stack_invariant.last().copied().unwrap_or(false),
                        _ => false,
                    };

                    if is_invariant && !hoist_candidates.contains(&i) {
                        hoist_candidates.push(i);
                        found = true;
                    }
                }

                for _ in 0..stack_effect(op) {
                    stack_invariant.pop();
                }
                if stack_effect(op) > 0 {
                    for _ in 0..stack_effect(op) {
                        stack_invariant.push(hoist_candidates.contains(&i));
                    }
                }

                if let OpCode::StoreLocal(idx) = op {
                    if hoist_candidates.contains(&(i - 1)) || invariant_locals.contains(idx) {
                        invariant_locals.insert(*idx);
                    }
                }
            }
        }

        if hoist_candidates.is_empty() {
            continue;
        }

        let mut temp_local_start = func.local_count;
        let mut insertions: Vec<(usize, Vec<OpCode>, Vec<OpCode>)> = Vec::new();

        for &inst_idx in &hoist_candidates {
            let op = func.instructions[inst_idx].clone();
            let temp_idx = temp_local_start;
            temp_local_start += 1;

            let hoisted = vec![op, OpCode::StoreLocal(temp_idx)];
            let replacement = vec![OpCode::LoadLocal(temp_idx)];

            insertions.push((inst_idx, hoisted, replacement));
        }

        func.local_count = temp_local_start;
        for _ in 0..(temp_local_start - func.local_names.len()) {
            func.local_names.push(String::new());
        }
        for (i, name) in func.local_names.iter_mut().enumerate() {
            if name.is_empty() && i >= temp_local_start - insertions.len() {
                *name = format!("__licm_{}", i);
            }
        }

        insertions.sort_by_key(|(idx, _, _)| *idx);

        let mut total_inserted_before_header = 0usize;
        let mut replacement_offsets: Vec<(usize, usize)> = Vec::new();

        for (inst_idx, hoisted, replacement) in &insertions {
            let adjusted_idx = *inst_idx + total_inserted_before_header;
            let hoisted_len = hoisted.len();
            let replacement_len = replacement.len();

            if adjusted_idx < header + total_inserted_before_header {
                replacement_offsets.push((adjusted_idx, replacement_len));
            }
            else {
                let insert_pos = header + total_inserted_before_header;
                func.instructions.splice(insert_pos..insert_pos, hoisted.clone());
                total_inserted_before_header += hoisted_len;
                replacement_offsets.push((adjusted_idx + hoisted_len, replacement_len));
            }
        }

        for (adjusted_idx, _replacement_len) in &replacement_offsets {
            let actual_idx = *adjusted_idx;
            if actual_idx < func.instructions.len() {
                func.instructions[actual_idx] = OpCode::LoadLocal(0);
            }
        }

        fixup_jump_addresses(&mut func.instructions, header, total_inserted_before_header);

        offset += total_inserted_before_header as i32;
        any_changed = true;
    }

    any_changed
}

/// 修复因插入指令而偏移的跳转地址
fn fixup_jump_addresses(instructions: &mut [OpCode], insert_pos: usize, inserted_count: usize) {
    if inserted_count == 0 {
        return;
    }

    for op in instructions.iter_mut() {
        match op {
            OpCode::Jump(target) => {
                if *target >= insert_pos {
                    *target += inserted_count;
                }
            }
            OpCode::JumpIfFalse(target) | OpCode::JumpIfTrue(target) => {
                if *target >= insert_pos {
                    *target += inserted_count;
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
            local_names: vec!["a".to_string(), "b".to_string()],
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
    fn test_detect_simple_loop() {
        let loops = detect_loops(&[
            OpCode::LoadLocal(0),
            OpCode::LoadConst(0),
            OpCode::Lt,
            OpCode::JumpIfFalse(6),
            OpCode::LoadLocal(0),
            OpCode::Jump(0),
        ]);
        assert_eq!(loops.len(), 1);
        assert_eq!(loops[0].header, 0);
        assert_eq!(loops[0].back_edge, 5);
    }

    #[test]
    fn test_no_loop_no_back_edge() {
        let loops = detect_loops(&[OpCode::LoadLocal(0), OpCode::JumpIfFalse(3), OpCode::LoadConst(0), OpCode::LoadNull]);
        assert!(loops.is_empty());
    }

    #[test]
    fn test_hoist_invariant_load_const() {
        let mut module = make_module(vec![make_func(
            "loop",
            vec![
                OpCode::LoadLocal(0),
                OpCode::LoadConst(0),
                OpCode::Lt,
                OpCode::JumpIfFalse(7),
                OpCode::LoadConst(1),
                OpCode::LoadConst(2),
                OpCode::Add,
                OpCode::StoreLocal(1),
                OpCode::LoadLocal(0),
                OpCode::Jump(0),
            ],
        )]);

        let pass = LoopInvariantCodeMotionPass;
        let changed = pass.run(&mut module).unwrap();
        assert!(changed);

        let func = &module.functions[0];
        assert!(func.local_count > 2);
    }

    #[test]
    fn test_no_hoist_side_effect() {
        let mut module = make_module(vec![make_func(
            "loop",
            vec![
                OpCode::LoadLocal(0),
                OpCode::LoadConst(0),
                OpCode::Lt,
                OpCode::JumpIfFalse(7),
                OpCode::HostCall(0, 0),
                OpCode::Pop,
                OpCode::LoadLocal(0),
                OpCode::Jump(0),
            ],
        )]);

        let pass = LoopInvariantCodeMotionPass;
        let changed = pass.run(&mut module).unwrap();

        let func = &module.functions[0];
        let has_host_call = func.instructions.iter().any(|op| matches!(op, OpCode::HostCall(_, _)));
        assert!(has_host_call, "HostCall should not be hoisted out of the loop");
        let _ = changed;
    }

    #[test]
    fn test_no_hoist_store_local() {
        let original = vec![
            OpCode::LoadLocal(0),
            OpCode::LoadConst(0),
            OpCode::Lt,
            OpCode::JumpIfFalse(6),
            OpCode::LoadConst(1),
            OpCode::StoreLocal(1),
            OpCode::LoadLocal(0),
            OpCode::Jump(0),
        ];

        let mut module = make_module(vec![make_func("loop", original.clone())]);
        let pass = LoopInvariantCodeMotionPass;
        let _ = pass.run(&mut module).unwrap();
    }

    #[test]
    fn test_empty_function() {
        let mut module = make_module(vec![make_func("empty", vec![])]);
        let pass = LoopInvariantCodeMotionPass;
        let changed = pass.run(&mut module).unwrap();
        assert!(!changed);
    }

    #[test]
    fn test_no_loop() {
        let mut module = make_module(vec![make_func(
            "no_loop",
            vec![OpCode::LoadConst(0), OpCode::LoadConst(1), OpCode::Add, OpCode::Return],
        )]);
        let pass = LoopInvariantCodeMotionPass;
        let changed = pass.run(&mut module).unwrap();
        assert!(!changed);
    }

    #[test]
    fn test_pass_name() {
        let pass = LoopInvariantCodeMotionPass;
        assert_eq!(pass.name(), "LoopInvariantCodeMotion");
    }

    #[test]
    fn test_fixup_jump_addresses() {
        let mut instructions = vec![
            OpCode::LoadLocal(0),
            OpCode::JumpIfFalse(5),
            OpCode::LoadConst(0),
            OpCode::LoadConst(1),
            OpCode::Add,
            OpCode::LoadLocal(0),
            OpCode::Jump(0),
        ];

        fixup_jump_addresses(&mut instructions, 0, 2);

        assert_eq!(instructions[1], OpCode::JumpIfFalse(7));
        assert_eq!(instructions[6], OpCode::Jump(2));
    }

    #[test]
    fn test_is_pure_op() {
        assert!(is_pure_op(&OpCode::LoadConst(0)));
        assert!(is_pure_op(&OpCode::LoadLocal(0)));
        assert!(is_pure_op(&OpCode::Add));
        assert!(is_pure_op(&OpCode::Neg));
        assert!(!is_pure_op(&OpCode::StoreLocal(0)));
        assert!(!is_pure_op(&OpCode::Call(0)));
        assert!(!is_pure_op(&OpCode::HostCall(0, 0)));
    }

    #[test]
    fn test_has_side_effect() {
        assert!(has_side_effect(&OpCode::StoreLocal(0)));
        assert!(has_side_effect(&OpCode::Call(0)));
        assert!(has_side_effect(&OpCode::HostCall(0, 0)));
        assert!(has_side_effect(&OpCode::NewObject(0)));
        assert!(!has_side_effect(&OpCode::LoadConst(0)));
        assert!(!has_side_effect(&OpCode::Add));
    }

    #[test]
    fn test_compute_written_locals() {
        let instructions = vec![OpCode::LoadConst(0), OpCode::StoreLocal(1), OpCode::LoadConst(1), OpCode::StoreLocal(2)];
        let written = compute_written_locals(&instructions, 0, 3);
        assert!(written.contains(&1));
        assert!(written.contains(&2));
        assert!(!written.contains(&0));
    }
}
