use std::io::Write;

use gg_core::{GError, GErrorKind, GResult};
use gg_ir::{IrModule, IrValue, OpCode};

use crate::{
    debug_info::DebugInfo,
    format::{
        BinaryWriter, BytecodeEntryPoint, BytecodeFunction, BytecodeInstruction, BytecodeModule, BytecodeValue, MAGIC, VERSION,
    },
};

/// 调试信息段魔数 "DEBU"
const DEBUG_MAGIC: u32 = 0x44454255;

/// 紧凑操作数短标记，表示后续跟 u8 操作数
const COMPACT_SHORT_MARKER: u8 = 0x00;

/// 紧凑操作数长标记，表示后续跟 u32_le 操作数
const COMPACT_LONG_MARKER: u8 = 0xFF;

/// 字节码写入器，将 IrModule 序列化为二进制格式
pub struct BytecodeWriter;

impl BytecodeWriter {
    /// 将 IrModule 序列化为二进制字节
    pub fn write(module: &IrModule) -> GResult<Vec<u8>> {
        let mut bytecode_module = BytecodeModule::new(&module.name);
        let mut writer = BinaryWriter::new();

        writer.write_u32(MAGIC);
        writer.write_u32(VERSION);
        writer.write_string(&module.name);

        Self::write_constants(&mut bytecode_module, &mut writer, &module.constants)?;

        for s in &module.string_pool {
            bytecode_module.add_string(s);
        }

        let compiled_functions = Self::compile_functions(&mut bytecode_module, &module.functions, &module.constants)?;

        Self::write_partitioned_constants(&mut writer, &bytecode_module)?;
        Self::write_string_pool(&mut writer, &bytecode_module.string_pool)?;
        Self::write_compiled_functions(&mut writer, &compiled_functions)?;
        Self::write_entry_points(&mut writer, &bytecode_module.entry_points)?;

        if let Some(ref debug_info) = bytecode_module.debug_info {
            Self::write_debug_info(&mut writer, debug_info)?;
        }

        Ok(writer.into_vec())
    }

    /// 将 BytecodeModule 序列化为二进制字节
    ///
    /// 与 `write` 不同，此方法直接序列化已构建的 BytecodeModule，
    /// 适用于需要手动构造模块并写入的场景（如包含调试信息的模块）。
    pub fn write_module(module: &BytecodeModule) -> GResult<Vec<u8>> {
        let mut writer = BinaryWriter::new();

        writer.write_u32(MAGIC);
        writer.write_u32(VERSION);
        writer.write_string(&module.name);

        Self::write_constants_from_module(&mut writer, &module.constants)?;

        Self::write_partitioned_constants_from_module(&mut writer, module)?;
        Self::write_string_pool(&mut writer, &module.string_pool)?;
        Self::write_compiled_functions(&mut writer, &module.functions)?;
        Self::write_entry_points(&mut writer, &module.entry_points)?;

        if let Some(ref debug_info) = module.debug_info {
            Self::write_debug_info(&mut writer, debug_info)?;
        }

        Ok(writer.into_vec())
    }

    /// 从 BytecodeValue 列表写入常量池
    fn write_constants_from_module(writer: &mut BinaryWriter, constants: &[BytecodeValue]) -> GResult<()> {
        writer.write_u32(constants.len() as u32);

        for value in constants {
            match value {
                BytecodeValue::Int(v) => {
                    writer.write_u8(0);
                    writer.write_i64(*v);
                }
                BytecodeValue::Float(v) => {
                    writer.write_u8(1);
                    writer.write_f64(*v);
                }
                BytecodeValue::Bool(v) => {
                    writer.write_u8(2);
                    writer.write_u8(if *v { 1 } else { 0 });
                }
                BytecodeValue::String(v) => {
                    writer.write_u8(3);
                    writer.write_string(v);
                }
                BytecodeValue::Entity(v) => {
                    writer.write_u8(4);
                    writer.write_u64(*v);
                }
                BytecodeValue::Null => {
                    writer.write_u8(5);
                }
                BytecodeValue::List(_) | BytecodeValue::Object(_) | BytecodeValue::Map(_) => {
                    return Err(GError {
                        kind: GErrorKind::Runtime,
                        message: format!("复合类型不能序列化到常量池: {:?}", value),
                    });
                }
            }
        }

        Ok(())
    }

