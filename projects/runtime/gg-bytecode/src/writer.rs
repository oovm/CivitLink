use gg_core::GResult;
use gg_ir::{IrModule, IrValue, OpCode};

use crate::{
    debug_info::DebugInfo,
    format::{BinaryWriter, BytecodeFunction, BytecodeInstruction, BytecodeModule, BytecodeValue, MAGIC, VERSION},
};

/// 调试信息段魔数 "DEBU"
const DEBUG_MAGIC: u32 = 0x44454255;

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

        let compiled_functions = Self::compile_functions(&mut bytecode_module, &module.functions)?;

        Self::write_string_pool(&mut writer, &bytecode_module.string_pool)?;
        Self::write_compiled_functions(&mut writer, &compiled_functions)?;

        if let Some(ref debug_info) = bytecode_module.debug_info {
            Self::write_debug_info(&mut writer, debug_info)?;
        }

        Ok(writer.into_vec())
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
            }
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
    ) -> GResult<Vec<BytecodeFunction>> {
        let mut compiled = Vec::new();

        for func in functions {
            let mut instructions = Vec::new();
            for op in &func.instructions {
                let bc_inst = Self::ir_op_to_instruction(bytecode_module, op)?;
                instructions.push(bc_inst);
            }

            compiled.push(BytecodeFunction {
                name: func.name.clone(),
                param_count: func.param_count as u32,
                local_count: func.local_count as u32,
                instructions,
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
        }

        Ok(())
    }

    /// 写入调试信息段
    fn write_debug_info(writer: &mut BinaryWriter, debug_info: &DebugInfo) -> GResult<()> {
        writer.write_u32(DEBUG_MAGIC);

        let entries = debug_info.entries();
        writer.write_u32(entries.len() as u32);
        for (&offset, location) in entries {
            writer.write_u32(offset);
            writer.write_string(&location.file);
            writer.write_u32(location.line);
            writer.write_u32(location.column);
        }

        let function_map = debug_info.function_map();
        writer.write_u32(function_map.len() as u32);
        for (function_name, source_file) in function_map {
            writer.write_string(function_name);
            writer.write_string(source_file);
        }

        Ok(())
    }

    /// 写入单条指令
    fn write_instruction(writer: &mut BinaryWriter, inst: &BytecodeInstruction) -> GResult<()> {
        writer.write_u8(inst.opcode().as_byte());

        match inst {
            BytecodeInstruction::LoadConst { index } => {
                writer.write_u32(*index);
            }
            BytecodeInstruction::LoadLocal { index } => {
                writer.write_u32(*index);
            }
            BytecodeInstruction::StoreLocal { index } => {
                writer.write_u32(*index);
            }
            BytecodeInstruction::Jump { address } => {
                writer.write_u32(*address);
            }
            BytecodeInstruction::JumpIfFalse { address } => {
                writer.write_u32(*address);
            }
            BytecodeInstruction::JumpIfTrue { address } => {
                writer.write_u32(*address);
            }
            BytecodeInstruction::Call { arg_count } => {
                writer.write_u32(*arg_count);
            }
            BytecodeInstruction::AddComponent { type_name_index } => {
                writer.write_u32(*type_name_index);
            }
            BytecodeInstruction::GetComponent { type_name_index } => {
                writer.write_u32(*type_name_index);
            }
            BytecodeInstruction::SetComponent { type_name_index } => {
                writer.write_u32(*type_name_index);
            }
            BytecodeInstruction::HostCall { name_index, arg_count } => {
                writer.write_u32(*name_index);
                writer.write_u32(*arg_count);
            }
            BytecodeInstruction::GetField { name_index } => {
                writer.write_u32(*name_index);
            }
            BytecodeInstruction::SetField { name_index } => {
                writer.write_u32(*name_index);
            }
            BytecodeInstruction::NewObject { field_count } => {
                writer.write_u32(*field_count);
            }
            BytecodeInstruction::NewList { element_count } => {
                writer.write_u32(*element_count);
            }
            BytecodeInstruction::NewMap { pair_count } => {
                writer.write_u32(*pair_count);
            }
            BytecodeInstruction::StringConcat { count } => {
                writer.write_u32(*count);
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
    fn ir_op_to_instruction(bytecode_module: &mut BytecodeModule, op: &OpCode) -> GResult<BytecodeInstruction> {
        match op {
            OpCode::LoadConst(idx) => Ok(BytecodeInstruction::LoadConst { index: *idx as u32 }),
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
            OpCode::Return => Ok(BytecodeInstruction::Return),
            OpCode::SpawnEntity => Ok(BytecodeInstruction::SpawnEntity),
            OpCode::DespawnEntity => Ok(BytecodeInstruction::DespawnEntity),
            OpCode::AddComponent(type_name) => {
                let idx = bytecode_module.add_string(type_name);
                Ok(BytecodeInstruction::AddComponent { type_name_index: idx })
            }
            OpCode::GetComponent(type_name) => {
                let idx = bytecode_module.add_string(type_name);
                Ok(BytecodeInstruction::GetComponent { type_name_index: idx })
            }
            OpCode::SetComponent(type_name) => {
                let idx = bytecode_module.add_string(type_name);
                Ok(BytecodeInstruction::SetComponent { type_name_index: idx })
            }
            OpCode::HostCall(name, arg_count) => {
                let idx = bytecode_module.add_string(name);
                Ok(BytecodeInstruction::HostCall { name_index: idx, arg_count: *arg_count as u32 })
            }
            OpCode::Pop => Ok(BytecodeInstruction::Pop),
            OpCode::Dup => Ok(BytecodeInstruction::Dup),
            OpCode::GetField(name) => {
                let idx = bytecode_module.add_string(name);
                Ok(BytecodeInstruction::GetField { name_index: idx })
            }
            OpCode::SetField(name) => {
                let idx = bytecode_module.add_string(name);
                Ok(BytecodeInstruction::SetField { name_index: idx })
            }
            OpCode::GetIndex => Ok(BytecodeInstruction::GetIndex),
            OpCode::SetIndex => Ok(BytecodeInstruction::SetIndex),
            OpCode::NewObject(field_count) => Ok(BytecodeInstruction::NewObject { field_count: *field_count as u32 }),
            OpCode::NewList(element_count) => Ok(BytecodeInstruction::NewList { element_count: *element_count as u32 }),
            OpCode::NewMap(pair_count) => Ok(BytecodeInstruction::NewMap { pair_count: *pair_count as u32 }),
            OpCode::StringConcat(count) => Ok(BytecodeInstruction::StringConcat { count: *count as u32 }),
        }
    }
}
