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

        if trimmed.starts_with("HashMap<") && trimmed.ends_with('>') {
            let inner = &trimmed[8..trimmed.len() - 1];
            let parts: Vec<&str> = inner.splitn(2, ',').collect();
            if parts.len() != 2 {
                return Err(SheetError::Config { message: format!("映射类型格式错误: '{}'", trimmed) });
            }
            let key_type = Self::parse(parts[0].trim())?;
            let value_type = Self::parse(parts[1].trim())?;
            return Ok(SheetType::Map { key: Box::new(key_type), value: Box::new(value_type) });
        }

        if trimmed.starts_with("dict<") && trimmed.ends_with('>') {
            let inner = &trimmed[5..trimmed.len() - 1];
            let parts: Vec<&str> = inner.splitn(2, ',').collect();
            if parts.len() != 2 {
                return Err(SheetError::Config { message: format!("映射类型格式错误: '{}'", trimmed) });
            }
            let key_type = Self::parse(parts[0].trim())?;
            let value_type = Self::parse(parts[1].trim())?;
            return Ok(SheetType::Map { key: Box::new(key_type), value: Box::new(value_type) });
        }

        if trimmed.starts_with("list<") && trimmed.ends_with('>') {
            let inner = &trimmed[5..trimmed.len() - 1];
            let inner_type = Self::parse(inner.trim())?;
            return Ok(SheetType::List(Box::new(inner_type)));
        }

        if trimmed.starts_with("Vec<") && trimmed.ends_with('>') {
            let inner = &trimmed[4..trimmed.len() - 1];
            let inner_type = Self::parse(inner.trim())?;
            return Ok(SheetType::List(Box::new(inner_type)));
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
            "string" | "str" | "text" | "utf8" => Ok(SheetType::String),
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
            SheetType::String => "UTF8Text".to_string(),
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
    /// 颜色值，包含红、绿、蓝、透明度通道
    Color {
        /// 红色通道（0-255）
        r: u8,
        /// 绿色通道（0-255）
        g: u8,
        /// 蓝色通道（0-255）
        b: u8,
        /// 透明度通道（0-255）
        a: u8,
    },
    /// 向量值，包含浮点数分量列表
    Vector {
        /// 分量列表
        components: Vec<f32>,
    },
    /// 列表值
    List(Vec<SheetValue>),
    /// 映射值，键值对列表
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
            SheetType::Color => {
                if trimmed.is_empty() {
                    return Ok(SheetValue::Color { r: 0, g: 0, b: 0, a: 255 });
                }
                let (r, g, b, a) = parse_color_str(trimmed)?;
                Ok(SheetValue::Color { r, g, b, a })
            }
            SheetType::Vector(kind) => {
                let expected = match kind {
                    VectorKind::Vec2 => 2,
                    VectorKind::Vec3 => 3,
                    VectorKind::Vec4 => 4,
                };
                if trimmed.is_empty() {
                    return Ok(SheetValue::Vector { components: vec![0.0; expected] });
                }
                let components = parse_vector_str(trimmed)?;
                if components.len() != expected {
                    return Err(SheetError::Parse {
                        path: std::path::PathBuf::new(),
                        line: 0,
                        message: format!("向量分量数量不匹配：期望 {} 个，实际 {} 个", expected, components.len()),
                    });
                }
                Ok(SheetValue::Vector { components })
            }
            SheetType::List(inner) => {
                if trimmed.is_empty() {
                    return Ok(SheetValue::List(Vec::new()));
                }
                let items: Vec<SheetValue> = trimmed.split(',').filter_map(|s| Self::parse_from_str(s, inner).ok()).collect();
                Ok(SheetValue::List(items))
            }
            SheetType::Map { value, .. } => {
                if trimmed.is_empty() {
                    return Ok(SheetValue::Map(Vec::new()));
                }
                let entries = parse_map_str(trimmed)?;
                let parsed_entries: Vec<(String, SheetValue)> = entries
                    .into_iter()
                    .map(|(k, v)| {
                        let val = Self::parse_from_str(&v, value)?;
                        Ok((k, val))
                    })
                    .collect::<SheetResult<Vec<(String, SheetValue)>>>()?;
                Ok(SheetValue::Map(parsed_entries))
            }
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

/// 字段验证规则
#[derive(Debug, Clone, PartialEq)]
pub enum ValidationRule {
    /// 数值最小值
    Min(f64),
    /// 数值最大值
    Max(f64),
    /// 字符串最小长度
    MinLength(usize),
    /// 字符串最大长度
    MaxLength(usize),
    /// 正则模式匹配
    Pattern(String),
    /// 必填字段
    Required,
    /// 自定义验证器引用
    Custom(String),
}

impl std::fmt::Display for ValidationRule {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ValidationRule::Min(v) => write!(f, "min({})", v),
            ValidationRule::Max(v) => write!(f, "max({})", v),
            ValidationRule::MinLength(v) => write!(f, "min_length({})", v),
            ValidationRule::MaxLength(v) => write!(f, "max_length({})", v),
            ValidationRule::Pattern(v) => write!(f, "pattern(\"{}\")", v),
            ValidationRule::Required => write!(f, "required"),
            ValidationRule::Custom(name) => write!(f, "custom({})", name),
        }
    }
}

