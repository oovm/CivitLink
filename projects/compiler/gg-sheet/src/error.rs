//! 配置表错误类型模块
//! 定义配置表编译过程中的错误类型

use std::path::PathBuf;

use gg_core::{GError, GErrorKind};

/// 配置表错误类型
#[derive(Debug)]
pub enum SheetError {
    /// IO 错误，包含文件路径和错误描述
    Io {
        /// 出错的文件路径
        path: PathBuf,
        /// 错误描述
        message: String,
    },
    /// 解析错误，包含文件路径、行号和错误描述
    Parse {
        /// 出错的文件路径
        path: PathBuf,
        /// 出错的行号（从 1 开始）
        line: usize,
        /// 错误描述
        message: String,
    },
    /// 类型错误，包含文件路径、列号和无法识别的类型字符串
    Type {
        /// 出错的文件路径
        path: PathBuf,
        /// 出错的列号（从 0 开始）
        column: usize,
        /// 无法识别的类型字符串
        type_str: String,
    },
    /// 代码生成错误
    Codegen {
        /// 错误描述
        message: String,
    },
    /// 配置错误
    Config {
        /// 错误描述
        message: String,
    },
}

impl std::fmt::Display for SheetError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SheetError::Io { path, message } => {
                write!(f, "IO 错误 ({}): {}", path.display(), message)
            }
            SheetError::Parse { path, line, message } => {
                write!(f, "解析错误 ({}:{}): {}", path.display(), line, message)
            }
            SheetError::Type { path, column, type_str } => {
                write!(f, "类型错误 ({}:列{}): 无法识别的类型 '{}'", path.display(), column, type_str)
            }
            SheetError::Codegen { message } => {
                write!(f, "代码生成错误: {}", message)
            }
            SheetError::Config { message } => {
                write!(f, "配置错误: {}", message)
            }
        }
    }
}

impl std::error::Error for SheetError {}

impl From<SheetError> for GError {
    fn from(err: SheetError) -> Self {
        GError { kind: GErrorKind::Other, message: err.to_string() }
    }
}

/// 配置表结果类型
pub type SheetResult<T> = Result<T, SheetError>;
