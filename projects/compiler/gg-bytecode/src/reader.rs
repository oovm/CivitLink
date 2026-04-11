use std::{collections::HashMap, io::Read};

use gg_core::{GError, GErrorKind, GResult};

use crate::{
    debug_info::{DebugInfo, SourceLocation},
    format::{
        BinaryReader, BytecodeEntryPoint, BytecodeFunction, BytecodeInstruction, BytecodeModule, BytecodeOpCode, BytecodeValue,
        MAGIC, VERSION,
    },
};

/// 调试信息段魔数 "DEBU"
const DEBUG_MAGIC: u32 = 0x44454255;

/// 紧凑操作数短标记，表示后续跟 u8 操作数
const COMPACT_SHORT_MARKER: u8 = 0x00;

/// 紧凑操作数长标记，表示后续跟 u32_le 操作数
const COMPACT_LONG_MARKER: u8 = 0xFF;

/// 字节码读取器，从二进制数据反序列化为 BytecodeModule
pub struct BytecodeReader;

impl BytecodeReader {
    /// 从二进制数据反序列化为 BytecodeModule
    pub fn read(data: &[u8]) -> GResult<BytecodeModule> {
        let mut reader = BinaryReader::new(data);

        let magic = reader.read_u32()?;
        if magic != MAGIC {
            return Err(GError {
                kind: GErrorKind::Runtime,
                message: format!("无效的字节码魔数: 期望 0x{:08X}, 实际 0x{:08X}", MAGIC, magic),
            });
        }

        let version = reader.read_u32()?;
        if version > VERSION {
            return Err(GError {
                kind: GErrorKind::Runtime,
                message: format!("不支持的字节码版本: 期望 <= {}, 实际 {}", VERSION, version),
            });
        }

        let name = reader.read_string()?;

        let constants = Self::read_constants(&mut reader)?;

        let (int_constants, float_constants, string_constants, entity_constants, bool_constants) = if version >= 2 {
            Self::read_partitioned_constants(&mut reader)?
        }
        else {
            Self::convert_to_partitioned(&constants)
        };

        let string_pool = Self::read_string_pool(&mut reader)?;
        let functions = Self::read_functions(&mut reader, version)?;
        let entry_points = Self::read_entry_points(&mut reader)?;
        let debug_info = Self::try_read_debug_info(&mut reader, version)?;

        Ok(BytecodeModule {
            name,
            version,
            constants,
            int_constants,
            float_constants,
            string_constants,
            entity_constants,
            bool_constants,
            string_pool,
            functions,
            function_index: HashMap::new(),
            debug_info,
            entry_points,
        })
    }

    /// 读取常量池
    fn read_constants(reader: &mut BinaryReader) -> GResult<Vec<BytecodeValue>> {
        let count = reader.read_u32()?;
        let mut constants = Vec::with_capacity(count as usize);

        for _ in 0..count {
            let tag = reader.read_u8()?;
            let value = match tag {
                0 => BytecodeValue::Int(reader.read_i64()?),
                1 => BytecodeValue::Float(reader.read_f64()?),
                2 => {
                    let v = reader.read_u8()?;
                    BytecodeValue::Bool(v != 0)
                }
                3 => BytecodeValue::String(reader.read_string()?),
                4 => BytecodeValue::Entity(reader.read_u64()?),
                5 => BytecodeValue::Null,
                _ => {
                    return Err(GError { kind: GErrorKind::Runtime, message: format!("未知的常量类型标签: {}", tag) });
                }
            };
            constants.push(value);
        }

        Ok(constants)
    }

    fn read_partitioned_constants(
        reader: &mut BinaryReader,
    ) -> GResult<(Vec<i64>, Vec<f64>, Vec<String>, Vec<u64>, Vec<bool>)> {
        let int_count = reader.read_u32()?;
        let mut int_constants = Vec::with_capacity(int_count as usize);
        for _ in 0..int_count {
            int_constants.push(reader.read_i64()?);
        }

        let float_count = reader.read_u32()?;
        let mut float_constants = Vec::with_capacity(float_count as usize);
        for _ in 0..float_count {
            float_constants.push(reader.read_f64()?);
        }

        let string_count = reader.read_u32()?;
        let mut string_constants = Vec::with_capacity(string_count as usize);
        for _ in 0..string_count {
            string_constants.push(reader.read_string()?);
        }

        let entity_count = reader.read_u32()?;
        let mut entity_constants = Vec::with_capacity(entity_count as usize);
        for _ in 0..entity_count {
            entity_constants.push(reader.read_u64()?);
        }

        let bool_count = reader.read_u32()?;
        let mut bool_constants = Vec::with_capacity(bool_count as usize);
        for _ in 0..bool_count {
            let v = reader.read_u8()?;
            bool_constants.push(v != 0);
        }

        Ok((int_constants, float_constants, string_constants, entity_constants, bool_constants))
    }

