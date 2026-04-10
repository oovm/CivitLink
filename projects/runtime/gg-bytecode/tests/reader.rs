use gg_bytecode::{
    format::{BytecodeInstruction, BytecodeValue, MAGIC, VERSION},
    reader::BytecodeReader,
    writer::BytecodeWriter,
};
use gg_ir::{IrFunction, IrModule, IrValue, OpCode};

/// 测试读取无效魔数返回错误
#[test]
fn test_read_invalid_magic_returns_error() {
    let mut data = Vec::new();
    data.extend_from_slice(&0xDEADBEEFu32.to_le_bytes());
    data.extend_from_slice(&VERSION.to_le_bytes());
    data.extend_from_slice(&(0u32).to_le_bytes());
    let result = BytecodeReader::read(&data);
    assert!(result.is_err());
}

/// 测试读取无效版本号返回错误
#[test]
fn test_read_invalid_version_returns_error() {
    let mut data = Vec::new();
    data.extend_from_slice(&MAGIC.to_le_bytes());
    data.extend_from_slice(&9999u32.to_le_bytes());
    data.extend_from_slice(&(0u32).to_le_bytes());
    let result = BytecodeReader::read(&data);
    assert!(result.is_err());
}

/// 测试写入后读回的结构一致性
#[test]
fn test_round_trip_read_structure() {
    let mut ir_module = IrModule::new("round_trip_test");
    ir_module.add_constant(IrValue::Int(100));
    ir_module.add_constant(IrValue::Float(3.14));
    ir_module.add_constant(IrValue::Bool(false));
    ir_module.add_constant(IrValue::String("test".to_string()));
    ir_module.add_constant(IrValue::Null);

    let func = IrFunction {
        name: "compute".to_string(),
        param_count: 2,
        local_count: 3,
        instructions: vec![
            OpCode::LoadConst(0),
            OpCode::LoadConst(1),
            OpCode::Add,
            OpCode::StoreLocal(0),
            OpCode::LoadLocal(0),
            OpCode::Return,
        ],
    };

    ir_module.add_function(func);

    let data = BytecodeWriter::write(&ir_module).unwrap();
    let module = BytecodeReader::read(&data).unwrap();

    assert_eq!(module.name, "round_trip_test");
    assert_eq!(module.constants.len(), 5);
    assert_eq!(module.constants[0], BytecodeValue::Int(100));
    assert_eq!(module.constants[1], BytecodeValue::Float(3.14));
    assert_eq!(module.constants[2], BytecodeValue::Bool(false));
    assert_eq!(module.constants[3], BytecodeValue::String("test".to_string()));
    assert_eq!(module.constants[4], BytecodeValue::Null);
    assert_eq!(module.functions.len(), 1);
    assert_eq!(module.functions[0].name, "compute");
    assert_eq!(module.functions[0].param_count, 2);
    assert_eq!(module.functions[0].local_count, 3);
    assert_eq!(module.functions[0].instructions.len(), 6);
    assert!(matches!(module.functions[0].instructions[0], BytecodeInstruction::LoadConst { index: 0 }));
    assert!(matches!(module.functions[0].instructions[2], BytecodeInstruction::Add));
}