    /// 从 BytecodeModule 写入分区常量池
    fn write_partitioned_constants_from_module(writer: &mut BinaryWriter, module: &BytecodeModule) -> GResult<()> {
        writer.write_u32(module.int_constants.len() as u32);
        for v in &module.int_constants {
            writer.write_i64(*v);
        }

        writer.write_u32(module.float_constants.len() as u32);
        for v in &module.float_constants {
            writer.write_f64(*v);
        }

        writer.write_u32(module.string_constants.len() as u32);
        for s in &module.string_constants {
            writer.write_string(s);
        }

        writer.write_u32(module.entity_constants.len() as u32);
        for v in &module.entity_constants {
            writer.write_u64(*v);
        }

        writer.write_u32(module.bool_constants.len() as u32);
        for v in &module.bool_constants {
            writer.write_u8(if *v { 1 } else { 0 });
        }

        Ok(())
    }

    /// 写入常量池
    fn write_constants(bytecode_module: &mut BytecodeModule, writer: &mut BinaryWriter, constants: &[IrValue]) -> GResult<()> {
        writer.write_u32(constants.len() as u32);

        for value in constants {
            let bc_value = Self::ir_value_to_bytecode(value);
            bytecode_module.add_constant(bc_value.clone());

            match &bc_value {
                BytecodeValue::Int(v) => {
                    writer.write_u8(0);
                    writer.write_i64(*v);
                }
                BytecodeValue::Float(v) => {
                    writer.write_u8(1);
                    writer.write_f64(*v);
                }
                BytecodeValue::Bool(v) => {
                    writer.write_u8(2);
                    writer.write_u8(if *v { 1 } else { 0 });
                }
                BytecodeValue::String(v) => {
                    writer.write_u8(3);
                    writer.write_string(v);
                }
                BytecodeValue::Entity(v) => {
                    writer.write_u8(4);
                    writer.write_u64(*v);
                }
                BytecodeValue::Null => {
                    writer.write_u8(5);
                }
                BytecodeValue::List(_) | BytecodeValue::Object(_) | BytecodeValue::Map(_) => {
                    return Err(GError {
                        kind: GErrorKind::Runtime,
                        message: format!("复合类型不能序列化到常量池: {:?}", bc_value),
                    });
                }
            }
        }

        Ok(())
    }

    fn write_partitioned_constants(writer: &mut BinaryWriter, bytecode_module: &BytecodeModule) -> GResult<()> {
        writer.write_u32(bytecode_module.int_constants.len() as u32);
        for v in &bytecode_module.int_constants {
            writer.write_i64(*v);
        }

        writer.write_u32(bytecode_module.float_constants.len() as u32);
        for v in &bytecode_module.float_constants {
            writer.write_f64(*v);
        }

        writer.write_u32(bytecode_module.string_constants.len() as u32);
        for s in &bytecode_module.string_constants {
            writer.write_string(s);
        }

        writer.write_u32(bytecode_module.entity_constants.len() as u32);
        for v in &bytecode_module.entity_constants {
            writer.write_u64(*v);
        }

        writer.write_u32(bytecode_module.bool_constants.len() as u32);
        for v in &bytecode_module.bool_constants {
            writer.write_u8(if *v { 1 } else { 0 });
        }

        Ok(())
    }

    /// 写入字符串池
    fn write_string_pool(writer: &mut BinaryWriter, string_pool: &[String]) -> GResult<()> {
        writer.write_u32(string_pool.len() as u32);
        for s in string_pool {
            writer.write_string(s);
        }
        Ok(())
    }

    /// 将 IR 函数列表编译为字节码函数列表（同时填充字符串池）
    fn compile_functions(
        bytecode_module: &mut BytecodeModule,
        functions: &[gg_ir::IrFunction],
        constants: &[IrValue],
    ) -> GResult<Vec<BytecodeFunction>> {
        let mut compiled = Vec::new();

        for func in functions {
            let mut instructions = Vec::new();
            for op in &func.instructions {
                let bc_inst = Self::ir_op_to_instruction(bytecode_module, op, constants)?;
                instructions.push(bc_inst);
            }

            compiled.push(BytecodeFunction {
                name: func.name.clone(),
                param_count: func.param_count as u32,
                local_count: func.local_count as u32,
                instructions,
                local_names: func.local_names.clone(),
            });
        }

        Ok(compiled)
    }

    /// 写入已编译的字节码函数列表
    fn write_compiled_functions(writer: &mut BinaryWriter, functions: &[BytecodeFunction]) -> GResult<()> {
        writer.write_u32(functions.len() as u32);

        for func in functions {
            writer.write_string(&func.name);
            writer.write_u32(func.param_count);
            writer.write_u32(func.local_count);

            writer.write_u32(func.instructions.len() as u32);
            for inst in &func.instructions {
                Self::write_instruction(writer, inst)?;
            }

            writer.write_u32(func.local_names.len() as u32);
            for name in &func.local_names {
                writer.write_string(name);
            }
        }

        Ok(())
    }