/// 解析颜色字符串为 RGBA 分量
///
/// 支持以下格式：
/// - `#RRGGBB`：6 位十六进制，透明度默认为 255
/// - `#RRGGBBAA`：8 位十六进制
/// - `rgb(r, g, b)`：RGB 函数，透明度默认为 255
/// - `rgba(r, g, b, a)`：RGBA 函数
pub fn parse_color_str(s: &str) -> SheetResult<(u8, u8, u8, u8)> {
    let trimmed = s.trim();

    if trimmed.starts_with('#') {
        let hex = &trimmed[1..];
        match hex.len() {
            6 => {
                let r = u8::from_str_radix(&hex[0..2], 16).map_err(|_| color_parse_error(trimmed))?;
                let g = u8::from_str_radix(&hex[2..4], 16).map_err(|_| color_parse_error(trimmed))?;
                let b = u8::from_str_radix(&hex[4..6], 16).map_err(|_| color_parse_error(trimmed))?;
                Ok((r, g, b, 255))
            }
            8 => {
                let r = u8::from_str_radix(&hex[0..2], 16).map_err(|_| color_parse_error(trimmed))?;
                let g = u8::from_str_radix(&hex[2..4], 16).map_err(|_| color_parse_error(trimmed))?;
                let b = u8::from_str_radix(&hex[4..6], 16).map_err(|_| color_parse_error(trimmed))?;
                let a = u8::from_str_radix(&hex[6..8], 16).map_err(|_| color_parse_error(trimmed))?;
                Ok((r, g, b, a))
            }
            _ => Err(color_parse_error(trimmed)),
        }
    }
    else if trimmed.starts_with("rgba(") && trimmed.ends_with(')') {
        let inner = &trimmed[5..trimmed.len() - 1];
        let parts: Vec<&str> = inner.split(',').map(|p| p.trim()).collect();
        if parts.len() != 4 {
            return Err(color_parse_error(trimmed));
        }
        let r = parts[0].parse::<u8>().map_err(|_| color_parse_error(trimmed))?;
        let g = parts[1].parse::<u8>().map_err(|_| color_parse_error(trimmed))?;
        let b = parts[2].parse::<u8>().map_err(|_| color_parse_error(trimmed))?;
        let a = parts[3].parse::<u8>().map_err(|_| color_parse_error(trimmed))?;
        Ok((r, g, b, a))
    }
    else if trimmed.starts_with("rgb(") && trimmed.ends_with(')') {
        let inner = &trimmed[4..trimmed.len() - 1];
        let parts: Vec<&str> = inner.split(',').map(|p| p.trim()).collect();
        if parts.len() != 3 {
            return Err(color_parse_error(trimmed));
        }
        let r = parts[0].parse::<u8>().map_err(|_| color_parse_error(trimmed))?;
        let g = parts[1].parse::<u8>().map_err(|_| color_parse_error(trimmed))?;
        let b = parts[2].parse::<u8>().map_err(|_| color_parse_error(trimmed))?;
        Ok((r, g, b, 255))
    }
    else {
        Err(color_parse_error(trimmed))
    }
}

/// 构造颜色解析错误
fn color_parse_error(s: &str) -> SheetError {
    SheetError::Parse { path: std::path::PathBuf::new(), line: 0, message: format!("无法将 '{}' 解析为颜色值", s) }
}

/// 解析向量字符串为浮点数分量列表
///
/// 支持格式：`(x, y)`、`(x, y, z)`、`(x, y, z, w)`
pub fn parse_vector_str(s: &str) -> SheetResult<Vec<f32>> {
    let trimmed = s.trim();

    if !trimmed.starts_with('(') || !trimmed.ends_with(')') {
        return Err(SheetError::Parse {
            path: std::path::PathBuf::new(),
            line: 0,
            message: format!("无法将 '{}' 解析为向量值", trimmed),
        });
    }

    let inner = &trimmed[1..trimmed.len() - 1];
    if inner.trim().is_empty() {
        return Err(SheetError::Parse {
            path: std::path::PathBuf::new(), line: 0, message: "向量值不能为空".to_string()
        });
    }

    let components: Vec<f32> = inner
        .split(',')
        .map(|p| {
            p.trim().parse::<f32>().map_err(|_| SheetError::Parse {
                path: std::path::PathBuf::new(),
                line: 0,
                message: format!("无法将 '{}' 解析为向量分量", p.trim()),
            })
        })
        .collect::<SheetResult<Vec<f32>>>()?;

    Ok(components)
}

/// 解析映射字符串为键值对列表
///
/// 支持格式：`{key1:value1,key2:value2}`
/// 字符串值使用引号：`{name:"Hero",title:"Brave"}`
/// 空花括号 `{}` 返回空列表
pub fn parse_map_str(s: &str) -> SheetResult<Vec<(String, String)>> {
    let trimmed = s.trim();

    if !trimmed.starts_with('{') || !trimmed.ends_with('}') {
        return Err(SheetError::Parse {
            path: std::path::PathBuf::new(),
            line: 0,
            message: format!("无法将 '{}' 解析为映射值", trimmed),
        });
    }

    let inner = &trimmed[1..trimmed.len() - 1];
    if inner.trim().is_empty() {
        return Ok(Vec::new());
    }

    let mut entries = Vec::new();
    let mut current_key = String::new();
    let mut current_value = String::new();
    let mut in_key = true;
    let mut in_quotes = false;
    let mut chars = inner.chars().peekable();

    while let Some(c) = chars.next() {
        if in_quotes {
            if c == '"' {
                in_quotes = false;
            }
            else {
                current_value.push(c);
            }
            continue;
        }

        match c {
            '"' => {
                in_quotes = true;
            }
            ':' if in_key => {
                in_key = false;
            }
            ',' => {
                entries.push((current_key.trim().to_string(), current_value.trim().to_string()));
                current_key.clear();
                current_value.clear();
                in_key = true;
            }
            _ => {
                if in_key {
                    current_key.push(c);
                }
                else {
                    current_value.push(c);
                }
            }
        }
    }

    if !current_key.is_empty() || !current_value.is_empty() {
        entries.push((current_key.trim().to_string(), current_value.trim().to_string()));
    }

    Ok(entries)
}
