//! 配置表类型系统模块
//! 定义配置表字段的类型描述和值类型

use crate::error::{SheetError, SheetResult};

/// 整数类型精度
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IntegerKind {
    /// 8 位有符号整数
    Integer8,
    /// 16 位有符号整数
    Integer16,
    /// 32 位有符号整数
    Integer32,
    /// 64 位有符号整数
    Integer64,
    /// 8 位无符号整数
    Unsigned8,
    /// 16 位无符号整数
    Unsigned16,
    /// 32 位无符号整数
    Unsigned32,
    /// 64 位无符号整数
    Unsigned64,
}

/// 小数类型精度
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DecimalKind {
    /// 32 位浮点数
    Float32,
    /// 64 位浮点数
    Float64,
}

/// 向量类型
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VectorKind {
    /// 二维向量
    Vec2,
    /// 三维向量
    Vec3,
    /// 四维向量
    Vec4,
}

/// 配置表字段类型
#[derive(Debug, Clone, PartialEq)]
pub enum SheetType {
    /// 布尔类型
    Boolean,
    /// 整数类型
    Integer(IntegerKind),
    /// 小数类型
    Decimal(DecimalKind),
    /// 字符串类型
    String,
    /// 颜色类型
    Color,
    /// 向量类型
    Vector(VectorKind),
    /// 列表类型
    List(Box<SheetType>),
    /// 映射类型
    Map {
        /// 键类型
        key: Box<SheetType>,
        /// 值类型
        value: Box<SheetType>,
    },
    /// 可选类型
    Optional(Box<SheetType>),
    /// 引用类型（引用另一张表的主键）
    Reference(String),
    /// 枚举类型
    Enumerate(String),
}

impl SheetType {
    /// 从类型字符串解析为 SheetType
    pub fn parse(type_str: &str) -> SheetResult<SheetType> {
        let trimmed = type_str.trim();
        if trimmed.is_empty() {
            return Err(SheetError::Config { message: "类型字符串为空".to_string() });
        }

        if trimmed.ends_with('?') {
            let inner = &trimmed[..trimmed.len() - 1];
            let inner_type = Self::parse(inner)?;
            return Ok(SheetType::Optional(Box::new(inner_type)));
        }

        if trimmed.starts_with('&') {
            let table_name = &trimmed[1..];
            return Ok(SheetType::Reference(table_name.to_string()));
        }

        if trimmed.starts_with('[') && trimmed.ends_with(']') {
            let inner = &trimmed[1..trimmed.len() - 1];
            let inner_type = Self::parse(inner)?;
            return Ok(SheetType::List(Box::new(inner_type)));
        }

        if trimmed.starts_with("Map<") && trimmed.ends_with('>') {
            let inner = &trimmed[4..trimmed.len() - 1];
            let parts: Vec<&str> = inner.splitn(2, ',').collect();
            if parts.len() != 2 {
                return Err(SheetError::Config { message: format!("映射类型格式错误: '{}'", trimmed) });
            }
            let key_type = Self::parse(parts[0].trim())?;
            let value_type = Self::parse(parts[1].trim())?;
            return Ok(SheetType::Map { key: Box::new(key_type), value: Box::new(value_type) });
        }

        match trimmed {
            "bool" | "boolean" => Ok(SheetType::Boolean),
            "i8" | "char" => Ok(SheetType::Integer(IntegerKind::Integer8)),
            "i16" | "short" => Ok(SheetType::Integer(IntegerKind::Integer16)),
            "i32" | "int" => Ok(SheetType::Integer(IntegerKind::Integer32)),
            "i64" | "long" | "longlong" => Ok(SheetType::Integer(IntegerKind::Integer64)),
            "u8" | "byte" => Ok(SheetType::Integer(IntegerKind::Unsigned8)),
            "u16" | "ushort" => Ok(SheetType::Integer(IntegerKind::Unsigned16)),
            "u32" | "uint" => Ok(SheetType::Integer(IntegerKind::Unsigned32)),
            "u64" | "ulong" => Ok(SheetType::Integer(IntegerKind::Unsigned64)),
            "f32" | "float" => Ok(SheetType::Decimal(DecimalKind::Float32)),
            "f64" | "double" => Ok(SheetType::Decimal(DecimalKind::Float64)),
            "string" | "str" | "text" => Ok(SheetType::String),
            "color" | "colour" => Ok(SheetType::Color),
            "vec2" | "v2" => Ok(SheetType::Vector(VectorKind::Vec2)),
            "vec3" | "v3" => Ok(SheetType::Vector(VectorKind::Vec3)),
            "vec4" | "v4" => Ok(SheetType::Vector(VectorKind::Vec4)),
            other => Ok(SheetType::Enumerate(other.to_string())),
        }
    }