    /// 写入入口点列表
    fn write_entry_points(writer: &mut BinaryWriter, entry_points: &[BytecodeEntryPoint]) -> GResult<()> {
        writer.write_u32(entry_points.len() as u32);

        for entry_point in entry_points {
            writer.write_string(&entry_point.name);
            writer.write_string(&entry_point.target);
            writer.write_u32(entry_point.priority);
        }

        Ok(())
    }

    /// 写入调试信息段
    fn write_debug_info(writer: &mut BinaryWriter, debug_info: &DebugInfo) -> GResult<()> {
        writer.write_u32(DEBUG_MAGIC);

        let entries = debug_info.entries();
        writer.write_u32(entries.len() as u32);
        for entry in entries {
            writer.write_u32(entry.offset);
            writer.write_string(&entry.location.file);
            writer.write_u32(entry.location.line);
            writer.write_u32(entry.location.column);
            writer.write_u32(entry.location.start_offset);
            writer.write_u32(entry.location.end_offset);
        }

        let function_map = debug_info.function_map();
        writer.write_u32(function_map.len() as u32);
        for (function_name, source_file) in function_map {
            writer.write_string(function_name);
            writer.write_string(source_file);
        }

        let variable_names = debug_info.variable_names();
        writer.write_u32(variable_names.len() as u32);
        for (function_name, names) in variable_names {
            writer.write_string(function_name);
            writer.write_u32(names.len() as u32);
            for name in names {
                writer.write_string(name);
            }
        }

        Ok(())
    }

    /// 写入紧凑操作数到输出流
    ///
    /// 如果操作数 <= 255，写入短标记 0x00 + u8（共 2 字节）；
    /// 如果操作数 > 255，写入长标记 0xFF + u32_le（共 5 字节）。
    fn write_compact_operand(writer: &mut impl Write, operand: u32) -> std::io::Result<()> {
        if operand <= 255 {
            writer.write_all(&[COMPACT_SHORT_MARKER, operand as u8])?;
        }
        else {
            writer.write_all(&[COMPACT_LONG_MARKER])?;
            writer.write_all(&operand.to_le_bytes())?;
        }
        Ok(())
    }

