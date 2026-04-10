#![warn(missing_docs)]

//! 常量折叠优化 Pass
//! 识别编译时可计算的常量表达式并折叠为常量加载

use crate::{IrModule, IrValue, OpCode, pass::IrPass};
use gg_core::GResult;

/// 常量折叠优化 Pass
/// 识别编译时可计算的常量表达式并折叠为常量加载
pub struct ConstantFoldPass;

impl IrPass for ConstantFoldPass {
    fn name(&self) -> &str {
        "constant_fold"
    }

    fn run(&self, module: &mut IrModule) -> GResult<bool> {
        let mut any_changed = false;
        for func_idx in 0..module.functions.len() {
            if fold_function_for_module(module, func_idx) {
                any_changed = true;
            }
        }
        Ok(any_changed)
    }
}

/// 对单个函数执行常量折叠，返回是否做了修改
fn fold_function_for_module(module: &mut IrModule, func_idx: usize) -> bool {
    let mut changed = false;
    let mut i = 0;

    while i < module.functions[func_idx].instructions.len() {
        let action = {
            let instructions = &module.functions[func_idx].instructions;
            find_fold_action(&module.constants, instructions, i)
        };

        match action {
            Some(FoldAction::Binary { result, consumed }) => {
                let result_idx = module.add_or_get_constant(result);
                let instructions = &mut module.functions[func_idx].instructions;
                instructions.splice(i..i + consumed, std::iter::once(OpCode::LoadConst(result_idx)));
                changed = true;
            }
            Some(FoldAction::Unary { result, consumed }) => {
                let result_idx = module.add_or_get_constant(result);
                let instructions = &mut module.functions[func_idx].instructions;
                instructions.splice(i..i + consumed, std::iter::once(OpCode::LoadConst(result_idx)));
                changed = true;
            }
            None => {
                i += 1;
            }
        }
    }

    changed
}

/// 折叠动作
enum FoldAction {
    /// 二元运算折叠
    Binary {
        /// 折叠结果
        result: IrValue,
        /// 消耗的指令数
        consumed: usize,
    },
    /// 一元运算折叠
    Unary {
        /// 折叠结果
        result: IrValue,
        /// 消耗的指令数
        consumed: usize,
    },
}

/// 在位置 i 处查找折叠机会
fn find_fold_action(constants: &[IrValue], instructions: &[OpCode], i: usize) -> Option<FoldAction> {
    if let OpCode::LoadConst(a_idx) = instructions[i] {
        let a_val = constants.get(a_idx)?;

        if i + 2 < instructions.len() {
            if let OpCode::LoadConst(b_idx) = instructions[i + 1] {
                let b_val = constants.get(b_idx)?;
                let op = &instructions[i + 2];

                if let Some(result) = try_fold_binary(a_val, b_val, op) {
                    return Some(FoldAction::Binary {
                        result,
                        consumed: 3,
                    });
                }

                if let Some(result) = try_fold_comparison(a_val, b_val, op) {
                    return Some(FoldAction::Binary {
                        result,
                        consumed: 3,
                    });
                }
            }
        }

        if i + 1 < instructions.len() {
            if let Some(result) = try_fold_unary(a_val, &instructions[i + 1]) {
                return Some(FoldAction::Unary {
                    result,
                    consumed: 2,
                });
            }
        }
    }

    None
}

/// 尝试常量折叠二元算术运算
fn try_fold_binary(a: &IrValue, b: &IrValue, op: &OpCode) -> Option<IrValue> {
    match op {
        OpCode::Add => match (a, b) {
            (IrValue::Int(x), IrValue::Int(y)) => Some(IrValue::Int(x + y)),
            (IrValue::Float(x), IrValue::Float(y)) => Some(IrValue::Float(x + y)),
            _ => None,
        },
        OpCode::Sub => match (a, b) {
            (IrValue::Int(x), IrValue::Int(y)) => Some(IrValue::Int(x - y)),
            (IrValue::Float(x), IrValue::Float(y)) => Some(IrValue::Float(x - y)),
            _ => None,
        },
        OpCode::Mul => match (a, b) {
            (IrValue::Int(x), IrValue::Int(y)) => Some(IrValue::Int(x * y)),
            (IrValue::Float(x), IrValue::Float(y)) => Some(IrValue::Float(x * y)),
            _ => None,
        },
        OpCode::Div => match (a, b) {
            (IrValue::Int(x), IrValue::Int(y)) => {
                if *y != 0 {
                    Some(IrValue::Int(x / y))
                } else {
                    None
                }
            }
            (IrValue::Float(x), IrValue::Float(y)) => {
                if *y != 0.0 {
                    Some(IrValue::Float(x / y))
                } else {
                    None
                }
            }
            _ => None,
        },
        OpCode::Mod => match (a, b) {
            (IrValue::Int(x), IrValue::Int(y)) => {
                if *y != 0 {
                    Some(IrValue::Int(x % y))
                } else {
                    None
                }
            }
            (IrValue::Float(x), IrValue::Float(y)) => {
                if *y != 0.0 {
                    Some(IrValue::Float(x % y))
                } else {
                    None
                }
            }
            _ => None,
        },
        _ => None,
    }
}

