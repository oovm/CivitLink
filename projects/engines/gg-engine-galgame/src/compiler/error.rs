//! Galgame 编译器错误类型定义

use gg_core::{GError, GErrorKind};

/// Galgame 编译器错误枚举
#[derive(Debug, Clone)]
pub enum GalgameError {
    /// 解析错误
    ParseError(String),
    /// 语义错误
    SemanticError(String),
    /// 代码生成错误
    CodegenError(String),
    /// 序列化错误
    SerializeError(String),
}

impl std::fmt::Display for GalgameError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GalgameError::ParseError(msg) => write!(f, "Parse error: {}", msg),
            GalgameError::SemanticError(msg) => write!(f, "Semantic error: {}", msg),
            GalgameError::CodegenError(msg) => write!(f, "Codegen error: {}", msg),
            GalgameError::SerializeError(msg) => write!(f, "Serialize error: {}", msg),
        }
    }
}

impl std::error::Error for GalgameError {}

impl From<GalgameError> for GError {
    fn from(err: GalgameError) -> Self {
        GError { kind: GErrorKind::Other, message: err.to_string() }
    }
}

/// Galgame 编译器结果类型别名
pub type GalgameResult<T> = Result<T, GalgameError>;
