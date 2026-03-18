#![warn(missing_docs)]

//! WAE Types - WAE 框架的基础类型定义
//!
//! 提供统一的错误类型、计费维度和工具函数。
//!
//! 作为微服务优先框架的基础层，所有类型都设计为：
//! - 异步友好：支持 tokio 运行时
//! - 序列化就绪：支持 JSON/MessagePack 等格式
//! - 零拷贝：最小化数据复制

pub use rust_decimal::Decimal;
pub use rust_decimal_macros::dec;

mod error;
mod value;

pub use error::*;
pub use value::*;

/// 计费维度
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct BillingDimensions {
    /// 输入文本字符数
    pub input_text: u64,
    /// 输出文本字符数
    pub output_text: u64,
    /// 输入像素数
    pub input_pixels: u64,
    /// 输出像素数
    pub output_pixels: u64,
}

/// 数据库错误类型别名
pub type DatabaseErrorKind = WaeErrorKind;

/// 构建对象的便捷宏
#[macro_export]
macro_rules! object {
    () => {
        $crate::Value::Object(std::collections::HashMap::new())
    };

    ($($key:expr => $value:expr),+ $(,)?) => {
        {
            let mut map = std::collections::HashMap::new();
            $(
                map.insert($key.to_string(), $value.into());
            )+
            $crate::Value::Object(map)
        }
    };
}

/// 构建数组的便捷宏
#[macro_export]
macro_rules! array {
    () => {
        $crate::Value::Array(Vec::new())
    };

    ($($value:expr),+ $(,)?) => {
        {
            let mut arr = Vec::new();
            $(
                arr.push($value.into());
            )+
            $crate::Value::Array(arr)
        }
    };
}

/// Common helper to format display names or slugs
pub fn format_slug(name: &str) -> String {
    name.to_lowercase().replace(' ', "-")
}

/// Common helper to truncate strings for logging
pub fn truncate_str(s: &str, max_len: usize) -> String {
    if s.len() <= max_len { s.to_string() } else { format!("{}...", &s[..max_len]) }
}

/// 将字节数组编码为十六进制字符串
pub fn hex_encode(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{:02x}", b)).collect()
}

/// 将十六进制字符串解码为字节数组
pub fn hex_decode(s: &str) -> Result<Vec<u8>, String> {
    if !s.len().is_multiple_of(2) {
        return Err("Invalid hex string length".to_string());
    }
    (0..s.len()).step_by(2).map(|i| u8::from_str_radix(&s[i..i + 2], 16).map_err(|e| e.to_string())).collect()
}

/// URL 编码字符串
pub fn url_encode(s: &str) -> String {
    s.chars()
        .map(|c| if c.is_alphanumeric() || "-_.~".contains(c) { c.to_string() } else { format!("%{:02X}", c as u8) })
        .collect()
}

/// URL 解码字符串
pub fn url_decode(s: &str) -> Result<String, String> {
    let mut result = String::new();
    let mut chars = s.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '%' {
            let hex: String = chars.by_ref().take(2).collect();
            let byte = u8::from_str_radix(&hex, 16).map_err(|e| e.to_string())?;
            result.push(byte as char);
        }
        else if c == '+' {
            result.push(' ');
        }
        else {
            result.push(c);
        }
    }
    Ok(result)
}
