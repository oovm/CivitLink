#![warn(missing_docs)]

//! GG 引擎错误类型模块
//! 提供统一的错误处理类型

/// 错误类型枚举
#[derive(Debug)]
pub enum GErrorKind {
    /// IO 错误
    Io,
    /// 资源错误
    Asset,
    /// 平台错误
    Platform,
    /// ECS 错误
    Ecs,
    /// 插件错误
    Plugin,
    /// 运行时错误
    Runtime,
    /// 其他错误
    Other,
}

/// GG 引擎错误类型
#[derive(Debug)]
pub struct GError {
    /// 错误类型
    pub kind: GErrorKind,
    /// 错误消息
    pub message: String,
}

impl std::fmt::Display for GError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}: {}", self.kind, self.message)
    }
}

impl std::error::Error for GError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        None
    }
}

/// GG 引擎结果类型
pub type GResult<T> = std::result::Result<T, GError>;