/// 尝试常量折叠比较运算
fn try_fold_comparison(a: &IrValue, b: &IrValue, op: &OpCode) -> Option<IrValue> {
    match op {
        OpCode::Eq => Some(IrValue::Bool(a == b)),
        OpCode::Ne => Some(IrValue::Bool(a != b)),
        OpCode::Lt => compare_values(a, b, |x, y| x < y),
        OpCode::Le => compare_values(a, b, |x, y| x <= y),
        OpCode::Gt => compare_values(a, b, |x, y| x > y),
        OpCode::Ge => compare_values(a, b, |x, y| x >= y),
        _ => None,
    }
}

/// 对两个值执行比较运算
fn compare_values<F>(a: &IrValue, b: &IrValue, cmp: F) -> Option<IrValue>
where
    F: Fn(&IrValue, &IrValue) -> bool,
{
    match (a, b) {
        (IrValue::Int(x), IrValue::Int(y)) => Some(IrValue::Bool(cmp(
            &IrValue::Int(*x),
            &IrValue::Int(*y),
        ))),
        (IrValue::Float(x), IrValue::Float(y)) => Some(IrValue::Bool(cmp(
            &IrValue::Float(*x),
            &IrValue::Float(*y),
        ))),
        (IrValue::Bool(x), IrValue::Bool(y)) => Some(IrValue::Bool(cmp(
            &IrValue::Bool(*x),
            &IrValue::Bool(*y),
        ))),
        (IrValue::String(x), IrValue::String(y)) => Some(IrValue::Bool(cmp(
            &IrValue::String(x.clone()),
            &IrValue::String(y.clone()),
        ))),
        _ => None,
    }
}

/// 尝试常量折叠一元运算
fn try_fold_unary(a: &IrValue, op: &OpCode) -> Option<IrValue> {
    match op {
        OpCode::Neg => match a {
            IrValue::Int(x) => Some(IrValue::Int(-x)),
            IrValue::Float(x) => Some(IrValue::Float(-x)),
            _ => None,
        },
        OpCode::Not => match a {
            IrValue::Bool(x) => Some(IrValue::Bool(!x)),
            _ => None,
        },
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{IrFunction, IrModule, IrValue, OpCode, pass::IrPass};

    /// 测试 LoadConst(1) + LoadConst(2) 折叠为 LoadConst(3)
    #[test]
    fn test_fold_add_constants() {
        let mut module = IrModule::new("test");
        let a_idx = module.add_constant(IrValue::Int(1));
        let b_idx = module.add_constant(IrValue::Int(2));

        module.add_function(IrFunction {
            name: "main".to_string(),
            param_count: 0,
            local_count: 0,
            instructions: vec![
                OpCode::LoadConst(a_idx),
                OpCode::LoadConst(b_idx),
                OpCode::Add,
                OpCode::Return,
            ],
        });

        let pass = ConstantFoldPass;
        let changed = pass.run(&mut module).unwrap();
        assert!(changed);

        let func = &module.functions[0];
        assert_eq!(func.instructions.len(), 2);
        match &func.instructions[0] {
            OpCode::LoadConst(idx) => {
                assert_eq!(module.constants[*idx], IrValue::Int(3));
            }
            other => panic!("期望 LoadConst，实际: {:?}", other),
        }
    }

    /// 测试 LoadConst(true), Not 折叠为 LoadConst(false)
    #[test]
    fn test_fold_not_bool() {
        let mut module = IrModule::new("test");
        let idx = module.add_constant(IrValue::Bool(true));

        module.add_function(IrFunction {
            name: "main".to_string(),
            param_count: 0,
            local_count: 0,
            instructions: vec![
                OpCode::LoadConst(idx),
                OpCode::Not,
                OpCode::Return,
            ],
        });

        let pass = ConstantFoldPass;
        let changed = pass.run(&mut module).unwrap();
        assert!(changed);

        let func = &module.functions[0];
        assert_eq!(func.instructions.len(), 2);
        match &func.instructions[0] {
            OpCode::LoadConst(idx) => {
                assert_eq!(module.constants[*idx], IrValue::Bool(false));
            }
            other => panic!("期望 LoadConst，实际: {:?}", other),
        }
    }

    /// 测试非常量表达式不会被折叠
    #[test]
    fn test_non_constant_expressions_not_folded() {
        let mut module = IrModule::new("test");
        let a_idx = module.add_constant(IrValue::Int(1));

        module.add_function(IrFunction {
            name: "main".to_string(),
            param_count: 0,
            local_count: 1,
            instructions: vec![
                OpCode::LoadConst(a_idx),
                OpCode::LoadLocal(0),
                OpCode::Add,
                OpCode::Return,
            ],
        });

        let pass = ConstantFoldPass;
        let changed = pass.run(&mut module).unwrap();
        assert!(!changed);

        let func = &module.functions[0];
        assert_eq!(func.instructions.len(), 4);
    }
}
