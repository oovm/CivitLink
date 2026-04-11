use gg_bytecode::{
    format::{BytecodeValue, MAGIC},
    reader::BytecodeReader,
    writer::BytecodeWriter,
};
use gg_ir::{IrFunction, IrModule, IrValue, OpCode};

fn create_test_module() -> IrModule {
    let mut module = IrModule::new("test_module");
    module.add_constant(IrValue::Int(42));
    module.add_constant(IrValue::Bool(true));
    module.add_constant(IrValue::String("hello".to_string()));

    let func = IrFunction {
        name: "main".to_string(),
        param_count: 0,
        local_count: 1,
        local_names: vec![],
        instructions: vec![OpCode::LoadConst(0), OpCode::Return],
        is_entry: false,
        target: None,
    };

    module.add_function(func);
    module
}

#[test]
fn test_write_produces_valid_magic_number() {
    let module = create_test_module();
    let data = BytecodeWriter::write(&module).unwrap();
    assert!(data.len() >= 4);
    let magic = u32::from_le_bytes([data[0], data[1], data[2], data[3]]);
    assert_eq!(magic, MAGIC);
}

#[test]
fn test_round_trip_write_read() {
    let module = create_test_module();
    let data = BytecodeWriter::write(&module).unwrap();
    let read_module = BytecodeReader::read(&data).unwrap();

    assert_eq!(read_module.name, "test_module");
    assert_eq!(read_module.constants.len(), 3);
    assert_eq!(read_module.constants[0], BytecodeValue::Int(42));
    assert_eq!(read_module.constants[1], BytecodeValue::Bool(true));
    assert_eq!(read_module.constants[2], BytecodeValue::String("hello".to_string()));
    assert_eq!(read_module.functions.len(), 1);
    assert_eq!(read_module.functions[0].name, "main");
    assert_eq!(read_module.functions[0].param_count, 0);
    assert_eq!(read_module.functions[0].local_count, 1);
    assert_eq!(read_module.functions[0].instructions.len(), 2);
    assert_eq!(read_module.entry_points.len(), 0);
}

#[test]
fn test_entry_points_round_trip() {
    let mut ir_module = IrModule::new("entry_test");
    ir_module.add_constant(IrValue::Int(1));

    let func = IrFunction {
        name: "init".to_string(),
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

    assert_eq!(read_module.name, "entry_test");
    assert_eq!(read_module.constants.len(), 1);
    assert_eq!(read_module.constants[0], BytecodeValue::Int(1));
    assert_eq!(read_module.functions.len(), 1);
    assert_eq!(read_module.functions[0].name, "init");
}