    /// 使用紧凑编码写入单条指令到输出流
    ///
    /// 对于有操作数的指令，使用可变长度的紧凑编码替代固定 u32 编码，
    /// 减少小操作数指令的序列化体积。
    fn write_instruction_compact(writer: &mut impl Write, instruction: &BytecodeInstruction) -> std::io::Result<()> {
        writer.write_all(&[instruction.opcode().as_byte()])?;

        match instruction {
            BytecodeInstruction::LoadConst { index } => {
                Self::write_compact_operand(writer, *index)?;
            }
            BytecodeInstruction::LoadConstInt { index } => {
                Self::write_compact_operand(writer, *index)?;
            }
            BytecodeInstruction::LoadConstFloat { index } => {
                Self::write_compact_operand(writer, *index)?;
            }
            BytecodeInstruction::LoadConstString { index } => {
                Self::write_compact_operand(writer, *index)?;
            }
            BytecodeInstruction::LoadConstEntity { index } => {
                Self::write_compact_operand(writer, *index)?;
            }
            BytecodeInstruction::LoadConstBool { index } => {
                Self::write_compact_operand(writer, *index)?;
            }
            BytecodeInstruction::LoadLocal { index } => {
                Self::write_compact_operand(writer, *index)?;
            }
            BytecodeInstruction::StoreLocal { index } => {
                Self::write_compact_operand(writer, *index)?;
            }
            BytecodeInstruction::Jump { address } => {
                Self::write_compact_operand(writer, *address)?;
            }
            BytecodeInstruction::JumpIfFalse { address } => {
                Self::write_compact_operand(writer, *address)?;
            }
            BytecodeInstruction::JumpIfTrue { address } => {
                Self::write_compact_operand(writer, *address)?;
            }
            BytecodeInstruction::Call { arg_count } => {
                Self::write_compact_operand(writer, *arg_count)?;
            }
            BytecodeInstruction::TailCall(arity) => {
                Self::write_compact_operand(writer, *arity)?;
            }
            BytecodeInstruction::AddComponent { type_name_index } => {
                Self::write_compact_operand(writer, *type_name_index)?;
            }
            BytecodeInstruction::GetComponent { type_name_index } => {
                Self::write_compact_operand(writer, *type_name_index)?;
            }
            BytecodeInstruction::SetComponent { type_name_index } => {
                Self::write_compact_operand(writer, *type_name_index)?;
            }
            BytecodeInstruction::HostCall { name_index, arg_count } => {
                Self::write_compact_operand(writer, *name_index)?;
                Self::write_compact_operand(writer, *arg_count)?;
            }
            BytecodeInstruction::GetField { name_index } => {
                Self::write_compact_operand(writer, *name_index)?;
            }
            BytecodeInstruction::SetField { name_index } => {
                Self::write_compact_operand(writer, *name_index)?;
            }
            BytecodeInstruction::NewObject { field_count } => {
                Self::write_compact_operand(writer, *field_count)?;
            }
            BytecodeInstruction::NewList { element_count } => {
                Self::write_compact_operand(writer, *element_count)?;
            }
            BytecodeInstruction::NewMap { pair_count } => {
                Self::write_compact_operand(writer, *pair_count)?;
            }
            BytecodeInstruction::StringConcat { count } => {
                Self::write_compact_operand(writer, *count)?;
            }
            BytecodeInstruction::LoadNull
            | BytecodeInstruction::LoadTrue
            | BytecodeInstruction::LoadFalse
            | BytecodeInstruction::Add
            | BytecodeInstruction::Sub
            | BytecodeInstruction::Mul
            | BytecodeInstruction::Div
            | BytecodeInstruction::Mod
            | BytecodeInstruction::Neg
            | BytecodeInstruction::Eq
            | BytecodeInstruction::Ne
            | BytecodeInstruction::Lt
            | BytecodeInstruction::Le
            | BytecodeInstruction::Gt
            | BytecodeInstruction::Ge
            | BytecodeInstruction::And
            | BytecodeInstruction::Or
            | BytecodeInstruction::Not
            | BytecodeInstruction::Return
            | BytecodeInstruction::SpawnEntity
            | BytecodeInstruction::DespawnEntity
            | BytecodeInstruction::Pop
            | BytecodeInstruction::Dup
            | BytecodeInstruction::GetIndex
            | BytecodeInstruction::SetIndex => {}
        }

        Ok(())
    }

    /// 写入单条指令（使用紧凑编码）
    fn write_instruction(writer: &mut BinaryWriter, inst: &BytecodeInstruction) -> GResult<()> {
        Self::write_instruction_compact(writer, inst)
            .map_err(|e| GError { kind: GErrorKind::Runtime, message: format!("写入指令失败: {}", e) })
    }

    /// 将 IrValue 转换为 BytecodeValue
    fn ir_value_to_bytecode(value: &IrValue) -> BytecodeValue {
        match value {
            IrValue::Int(v) => BytecodeValue::Int(*v),
            IrValue::Float(v) => BytecodeValue::Float(*v),
            IrValue::Bool(v) => BytecodeValue::Bool(*v),
            IrValue::String(v) => BytecodeValue::String(v.clone()),
            IrValue::Entity(v) => BytecodeValue::Entity(*v),
            IrValue::Null => BytecodeValue::Null,
        }
    }

