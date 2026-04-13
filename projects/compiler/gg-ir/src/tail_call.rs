#![warn(missing_docs)]

//! 尾调用优化 Pass
//! 识别尾位置的 Call 指令并将其替换为 TailCall 指令，实现尾调用优化

use crate::{IrModule, OpCode, pass::IrPass};
use gg_core::GResult;

/// 尾调用优化 Pass
/// 扫描函数中的指令序列，将紧接 Return 的 Call 替换为 TailCall
pub struct TailCallOptPass;

impl IrPass for TailCallOptPass {
    fn name(&self) -> &str {
        "TailCallOpt"
    }

    fn run(&self, module: &mut IrModule) -> GResult<bool> {
        let mut any_changed = false;
        for func in &mut module.functions {
            if optimize_tail_calls(func) {
                any_changed = true;
            }
        }
        Ok(any_changed)
    }
}

/// 对单个函数执行尾调用优化，返回是否做了修改
fn optimize_tail_calls(func: &mut crate::IrFunction) -> bool {
    let mut changed = false;
    let len = func.instructions.len();

    if len < 2 {
        return false;
    }

    for i in 0..len - 1 {
        if let OpCode::Call(arity) = func.instructions[i]
            && func.instructions[i + 1] == OpCode::Return
        {
            func.instructions[i] = OpCode::TailCall(arity);
            changed = true;
        }
    }

    changed
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{IrFunction, IrModule};

    fn make_func(name: &str, instructions: Vec<OpCode>) -> IrFunction {
        IrFunction {
            name: name.to_string(),
            param_count: 0,
            local_count: 0,
            local_names: vec![],
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
    fn test_simple_tail_call() {
        let mut module = make_module(vec![make_func("f", vec![OpCode::Call(0), OpCode::Return])]);
        let pass = TailCallOptPass;
        let changed = pass.run(&mut module).unwrap();
        assert!(changed);
        assert_eq!(module.functions[0].instructions, vec![OpCode::TailCall(0), OpCode::Return,]);
    }

    #[test]
    fn test_tail_call_with_arity() {
        let mut module = make_module(vec![make_func("f", vec![OpCode::LoadConst(0), OpCode::Call(1), OpCode::Return])]);
        let pass = TailCallOptPass;
        let changed = pass.run(&mut module).unwrap();
        assert!(changed);
        assert_eq!(module.functions[0].instructions, vec![OpCode::LoadConst(0), OpCode::TailCall(1), OpCode::Return,]);
    }

    #[test]
    fn test_non_tail_call() {
        let mut module =
            make_module(vec![make_func("f", vec![OpCode::Call(0), OpCode::Pop, OpCode::LoadNull, OpCode::Return])]);
        let pass = TailCallOptPass;
        let changed = pass.run(&mut module).unwrap();
        assert!(!changed);
        assert_eq!(module.functions[0].instructions, vec![OpCode::Call(0), OpCode::Pop, OpCode::LoadNull, OpCode::Return,]);
    }

    #[test]
    fn test_call_without_return() {
        let mut module = make_module(vec![make_func("f", vec![OpCode::Call(0)])]);
        let pass = TailCallOptPass;
        let changed = pass.run(&mut module).unwrap();
        assert!(!changed);
        assert_eq!(module.functions[0].instructions, vec![OpCode::Call(0),]);
    }

    #[test]
    fn test_recursive_tail_call() {
        let mut module = make_module(vec![make_func(
            "factorial",
            vec![
                OpCode::LoadLocal(0),
                OpCode::LoadConst(0),
                OpCode::Eq,
                OpCode::JumpIfFalse(6),
                OpCode::LoadConst(1),
                OpCode::Return,
                OpCode::LoadLocal(0),
                OpCode::LoadLocal(0),
                OpCode::LoadConst(1),
                OpCode::Sub,
                OpCode::Call(2),
                OpCode::Mul,
                OpCode::Return,
            ],
        )]);
        let pass = TailCallOptPass;
        let changed = pass.run(&mut module).unwrap();
        assert!(!changed);
        assert_eq!(module.functions[0].instructions[10], OpCode::Call(2));
        assert_eq!(module.functions[0].instructions[11], OpCode::Mul);
        assert_eq!(module.functions[0].instructions[12], OpCode::Return);
    }

    #[test]
    fn test_tail_call_after_jump_target() {
        let mut module = make_module(vec![make_func(
            "f",
            vec![OpCode::LoadTrue, OpCode::JumpIfFalse(4), OpCode::Call(0), OpCode::Return, OpCode::LoadNull, OpCode::Return],
        )]);
        let pass = TailCallOptPass;
        let changed = pass.run(&mut module).unwrap();
        assert!(changed);
        assert_eq!(module.functions[0].instructions[2], OpCode::TailCall(0));
        assert_eq!(module.functions[0].instructions[3], OpCode::Return);
        assert_eq!(module.functions[0].instructions[4], OpCode::LoadNull);
        assert_eq!(module.functions[0].instructions[5], OpCode::Return);
    }

    #[test]
    fn test_empty_function() {
        let mut module = make_module(vec![make_func("f", vec![])]);
        let pass = TailCallOptPass;
        let changed = pass.run(&mut module).unwrap();
        assert!(!changed);
        assert!(module.functions[0].instructions.is_empty());
    }

    #[test]
    fn test_only_return() {
        let mut module = make_module(vec![make_func("f", vec![OpCode::Return])]);
        let pass = TailCallOptPass;
        let changed = pass.run(&mut module).unwrap();
        assert!(!changed);
        assert_eq!(module.functions[0].instructions, vec![OpCode::Return]);
    }

    #[test]
    fn test_multiple_tail_calls_in_branches() {
        let mut module = make_module(vec![make_func(
            "f",
            vec![
                OpCode::LoadLocal(0),
                OpCode::JumpIfFalse(5),
                OpCode::LoadConst(0),
                OpCode::Call(1),
                OpCode::Return,
                OpCode::LoadConst(1),
                OpCode::Call(2),
                OpCode::Return,
            ],
        )]);
        let pass = TailCallOptPass;
        let changed = pass.run(&mut module).unwrap();
        assert!(changed);
        assert_eq!(
            module.functions[0].instructions,
            vec![
                OpCode::LoadLocal(0),
                OpCode::JumpIfFalse(5),
                OpCode::LoadConst(0),
                OpCode::TailCall(1),
                OpCode::Return,
                OpCode::LoadConst(1),
                OpCode::TailCall(2),
                OpCode::Return,
            ]
        );
    }

    #[test]
    fn test_already_tail_call() {
        let mut module = make_module(vec![make_func("f", vec![OpCode::TailCall(0), OpCode::Return])]);
        let pass = TailCallOptPass;
        let changed = pass.run(&mut module).unwrap();
        assert!(!changed);
        assert_eq!(module.functions[0].instructions, vec![OpCode::TailCall(0), OpCode::Return,]);
    }

    #[test]
    fn test_multiple_functions() {
        let mut module = make_module(vec![
            make_func("f1", vec![OpCode::Call(0), OpCode::Return]),
            make_func("f2", vec![OpCode::Call(0), OpCode::Pop, OpCode::Return]),
        ]);
        let pass = TailCallOptPass;
        let changed = pass.run(&mut module).unwrap();
        assert!(changed);
        assert_eq!(module.functions[0].instructions, vec![OpCode::TailCall(0), OpCode::Return]);
        assert_eq!(module.functions[1].instructions, vec![OpCode::Call(0), OpCode::Pop, OpCode::Return]);
    }
}
