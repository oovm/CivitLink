use gg_bytecode::{
    DebugInfo, SourceLocation,
    format::{BytecodeInstruction, BytecodeModule, BytecodeValue, MAGIC, VERSION},
    reader::BytecodeReader,
    writer::BytecodeWriter,
};
use gg_ir::{IrFunction, IrModule, IrValue, OpCode};

#[test]
fn test_read_invalid_magic_returns_error() {
    let mut data = Vec::new();
    data.extend_from_slice(&0xDEADBEEFu32.to_le_bytes());
    data.extend_from_slice(&VERSION.to_le_bytes());
    data.extend_from_slice(&(0u32).to_le_bytes());
    let result = BytecodeReader::read(&data);
    assert!(result.is_err());
}

#[test]
fn test_read_invalid_version_returns_error() {
    let mut data = Vec::new();
    data.extend_from_slice(&MAGIC.to_le_bytes());
    data.extend_from_slice(&9999u32.to_le_bytes());
    data.extend_from_slice(&(0u32).to_le_bytes());
    let result = BytecodeReader::read(&data);
    assert!(result.is_err());
}

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
        local_names: vec![],
        instructions: vec![
            OpCode::LoadConst(0),
            OpCode::LoadConst(1),
            OpCode::Add,
            OpCode::StoreLocal(0),
            OpCode::LoadLocal(0),
            OpCode::Return,
        ],
        is_entry: false,
        target: None,
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
    assert!(matches!(module.functions[0].instructions[0], BytecodeInstruction::LoadConstInt { index: 0 }));
    assert!(matches!(module.functions[0].instructions[2], BytecodeInstruction::Add));
    assert_eq!(module.entry_points.len(), 0);
}

#[test]
fn test_read_entry_points() {
    let mut ir_module = IrModule::new("entry_reader_test");
    ir_module.add_constant(IrValue::Int(10));

    let func = IrFunction {
        name: "start".to_string(),
        param_count: 0,
        local_count: 0,
        local_names: vec![],
        instructions: vec![OpCode::LoadConst(0), OpCode::Return],
        is_entry: false,
        target: None,
    };

    ir_module.add_function(func);

    let data = BytecodeWriter::write(&ir_module).unwrap();
    let read_module = BytecodeReader::read(&data).unwrap();

    assert_eq!(read_module.name, "entry_reader_test");
    assert_eq!(read_module.constants.len(), 1);
    assert_eq!(read_module.constants[0], BytecodeValue::Int(10));
    assert_eq!(read_module.functions.len(), 1);
    assert_eq!(read_module.functions[0].name, "start");
}

#[test]
fn test_compact_encoding_round_trip() {
    let mut ir_module = IrModule::new("compact_test");

    ir_module.add_constant(IrValue::Int(1));
    ir_module.add_constant(IrValue::Int(2));
    ir_module.add_constant(IrValue::Int(3));

    let func = IrFunction {
        name: "test_compact".to_string(),
        param_count: 0,
        local_count: 5,
        local_names: vec![],
        instructions: vec![
            OpCode::LoadConst(0),
            OpCode::LoadConst(1),
            OpCode::LoadConst(2),
            OpCode::Add,
            OpCode::StoreLocal(0),
            OpCode::LoadLocal(4),
            OpCode::Jump(10),
            OpCode::JumpIfFalse(20),
            OpCode::JumpIfTrue(30),
            OpCode::Call(3),
            OpCode::Return,
        ],
        is_entry: false,
        target: None,
    };

    ir_module.add_function(func);

    let data = BytecodeWriter::write(&ir_module).unwrap();
    let module = BytecodeReader::read(&data).unwrap();

    assert_eq!(module.functions.len(), 1);
    let func = &module.functions[0];
    assert_eq!(func.name, "test_compact");

    assert!(matches!(func.instructions[0], BytecodeInstruction::LoadConstInt { index: 0 }));
    assert!(matches!(func.instructions[1], BytecodeInstruction::LoadConstInt { index: 1 }));
    assert!(matches!(func.instructions[2], BytecodeInstruction::LoadConstInt { index: 2 }));
    assert!(matches!(func.instructions[3], BytecodeInstruction::Add));
    assert!(matches!(func.instructions[4], BytecodeInstruction::StoreLocal { index } if index == 0));
    assert!(matches!(func.instructions[5], BytecodeInstruction::LoadLocal { index } if index == 4));
    assert!(matches!(func.instructions[6], BytecodeInstruction::Jump { address } if address == 10));
    assert!(matches!(func.instructions[7], BytecodeInstruction::JumpIfFalse { address } if address == 20));
    assert!(matches!(func.instructions[8], BytecodeInstruction::JumpIfTrue { address } if address == 30));
    assert!(matches!(func.instructions[9], BytecodeInstruction::Call { arg_count } if arg_count == 3));
    assert!(matches!(func.instructions[10], BytecodeInstruction::Return));
}