    /// 将 IR 操作码转换为字节码指令
    fn ir_op_to_instruction(
        bytecode_module: &mut BytecodeModule,
        op: &OpCode,
        constants: &[IrValue],
    ) -> GResult<BytecodeInstruction> {
        match op {
            OpCode::LoadConst(idx) => match constants.get(*idx) {
                Some(IrValue::Int(v)) => {
                    let bc_idx = bytecode_module.add_int_constant(*v);
                    Ok(BytecodeInstruction::LoadConstInt { index: bc_idx })
                }
                Some(IrValue::Float(v)) => {
                    let bc_idx = bytecode_module.add_float_constant(*v);
                    Ok(BytecodeInstruction::LoadConstFloat { index: bc_idx })
                }
                Some(IrValue::String(v)) => {
                    let bc_idx = bytecode_module.add_string_constant(v.clone());
                    Ok(BytecodeInstruction::LoadConstString { index: bc_idx })
                }
                Some(IrValue::Entity(v)) => {
                    let bc_idx = bytecode_module.add_entity_constant(*v);
                    Ok(BytecodeInstruction::LoadConstEntity { index: bc_idx })
                }
                Some(IrValue::Bool(v)) => {
                    let bc_idx = bytecode_module.add_bool_constant(*v);
                    Ok(BytecodeInstruction::LoadConstBool { index: bc_idx })
                }
                Some(IrValue::Null) => Ok(BytecodeInstruction::LoadNull),
                None => {
                    Err(gg_core::GError { kind: gg_core::GErrorKind::Runtime, message: format!("常量索引越界: {}", idx) })
                }
            },
            OpCode::LoadNull => Ok(BytecodeInstruction::LoadNull),
            OpCode::LoadTrue => Ok(BytecodeInstruction::LoadTrue),
            OpCode::LoadFalse => Ok(BytecodeInstruction::LoadFalse),
            OpCode::LoadLocal(idx) => Ok(BytecodeInstruction::LoadLocal { index: *idx as u32 }),
            OpCode::StoreLocal(idx) => Ok(BytecodeInstruction::StoreLocal { index: *idx as u32 }),
            OpCode::Add => Ok(BytecodeInstruction::Add),
            OpCode::Sub => Ok(BytecodeInstruction::Sub),
            OpCode::Mul => Ok(BytecodeInstruction::Mul),
            OpCode::Div => Ok(BytecodeInstruction::Div),
            OpCode::Mod => Ok(BytecodeInstruction::Mod),
            OpCode::Neg => Ok(BytecodeInstruction::Neg),
            OpCode::Eq => Ok(BytecodeInstruction::Eq),
            OpCode::Ne => Ok(BytecodeInstruction::Ne),
            OpCode::Lt => Ok(BytecodeInstruction::Lt),
            OpCode::Le => Ok(BytecodeInstruction::Le),
            OpCode::Gt => Ok(BytecodeInstruction::Gt),
            OpCode::Ge => Ok(BytecodeInstruction::Ge),
            OpCode::And => Ok(BytecodeInstruction::And),
            OpCode::Or => Ok(BytecodeInstruction::Or),
            OpCode::Not => Ok(BytecodeInstruction::Not),
            OpCode::Jump(addr) => Ok(BytecodeInstruction::Jump { address: *addr as u32 }),
            OpCode::JumpIfFalse(addr) => Ok(BytecodeInstruction::JumpIfFalse { address: *addr as u32 }),
            OpCode::JumpIfTrue(addr) => Ok(BytecodeInstruction::JumpIfTrue { address: *addr as u32 }),
            OpCode::Call(arg_count) => Ok(BytecodeInstruction::Call { arg_count: *arg_count as u32 }),
            OpCode::TailCall(arg_count) => Ok(BytecodeInstruction::TailCall(*arg_count as u32)),
            OpCode::Return => Ok(BytecodeInstruction::Return),
            OpCode::SpawnEntity => Ok(BytecodeInstruction::SpawnEntity),
            OpCode::DespawnEntity => Ok(BytecodeInstruction::DespawnEntity),
            OpCode::AddComponent(string_idx) => Ok(BytecodeInstruction::AddComponent { type_name_index: *string_idx as u32 }),
            OpCode::GetComponent(string_idx) => Ok(BytecodeInstruction::GetComponent { type_name_index: *string_idx as u32 }),
            OpCode::SetComponent(string_idx) => Ok(BytecodeInstruction::SetComponent { type_name_index: *string_idx as u32 }),
            OpCode::HostCall(name_idx, arg_count) => {
                Ok(BytecodeInstruction::HostCall { name_index: *name_idx as u32, arg_count: *arg_count as u32 })
            }
            OpCode::Pop => Ok(BytecodeInstruction::Pop),
            OpCode::Dup => Ok(BytecodeInstruction::Dup),
            OpCode::GetField(name_idx) => Ok(BytecodeInstruction::GetField { name_index: *name_idx as u32 }),
            OpCode::SetField(name_idx) => Ok(BytecodeInstruction::SetField { name_index: *name_idx as u32 }),
            OpCode::GetIndex => Ok(BytecodeInstruction::GetIndex),
            OpCode::SetIndex => Ok(BytecodeInstruction::SetIndex),
            OpCode::NewObject(field_count) => Ok(BytecodeInstruction::NewObject { field_count: *field_count as u32 }),
            OpCode::NewList(element_count) => Ok(BytecodeInstruction::NewList { element_count: *element_count as u32 }),
            OpCode::NewMap(pair_count) => Ok(BytecodeInstruction::NewMap { pair_count: *pair_count as u32 }),
            OpCode::StringConcat(count) => Ok(BytecodeInstruction::StringConcat { count: *count as u32 }),
        }
    }
}