    fn convert_to_partitioned(constants: &[BytecodeValue]) -> (Vec<i64>, Vec<f64>, Vec<String>, Vec<u64>, Vec<bool>) {
        let mut int_constants = Vec::new();
        let mut float_constants = Vec::new();
        let mut string_constants = Vec::new();
        let mut entity_constants = Vec::new();
        let mut bool_constants = Vec::new();

        for value in constants {
            match value {
                BytecodeValue::Int(v) => int_constants.push(*v),
                BytecodeValue::Float(v) => float_constants.push(*v),
                BytecodeValue::String(v) => string_constants.push(v.clone()),
                BytecodeValue::Entity(v) => entity_constants.push(*v),
                BytecodeValue::Bool(v) => bool_constants.push(*v),
                _ => {}
            }
        }

        (int_constants, float_constants, string_constants, entity_constants, bool_constants)
    }

    /// 读取字符串池
    fn read_string_pool(reader: &mut BinaryReader) -> GResult<Vec<String>> {
        let count = reader.read_u32()?;
        let mut pool = Vec::with_capacity(count as usize);

        for _ in 0..count {
            pool.push(reader.read_string()?);
        }

        Ok(pool)
    }

    /// 读取函数列表
    fn read_functions(reader: &mut BinaryReader, version: u32) -> GResult<Vec<BytecodeFunction>> {
        let count = reader.read_u32()?;
        let mut functions = Vec::with_capacity(count as usize);

        for _ in 0..count {
            let name = reader.read_string()?;
            let param_count = reader.read_u32()?;
            let local_count = reader.read_u32()?;

            let inst_count = reader.read_u32()?;
            let mut instructions = Vec::with_capacity(inst_count as usize);

            for _ in 0..inst_count {
                instructions.push(Self::read_instruction(reader, version)?);
            }

            let local_names = if version >= 3 {
                let ln_count = reader.read_u32()?;
                let mut names = Vec::with_capacity(ln_count as usize);
                for _ in 0..ln_count {
                    names.push(reader.read_string()?);
                }
                names
            }
            else {
                Vec::new()
            };

            functions.push(BytecodeFunction { name, param_count, local_count, instructions, local_names });
        }

        Ok(functions)
    }

    /// 读取入口点列表
    fn read_entry_points(reader: &mut BinaryReader) -> GResult<Vec<BytecodeEntryPoint>> {
        let count = reader.read_u32()?;
        let mut entry_points = Vec::with_capacity(count as usize);

        for _ in 0..count {
            let name = reader.read_string()?;
            let target = reader.read_string()?;
            let priority = reader.read_u32()?;

            entry_points.push(BytecodeEntryPoint { name, target, priority });
        }

        Ok(entry_points)
    }

    /// 尝试读取调试信息段（向后兼容）
    fn try_read_debug_info(reader: &mut BinaryReader, version: u32) -> GResult<Option<DebugInfo>> {
        if reader.remaining() < 4 {
            return Ok(None);
        }

        let magic = reader.read_u32()?;
        if magic != DEBUG_MAGIC {
            return Ok(None);
        }

        Ok(Some(Self::read_debug_info(reader, version)?))
    }

    /// 读取调试信息段
    fn read_debug_info(reader: &mut BinaryReader, version: u32) -> GResult<DebugInfo> {
        let mut debug_info = DebugInfo::new();

        let entry_count = reader.read_u32()?;
        for _ in 0..entry_count {
            let offset = reader.read_u32()?;
            let file = reader.read_string()?;
            let line = reader.read_u32()?;
            let column = reader.read_u32()?;

            if version >= 3 {
                let start_offset = reader.read_u32()?;
                let end_offset = reader.read_u32()?;
                debug_info.add_entry(offset, SourceLocation::with_offsets(file, line, column, start_offset, end_offset));
            }
            else {
                debug_info.add_entry(offset, SourceLocation::new(file, line, column));
            }
        }

        let func_count = reader.read_u32()?;
        for _ in 0..func_count {
            let function_name = reader.read_string()?;
            let source_file = reader.read_string()?;
            debug_info.add_function(function_name, source_file);
        }

        if version >= 3 {
            let var_count = reader.read_u32()?;
            for _ in 0..var_count {
                let function_name = reader.read_string()?;
                let names_count = reader.read_u32()?;
                let mut names = Vec::with_capacity(names_count as usize);
                for _ in 0..names_count {
                    names.push(reader.read_string()?);
                }
                debug_info.add_variable_names(function_name, names);
            }
        }

        Ok(debug_info)
    }

