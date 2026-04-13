//! Schema 编译器错误类型模块

use gg_core::{GError, GErrorKind};

/// Schema 编译错误类型
#[derive(Debug)]
pub enum SchemaError {
    /// 解析错误
    ParseError {
        /// 错误消息
        message: String,
        /// 行号（从1开始）
        line: usize,
        /// 列号（从1开始）
        column: usize,
        /// 期望的语法
        expected: Option<String>,
    },
    /// 验证错误
    ValidationError(String),
    /// 代码生成错误
    CodegenError(String),
    /// 序列化错误
    SerializeError(String),
}

impl SchemaError {
    /// 创建简单的解析错误（行号和列号默认为0）
    pub fn parse_error(message: impl Into<String>) -> Self {
        SchemaError::ParseError { message: message.into(), line: 0, column: 0, expected: None }
    }

    /// 创建带位置信息的解析错误
    pub fn parse_error_at(message: impl Into<String>, line: usize, column: usize) -> Self {
        SchemaError::ParseError { message: message.into(), line, column, expected: None }
    }

    /// 创建带位置和期望语法的解析错误
    pub fn parse_error_expected(message: impl Into<String>, line: usize, column: usize, expected: impl Into<String>) -> Self {
        SchemaError::ParseError { message: message.into(), line, column, expected: Some(expected.into()) }
    }
}

impl std::fmt::Display for SchemaError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SchemaError::ParseError { message, line, column, expected } => {
                write!(f, "Schema 解析错误 (行:{line}, 列:{column}): {message}")?;
                if let Some(exp) = expected {
                    write!(f, ", 期望: {exp}")?;
                }
                Ok(())
            }
            SchemaError::ValidationError(msg) => write!(f, "Schema 验证错误: {msg}"),
            SchemaError::CodegenError(msg) => write!(f, "Schema 代码生成错误: {msg}"),
            SchemaError::SerializeError(msg) => write!(f, "Schema 序列化错误: {msg}"),
        }
    }
}

impl std::error::Error for SchemaError {}

impl From<SchemaError> for GError {
    fn from(err: SchemaError) -> Self {
        GError::with_kind(GErrorKind::Other, &err.to_string())
    }
}

/// Schema 编译结果类型
pub type SchemaResult<T> = std::result::Result<T, SchemaError>;