#[test]
fn test_local_names_round_trip() {
    let mut ir_module = IrModule::new("local_names_test");
    ir_module.add_constant(IrValue::Int(42));

    let func = IrFunction {
        name: "func_with_names".to_string(),
        param_count: 2,
        local_count: 4,
        local_names: vec!["self".to_string(), "arg1".to_string(), "temp".to_string(), "result".to_string()],
        instructions: vec![OpCode::LoadConst(0), OpCode::StoreLocal(2), OpCode::Return],
        is_entry: false,
        target: None,
    };

    ir_module.add_function(func);

    let data = BytecodeWriter::write(&ir_module).unwrap();
    let module = BytecodeReader::read(&data).unwrap();

    assert_eq!(module.functions.len(), 1);
    let func = &module.functions[0];
    assert_eq!(func.name, "func_with_names");
    assert_eq!(func.local_names.len(), 4);
    assert_eq!(func.local_names[0], "self");
    assert_eq!(func.local_names[1], "arg1");
    assert_eq!(func.local_names[2], "temp");
    assert_eq!(func.local_names[3], "result");
}

#[test]
fn test_variable_names_round_trip() {
    let mut module = BytecodeModule::new("var_names_test");
    module.add_int_constant(10);
    module.add_string_constant("hello".to_string());

    module.add_function(gg_bytecode::BytecodeFunction {
        name: "main".to_string(),
        param_count: 0,
        local_count: 2,
        instructions: vec![
            BytecodeInstruction::LoadConstInt { index: 0 },
            BytecodeInstruction::StoreLocal { index: 0 },
            BytecodeInstruction::Return,
        ],
        local_names: vec!["x".to_string(), "y".to_string()],
    });

    let mut debug_info = DebugInfo::new();
    debug_info.add_entry(0, SourceLocation::with_offsets("test.gg", 1, 1, 0, 5));
    debug_info.add_entry(3, SourceLocation::with_offsets("test.gg", 2, 5, 6, 10));
    debug_info.add_function("main", "test.gg");
    debug_info.add_variable_names("main".to_string(), vec!["x".to_string(), "y".to_string()]);

    module.debug_info = Some(debug_info);

    let data = BytecodeWriter::write_module(&module).unwrap();
    let read_module = BytecodeReader::read(&data).unwrap();

    assert_eq!(read_module.name, "var_names_test");
    assert!(read_module.debug_info.is_some());

    let di = read_module.debug_info.as_ref().unwrap();

    let loc = di.lookup(0).unwrap();
    assert_eq!(loc.file, "test.gg");
    assert_eq!(loc.line, 1);
    assert_eq!(loc.column, 1);
    assert_eq!(loc.start_offset, 0);
    assert_eq!(loc.end_offset, 5);

    let loc2 = di.lookup(3).unwrap();
    assert_eq!(loc2.file, "test.gg");
    assert_eq!(loc2.line, 2);
    assert_eq!(loc2.column, 5);
    assert_eq!(loc2.start_offset, 6);
    assert_eq!(loc2.end_offset, 10);

    assert_eq!(di.lookup_function("main"), Some("test.gg"));

    let var_names = di.lookup_variable_names("main").unwrap();
    assert_eq!(var_names.len(), 2);
    assert_eq!(var_names[0], "x");
    assert_eq!(var_names[1], "y");
}