    /// 从输入流读取紧凑操作数
    ///
    /// 读取标记字节：如果为 0x00，读取 u8 并返回为 u32；
    /// 如果为 0xFF，读取 u32_le。
    fn read_compact_operand(reader: &mut impl Read) -> std::io::Result<u32> {
        let mut marker = [0u8; 1];
        reader.read_exact(&mut marker)?;
        match marker[0] {
            COMPACT_SHORT_MARKER => {
                let mut buf = [0u8; 1];
                reader.read_exact(&mut buf)?;
                Ok(buf[0] as u32)
            }
            COMPACT_LONG_MARKER => {
                let mut buf = [0u8; 4];
                reader.read_exact(&mut buf)?;
                Ok(u32::from_le_bytes(buf))
            }
            _ => {
                Err(std::io::Error::new(std::io::ErrorKind::InvalidData, format!("无效的紧凑操作数标记: 0x{:02X}", marker[0])))
            }
        }
    }

    /// 从 BinaryReader 读取紧凑操作数，将 io 错误转换为 GError
    fn read_compact_operand_gresult(reader: &mut BinaryReader) -> GResult<u32> {
        Self::read_compact_operand(reader)
            .map_err(|e| GError { kind: GErrorKind::Runtime, message: format!("读取紧凑操作数失败: {}", e) })
    }

