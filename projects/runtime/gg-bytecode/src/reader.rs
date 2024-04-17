use gg_core::{GError, GErrorKind, GResult};

use crate::{
    debug_info::{DebugInfo, SourceLocation},
    format::{
        BinaryReader, BytecodeFunction, BytecodeInstruction, BytecodeModule, BytecodeOpCode, BytecodeValue, MAGIC, VERSION,
    },
};

/// 调试信息段魔数 "DEBU"
const DEBUG_MAGIC: u32 = 0x44454255;

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
        if version != VERSION {
            return Err(GError {
                kind: GErrorKind::Runtime,
                message: format!("不支持的字节码版本: 期望 {}, 实际 {}", VERSION, version),
            });
        }

        let name = reader.read_string()?;

        let constants = Self::read_constants(&mut reader)?;
        let string_pool = Self::read_string_pool(&mut reader)?;
        let functions = Self::read_functions(&mut reader)?;
        let debug_info = Self::try_read_debug_info(&mut reader)?;

        Ok(BytecodeModule { name, version, constants, string_pool, functions, debug_info })
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
    fn read_functions(reader: &mut BinaryReader) -> GResult<Vec<BytecodeFunction>> {
        let count = reader.read_u32()?;
        let mut functions = Vec::with_capacity(count as usize);

        for _ in 0..count {
            let name = reader.read_string()?;
            let param_count = reader.read_u32()?;
            let local_count = reader.read_u32()?;

            let inst_count = reader.read_u32()?;
            let mut instructions = Vec::with_capacity(inst_count as usize);

            for _ in 0..inst_count {
                instructions.push(Self::read_instruction(reader)?);
            }

            functions.push(BytecodeFunction { name, param_count, local_count, instructions });
        }

        Ok(functions)
    }

    /// 尝试读取调试信息段（向后兼容）
    fn try_read_debug_info(reader: &mut BinaryReader) -> GResult<Option<DebugInfo>> {
        if reader.remaining() < 4 {
            return Ok(None);
        }

        let magic = reader.read_u32()?;
        if magic != DEBUG_MAGIC {
            return Ok(None);
        }

        Ok(Some(Self::read_debug_info(reader)?))
    }

    /// 读取调试信息段
    fn read_debug_info(reader: &mut BinaryReader) -> GResult<DebugInfo> {
        let mut debug_info = DebugInfo::new();

        let entry_count = reader.read_u32()?;
        for _ in 0..entry_count {
            let offset = reader.read_u32()?;
            let file = reader.read_string()?;
            let line = reader.read_u32()?;
            let column = reader.read_u32()?;
            debug_info.add_entry(offset, SourceLocation::new(file, line, column));
        }

        let func_count = reader.read_u32()?;
        for _ in 0..func_count {
            let function_name = reader.read_string()?;
            let source_file = reader.read_string()?;
            debug_info.add_function(function_name, source_file);
        }

        Ok(debug_info)
    }

    /// 读取单条指令
    fn read_instruction(reader: &mut BinaryReader) -> GResult<BytecodeInstruction> {
        let opcode_byte = reader.read_u8()?;
        let opcode = BytecodeOpCode::from_byte(opcode_byte)
            .ok_or_else(|| GError {
                kind: GErrorKind::Runtime, message: format!("未知的操作码: 0x{:02X}", opcode_byte)
            })?;

        let instruction = match opcode {
            BytecodeOpCode::LoadConst => BytecodeInstruction::LoadConst { index: reader.read_u32()? },
            BytecodeOpCode::LoadNull => BytecodeInstruction::LoadNull,
            BytecodeOpCode::LoadTrue => BytecodeInstruction::LoadTrue,
            BytecodeOpCode::LoadFalse => BytecodeInstruction::LoadFalse,
            BytecodeOpCode::LoadLocal => BytecodeInstruction::LoadLocal { index: reader.read_u32()? },
            BytecodeOpCode::StoreLocal => BytecodeInstruction::StoreLocal { index: reader.read_u32()? },
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
            BytecodeOpCode::Jump => BytecodeInstruction::Jump { address: reader.read_u32()? },
            BytecodeOpCode::JumpIfFalse => BytecodeInstruction::JumpIfFalse { address: reader.read_u32()? },
            BytecodeOpCode::JumpIfTrue => BytecodeInstruction::JumpIfTrue { address: reader.read_u32()? },
            BytecodeOpCode::Call => BytecodeInstruction::Call { arg_count: reader.read_u32()? },
            BytecodeOpCode::Return => BytecodeInstruction::Return,
            BytecodeOpCode::SpawnEntity => BytecodeInstruction::SpawnEntity,
            BytecodeOpCode::DespawnEntity => BytecodeInstruction::DespawnEntity,
            BytecodeOpCode::AddComponent => BytecodeInstruction::AddComponent { type_name_index: reader.read_u32()? },
            BytecodeOpCode::GetComponent => BytecodeInstruction::GetComponent { type_name_index: reader.read_u32()? },
            BytecodeOpCode::SetComponent => BytecodeInstruction::SetComponent { type_name_index: reader.read_u32()? },
            BytecodeOpCode::HostCall => {
                BytecodeInstruction::HostCall { name_index: reader.read_u32()?, arg_count: reader.read_u32()? }
            }
            BytecodeOpCode::Pop => BytecodeInstruction::Pop,
            BytecodeOpCode::Dup => BytecodeInstruction::Dup,
        };

        Ok(instruction)
    }
}