#[test]
fn test_tail_call_round_trip() {
    let mut ir_module = IrModule::new("tail_call_test");
    ir_module.add_constant(IrValue::Int(7));

    let func = IrFunction {
        name: "recursive".to_string(),
        param_count: 1,
        local_count: 1,
        local_names: vec!["n".to_string()],
        instructions: vec![
            OpCode::LoadLocal(0),
            OpCode::LoadConst(0),
            OpCode::Gt,
            OpCode::JumpIfFalse(10),
            OpCode::LoadLocal(0),
            OpCode::LoadConst(0),
            OpCode::Sub,
            OpCode::TailCall(1),
            OpCode::Jump(12),
            OpCode::LoadLocal(0),
            OpCode::Return,
        ],
        is_entry: false,
        target: None,
    };

    ir_module.add_function(func);

    let data = BytecodeWriter::write(&ir_module).unwrap();
    let module = BytecodeReader::read(&data).unwrap();

    assert_eq!(module.functions.len(), 1);
    let func = &module.functions[0];
    assert_eq!(func.name, "recursive");
    assert_eq!(func.local_names.len(), 1);
    assert_eq!(func.local_names[0], "n");

    assert!(matches!(func.instructions[7], BytecodeInstruction::TailCall(arity) if arity == 1));
}

#[test]
fn test_host_call_compact_round_trip() {
    let mut module = BytecodeModule::new("host_call_test");
    module.add_int_constant(42);
    let name_idx = module.add_string("print");

    module.add_function(gg_bytecode::BytecodeFunction {
        name: "test_host".to_string(),
        param_count: 0,
        local_count: 0,
        instructions: vec![
            BytecodeInstruction::LoadConstInt { index: 0 },
            BytecodeInstruction::HostCall { name_index: name_idx, arg_count: 1 },
            BytecodeInstruction::Return,
        ],
        local_names: vec![],
    });

    let data = BytecodeWriter::write_module(&module).unwrap();
    let read_module = BytecodeReader::read(&data).unwrap();

    assert_eq!(read_module.functions.len(), 1);
    let func = &read_module.functions[0];
    assert_eq!(func.instructions.len(), 3);

    assert!(
        matches!(func.instructions[1], BytecodeInstruction::HostCall { name_index, arg_count } if name_index == name_idx && arg_count == 1)
    );
}

#[test]
fn test_large_operand_compact_round_trip() {
    let mut module = BytecodeModule::new("large_operand_test");
    for i in 0..300 {
        module.add_int_constant(i as i64);
    }

    let large_index = 256u32;
    module.add_function(gg_bytecode::BytecodeFunction {
        name: "large_op".to_string(),
        param_count: 0,
        local_count: 0,
        instructions: vec![
            BytecodeInstruction::LoadConstInt { index: 0 },
            BytecodeInstruction::LoadConstInt { index: large_index },
            BytecodeInstruction::Return,
        ],
        local_names: vec![],
    });

    let data = BytecodeWriter::write_module(&module).unwrap();
    let read_module = BytecodeReader::read(&data).unwrap();

    assert_eq!(read_module.functions.len(), 1);
    let func = &read_module.functions[0];
    assert_eq!(func.instructions.len(), 3);

    assert!(matches!(func.instructions[0], BytecodeInstruction::LoadConstInt { index } if index == 0));
    assert!(matches!(func.instructions[1], BytecodeInstruction::LoadConstInt { index } if index == large_index));
}
