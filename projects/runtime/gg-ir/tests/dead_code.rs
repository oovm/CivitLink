use gg_ir::{IrFunction, IrModule, IrValue, OpCode, dead_code::DeadCodeElimPass};

/// 测试无条件 Jump 之后的代码被移除
#[test]
fn test_dead_code_after_unconditional_jump_removed() {
    let mut module = IrModule::new("test");
    let val_idx = module.add_constant(IrValue::Int(1));

    module.add_function(IrFunction {
        name: "main".to_string(),
        param_count: 0,
        local_count: 0,
        instructions: vec![OpCode::LoadConst(val_idx), OpCode::Return, OpCode::LoadConst(val_idx), OpCode::Pop],
    });

    let pass = DeadCodeElimPass;
    let changed = pass.run(&mut module).unwrap();

    assert!(changed);

    let func = &module.functions[0];
    assert_eq!(func.instructions.len(), 2);
    assert!(matches!(func.instructions[0], OpCode::LoadConst(_)));
    assert!(matches!(func.instructions[1], OpCode::Return));
}

/// 测试可达代码被保留
#[test]
fn test_reachable_code_preserved() {
    let mut module = IrModule::new("test");
    let val_idx = module.add_constant(IrValue::Int(1));

    module.add_function(IrFunction {
        name: "main".to_string(),
        param_count: 0,
        local_count: 0,
        instructions: vec![
            OpCode::LoadConst(val_idx),
            OpCode::LoadTrue,
            OpCode::JumpIfFalse(4),
            OpCode::LoadConst(val_idx),
            OpCode::Return,
        ],
    });

    let pass = DeadCodeElimPass;
    let changed = pass.run(&mut module).unwrap();

    assert!(!changed);

    let func = &module.functions[0];
    assert_eq!(func.instructions.len(), 5);
}

/// 测试删除后跳转地址被正确修复
#[test]
fn test_jump_addresses_fixed_up_after_removal() {
    let mut module = IrModule::new("test");
    let val_idx = module.add_constant(IrValue::Int(1));

    module.add_function(IrFunction {
        name: "main".to_string(),
        param_count: 0,
        local_count: 0,
        instructions: vec![
            OpCode::LoadTrue,
            OpCode::JumpIfFalse(5),
            OpCode::LoadConst(val_idx),
            OpCode::Return,
            OpCode::LoadConst(val_idx),
            OpCode::LoadConst(val_idx),
            OpCode::Pop,
            OpCode::Return,
        ],
    });

    let pass = DeadCodeElimPass;
    let _ = pass.run(&mut module).unwrap();

    let func = &module.functions[0];
    for (i, instr) in func.instructions.iter().enumerate() {
        match instr {
            OpCode::Jump(target) => {
                assert!(
                    *target < func.instructions.len(),
                    "Jump 目标 {} 超出指令范围 {} ( 指令 {})",
                    target,
                    func.instructions.len(),
                    i
                );
            }
            OpCode::JumpIfFalse(target) => {
                assert!(
                    *target < func.instructions.len(),
                    "JumpIfFalse 目标 {} 超出指令范围 {} (指令 {})",
                    target,
                    func.instructions.len(),
                    i
                );
            }
            OpCode::JumpIfTrue(target) => {
                assert!(
                    *target < func.instructions.len(),
                    "JumpIfTrue 目标 {} 超出指令范围 {} (指令 {})",
                    target,
                    func.instructions.len(),
                    i
                );
            }
            _ => {}
        }
    }
}