    /// 将类型转换为 Valkyrie 脚本中的类型名
    pub fn to_valkyrie_type(&self) -> String {
        match self {
            SheetType::Boolean => "bool".to_string(),
            SheetType::Integer(kind) => match kind {
                IntegerKind::Integer8 => "i8".to_string(),
                IntegerKind::Integer16 => "i16".to_string(),
                IntegerKind::Integer32 => "i32".to_string(),
                IntegerKind::Integer64 => "i64".to_string(),
                IntegerKind::Unsigned8 => "u8".to_string(),
                IntegerKind::Unsigned16 => "u16".to_string(),
                IntegerKind::Unsigned32 => "u32".to_string(),
                IntegerKind::Unsigned64 => "u64".to_string(),
            },
            SheetType::Decimal(kind) => match kind {
                DecimalKind::Float32 => "f32".to_string(),
                DecimalKind::Float64 => "f64".to_string(),
            },
            SheetType::String => "string".to_string(),
            SheetType::Color => "Color".to_string(),
            SheetType::Vector(kind) => match kind {
                VectorKind::Vec2 => "Vec2".to_string(),
                VectorKind::Vec3 => "Vec3".to_string(),
                VectorKind::Vec4 => "Vec4".to_string(),
            },
            SheetType::List(inner) => format!("List<{}>", inner.to_valkyrie_type()),
            SheetType::Map { key, value } => format!("Map<{}, {}>", key.to_valkyrie_type(), value.to_valkyrie_type()),
            SheetType::Optional(inner) => format!("Option<{}>", inner.to_valkyrie_type()),
            SheetType::Reference(_) => "i32".to_string(),
            SheetType::Enumerate(name) => name.clone(),
        }
    }
}

/// 配置表字段值
#[derive(Debug, Clone, PartialEq)]
pub enum SheetValue {
    /// 布尔值
    Boolean(bool),
    /// 8 位有符号整数
    Integer8(i8),
    /// 16 位有符号整数
    Integer16(i16),
    /// 32 位有符号整数
    Integer32(i32),
    /// 64 位有符号整数
    Integer64(i64),
    /// 8 位无符号整数
    Unsigned8(u8),
    /// 16 位无符号整数
    Unsigned16(u16),
    /// 32 位无符号整数
    Unsigned32(u32),
    /// 64 位无符号整数
    Unsigned64(u64),
    /// 32 位浮点数
    Float32(f32),
    /// 64 位浮点数
    Float64(f64),
    /// 字符串值
    String(String),
    /// 列表值
    List(Vec<SheetValue>),
    /// 映射值
    Map(Vec<(String, SheetValue)>),
    /// 可选值
    Optional(Option<Box<SheetValue>>),
    /// 枚举值
    Enumerate(String),
    /// 引用值（目标表主键）
    Reference(i64),
}