    /// 读取单条指令
    ///
    /// v3 格式使用紧凑操作数编码，v2 格式使用固定 u32 操作数。
    fn read_instruction(reader: &mut BinaryReader, version: u32) -> GResult<BytecodeInstruction> {
        let opcode_byte = reader.read_u8()?;
        let opcode = BytecodeOpCode::from_byte(opcode_byte)
            .ok_or_else(|| GError {
                kind: GErrorKind::Runtime, message: format!("未知的操作码: 0x{:02X}", opcode_byte)
            })?;

        let use_compact = version >= 3;

        let instruction = match opcode {
            BytecodeOpCode::LoadConst => {
                let index = if use_compact { Self::read_compact_operand_gresult(reader)? } else { reader.read_u32()? };
                BytecodeInstruction::LoadConst { index }
            }
            BytecodeOpCode::LoadConstInt => {
                let index = if use_compact { Self::read_compact_operand_gresult(reader)? } else { reader.read_u32()? };
                BytecodeInstruction::LoadConstInt { index }
            }
            BytecodeOpCode::LoadConstFloat => {
                let index = if use_compact { Self::read_compact_operand_gresult(reader)? } else { reader.read_u32()? };
                BytecodeInstruction::LoadConstFloat { index }
            }
            BytecodeOpCode::LoadConstString => {
                let index = if use_compact { Self::read_compact_operand_gresult(reader)? } else { reader.read_u32()? };
                BytecodeInstruction::LoadConstString { index }
            }
            BytecodeOpCode::LoadConstEntity => {
                let index = if use_compact { Self::read_compact_operand_gresult(reader)? } else { reader.read_u32()? };
                BytecodeInstruction::LoadConstEntity { index }
            }
            BytecodeOpCode::LoadConstBool => {
                let index = if use_compact { Self::read_compact_operand_gresult(reader)? } else { reader.read_u32()? };
                BytecodeInstruction::LoadConstBool { index }
            }
            BytecodeOpCode::LoadNull => BytecodeInstruction::LoadNull,
            BytecodeOpCode::LoadTrue => BytecodeInstruction::LoadTrue,
            BytecodeOpCode::LoadFalse => BytecodeInstruction::LoadFalse,
            BytecodeOpCode::LoadLocal => {
                let index = if use_compact { Self::read_compact_operand_gresult(reader)? } else { reader.read_u32()? };
                BytecodeInstruction::LoadLocal { index }
            }
            BytecodeOpCode::StoreLocal => {
                let index = if use_compact { Self::read_compact_operand_gresult(reader)? } else { reader.read_u32()? };
                BytecodeInstruction::StoreLocal { index }
            }
            BytecodeOpCode::Add => BytecodeInstruction::Add,
            BytecodeOpCode::Sub => BytecodeInstruction::Sub,
            BytecodeOpCode::Mul => BytecodeInstruction::Mul,
            BytecodeOpCode::Div => BytecodeInstruction::Div,
            BytecodeOpCode::Mod => BytecodeInstruction::Mod,
            BytecodeOpCode::Neg => BytecodeInstruction::Neg,
            BytecodeOpCode::Eq => BytecodeInstruction::Eq,
            BytecodeOpCode::Ne => BytecodeInstruction::Ne,
            BytecodeOpCode::Lt => BytecodeInstruction::Lt,
            BytecodeOpCode::Le => BytecodeInstruction::Le,
            BytecodeOpCode::Gt => BytecodeInstruction::Gt,
            BytecodeOpCode::Ge => BytecodeInstruction::Ge,
            BytecodeOpCode::And => BytecodeInstruction::And,
            BytecodeOpCode::Or => BytecodeInstruction::Or,
            BytecodeOpCode::Not => BytecodeInstruction::Not,
            BytecodeOpCode::Jump => {
                let address = if use_compact { Self::read_compact_operand_gresult(reader)? } else { reader.read_u32()? };
                BytecodeInstruction::Jump { address }
            }
            BytecodeOpCode::JumpIfFalse => {
                let address = if use_compact { Self::read_compact_operand_gresult(reader)? } else { reader.read_u32()? };
                BytecodeInstruction::JumpIfFalse { address }
            }
            BytecodeOpCode::JumpIfTrue => {
                let address = if use_compact { Self::read_compact_operand_gresult(reader)? } else { reader.read_u32()? };
                BytecodeInstruction::JumpIfTrue { address }
            }
            BytecodeOpCode::Call => {
                let arg_count = if use_compact { Self::read_compact_operand_gresult(reader)? } else { reader.read_u32()? };
                BytecodeInstruction::Call { arg_count }
            }
            BytecodeOpCode::TailCall => {
                let arity = if use_compact { Self::read_compact_operand_gresult(reader)? } else { reader.read_u32()? };
                BytecodeInstruction::TailCall(arity)
            }
            BytecodeOpCode::Return => BytecodeInstruction::Return,
            BytecodeOpCode::SpawnEntity => BytecodeInstruction::SpawnEntity,
            BytecodeOpCode::DespawnEntity => BytecodeInstruction::DespawnEntity,
            BytecodeOpCode::AddComponent => {
                let type_name_index =
                    if use_compact { Self::read_compact_operand_gresult(reader)? } else { reader.read_u32()? };
                BytecodeInstruction::AddComponent { type_name_index }
            }
            BytecodeOpCode::GetComponent => {
                let type_name_index =
                    if use_compact { Self::read_compact_operand_gresult(reader)? } else { reader.read_u32()? };
                BytecodeInstruction::GetComponent { type_name_index }
            }
            BytecodeOpCode::SetComponent => {
                let type_name_index =
                    if use_compact { Self::read_compact_operand_gresult(reader)? } else { reader.read_u32()? };
                BytecodeInstruction::SetComponent { type_name_index }
            }
            BytecodeOpCode::HostCall => {
                let name_index = if use_compact { Self::read_compact_operand_gresult(reader)? } else { reader.read_u32()? };
                let arg_count = if use_compact { Self::read_compact_operand_gresult(reader)? } else { reader.read_u32()? };
                BytecodeInstruction::HostCall { name_index, arg_count }
            }
            BytecodeOpCode::Pop => BytecodeInstruction::Pop,
            BytecodeOpCode::Dup => BytecodeInstruction::Dup,
            BytecodeOpCode::GetField => {
                let name_index = if use_compact { Self::read_compact_operand_gresult(reader)? } else { reader.read_u32()? };
                BytecodeInstruction::GetField { name_index }
            }
            BytecodeOpCode::SetField => {
                let name_index = if use_compact { Self::read_compact_operand_gresult(reader)? } else { reader.read_u32()? };
                BytecodeInstruction::SetField { name_index }
            }
            BytecodeOpCode::GetIndex => BytecodeInstruction::GetIndex,
            BytecodeOpCode::SetIndex => BytecodeInstruction::SetIndex,
            BytecodeOpCode::NewObject => {
                let field_count = if use_compact { Self::read_compact_operand_gresult(reader)? } else { reader.read_u32()? };
                BytecodeInstruction::NewObject { field_count }
            }
            BytecodeOpCode::NewList => {
                let element_count =
                    if use_compact { Self::read_compact_operand_gresult(reader)? } else { reader.read_u32()? };
                BytecodeInstruction::NewList { element_count }
            }
            BytecodeOpCode::NewMap => {
                let pair_count = if use_compact { Self::read_compact_operand_gresult(reader)? } else { reader.read_u32()? };
                BytecodeInstruction::NewMap { pair_count }
            }
            BytecodeOpCode::StringConcat => {
                let count = if use_compact { Self::read_compact_operand_gresult(reader)? } else { reader.read_u32()? };
                BytecodeInstruction::StringConcat { count }
            }
        };

        Ok(instruction)
    }
}
