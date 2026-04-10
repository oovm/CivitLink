use gg_core::{GError, GErrorKind, GResult};

use crate::format::{
    BinaryReader, BytecodeFunction, BytecodeInstruction, BytecodeModule, BytecodeOpCode,
    BytecodeValue, MAGIC, VERSION,
};

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
                message: format!(
                    "无效的字节码魔数: 期望 0x{:08X}, 实际 0x{:08X}",
                    MAGIC, magic
                ),
            });
        }

        let version = reader.read_u32()?;
        if version != VERSION {
            return Err(GError {
                kind: GErrorKind::Runtime,
                message: format!(
                    "不支持的字节码版本: 期望 {}, 实际 {}",
                    VERSION, version
                ),
            });
        }

        let name = reader.read_string()?;

        let constants = Self::read_constants(&mut reader)?;
        let string_pool = Self::read_string_pool(&mut reader)?;
        let functions = Self::read_functions(&mut reader)?;

        Ok(BytecodeModule {
            name,
            version,
            constants,
            string_pool,
            functions,
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
                    return Err(GError {
                        kind: GErrorKind::Runtime,
                        message: format!("未知的常量类型标签: {}", tag),
                    });
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
    fn read_functions(
        reader: &mut BinaryReader,
    ) -> GResult<Vec<BytecodeFunction>> {
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

            functions.push(BytecodeFunction {
                name,
                param_count,
                local_count,
                instructions,
            });
        }

        Ok(functions)
    }

    /// 读取单条指令
    fn read_instruction(
        reader: &mut BinaryReader,
    ) -> GResult<BytecodeInstruction> {
        let opcode_byte = reader.read_u8()?;
        let opcode = BytecodeOpCode::from_byte(opcode_byte).ok_or_else(|| GError {
            kind: GErrorKind::Runtime,
            message: format!("未知的操作码: 0x{:02X}", opcode_byte),
        })?;

        let instruction = match opcode {
            BytecodeOpCode::LoadConst => BytecodeInstruction::LoadConst {
                index: reader.read_u32()?,
            },
            BytecodeOpCode::LoadNull => BytecodeInstruction::LoadNull,
            BytecodeOpCode::LoadTrue => BytecodeInstruction::LoadTrue,
            BytecodeOpCode::LoadFalse => BytecodeInstruction::LoadFalse,
            BytecodeOpCode::LoadLocal => BytecodeInstruction::LoadLocal {
                index: reader.read_u32()?,
            },
            BytecodeOpCode::StoreLocal => BytecodeInstruction::StoreLocal {
                index: reader.read_u32()?,
            },
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
            BytecodeOpCode::Jump => BytecodeInstruction::Jump {
                address: reader.read_u32()?,
            },
            BytecodeOpCode::JumpIfFalse => BytecodeInstruction::JumpIfFalse {
                address: reader.read_u32()?,
            },
            BytecodeOpCode::JumpIfTrue => BytecodeInstruction::JumpIfTrue {
                address: reader.read_u32()?,
            },
            BytecodeOpCode::Call => BytecodeInstruction::Call {
                arg_count: reader.read_u32()?,
            },
            BytecodeOpCode::Return => BytecodeInstruction::Return,
            BytecodeOpCode::SpawnEntity => BytecodeInstruction::SpawnEntity,
            BytecodeOpCode::DespawnEntity => BytecodeInstruction::DespawnEntity,
            BytecodeOpCode::AddComponent => BytecodeInstruction::AddComponent {
                type_name_index: reader.read_u32()?,
            },
            BytecodeOpCode::GetComponent => BytecodeInstruction::GetComponent {
                type_name_index: reader.read_u32()?,
            },
            BytecodeOpCode::SetComponent => BytecodeInstruction::SetComponent {
                type_name_index: reader.read_u32()?,
            },
            BytecodeOpCode::HostCall => BytecodeInstruction::HostCall {
                name_index: reader.read_u32()?,
                arg_count: reader.read_u32()?,
            },
            BytecodeOpCode::Pop => BytecodeInstruction::Pop,
            BytecodeOpCode::Dup => BytecodeInstruction::Dup,
        };

        Ok(instruction)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::format::{BytecodeInstruction, BytecodeValue, MAGIC, VERSION};
    use crate::writer::BytecodeWriter;
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
}