impl SheetValue {
    /// 从字符串值和类型信息解析为 SheetValue
    pub fn parse_from_str(value: &str, ty: &SheetType) -> SheetResult<SheetValue> {
        let trimmed = value.trim();
        match ty {
            SheetType::Boolean => match trimmed.to_lowercase().as_str() {
                "true" | "1" => Ok(SheetValue::Boolean(true)),
                "false" | "0" | "" => Ok(SheetValue::Boolean(false)),
                _ => Err(SheetError::Parse {
                    path: std::path::PathBuf::new(),
                    line: 0,
                    message: format!("无法将 '{}' 解析为布尔值", trimmed),
                }),
            },
            SheetType::Integer(kind) => {
                if trimmed.is_empty() {
                    return Ok(Self::default_integer(kind));
                }
                match kind {
                    IntegerKind::Integer8 => trimmed.parse::<i8>().map(SheetValue::Integer8),
                    IntegerKind::Integer16 => trimmed.parse::<i16>().map(SheetValue::Integer16),
                    IntegerKind::Integer32 => trimmed.parse::<i32>().map(SheetValue::Integer32),
                    IntegerKind::Integer64 => trimmed.parse::<i64>().map(SheetValue::Integer64),
                    IntegerKind::Unsigned8 => trimmed.parse::<u8>().map(SheetValue::Unsigned8),
                    IntegerKind::Unsigned16 => trimmed.parse::<u16>().map(SheetValue::Unsigned16),
                    IntegerKind::Unsigned32 => trimmed.parse::<u32>().map(SheetValue::Unsigned32),
                    IntegerKind::Unsigned64 => trimmed.parse::<u64>().map(SheetValue::Unsigned64),
                }
                .map_err(|_| SheetError::Parse {
                    path: std::path::PathBuf::new(),
                    line: 0,
                    message: format!("无法将 '{}' 解析为整数", trimmed),
                })
            }
            SheetType::Decimal(kind) => {
                if trimmed.is_empty() {
                    return Ok(Self::default_decimal(kind));
                }
                match kind {
                    DecimalKind::Float32 => trimmed.parse::<f32>().map(|v| SheetValue::Float32(v)),
                    DecimalKind::Float64 => trimmed.parse::<f64>().map(|v| SheetValue::Float64(v)),
                }
                .map_err(|_| SheetError::Parse {
                    path: std::path::PathBuf::new(),
                    line: 0,
                    message: format!("无法将 '{}' 解析为浮点数", trimmed),
                })
            }
            SheetType::String => Ok(SheetValue::String(trimmed.to_string())),
            SheetType::Color => Ok(SheetValue::String(trimmed.to_string())),
            SheetType::Vector(_) => Ok(SheetValue::String(trimmed.to_string())),
            SheetType::List(inner) => {
                if trimmed.is_empty() {
                    return Ok(SheetValue::List(Vec::new()));
                }
                let items: Vec<SheetValue> = trimmed.split(',').filter_map(|s| Self::parse_from_str(s, inner).ok()).collect();
                Ok(SheetValue::List(items))
            }
            SheetType::Map { .. } => Ok(SheetValue::String(trimmed.to_string())),
            SheetType::Optional(inner) => {
                if trimmed.is_empty() {
                    Ok(SheetValue::Optional(None))
                }
                else {
                    let val = Self::parse_from_str(trimmed, inner)?;
                    Ok(SheetValue::Optional(Some(Box::new(val))))
                }
            }
            SheetType::Reference(_) => {
                if trimmed.is_empty() {
                    Ok(SheetValue::Reference(0))
                }
                else {
                    trimmed.parse::<i64>().map(SheetValue::Reference).map_err(|_| SheetError::Parse {
                        path: std::path::PathBuf::new(),
                        line: 0,
                        message: format!("无法将 '{}' 解析为引用键", trimmed),
                    })
                }
            }
            SheetType::Enumerate(_) => Ok(SheetValue::Enumerate(trimmed.to_string())),
        }
    }

    /// 获取整数类型的默认值
    fn default_integer(kind: &IntegerKind) -> SheetValue {
        match kind {
            IntegerKind::Integer8 => SheetValue::Integer8(0),
            IntegerKind::Integer16 => SheetValue::Integer16(0),
            IntegerKind::Integer32 => SheetValue::Integer32(0),
            IntegerKind::Integer64 => SheetValue::Integer64(0),
            IntegerKind::Unsigned8 => SheetValue::Unsigned8(0),
            IntegerKind::Unsigned16 => SheetValue::Unsigned16(0),
            IntegerKind::Unsigned32 => SheetValue::Unsigned32(0),
            IntegerKind::Unsigned64 => SheetValue::Unsigned64(0),
        }
    }

    /// 获取浮点类型的默认值
    fn default_decimal(kind: &DecimalKind) -> SheetValue {
        match kind {
            DecimalKind::Float32 => SheetValue::Float32(0.0),
            DecimalKind::Float64 => SheetValue::Float64(0.0),
        }
    }
}
