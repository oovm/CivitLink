use crate::target::TargetPlatform;

/// AOT 编译错误类型
#[derive(Debug)]
pub enum AotError {
    /// 不支持的目标平台
    UnsupportedTarget {
        /// 请求的目标平台
        target: TargetPlatform,
    },
    /// 编译失败
    CompilationFailed {
        /// 目标平台
        target: TargetPlatform,
        /// 错误消息
        message: String,
    },
    /// 后端未注册
    NoBackendRegistered,
}

impl std::fmt::Display for AotError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AotError::UnsupportedTarget { target } => {
                write!(f, "不支持的目标平台: {}", target.name())
            }
            AotError::CompilationFailed { target, message } => {
                write!(f, "编译失败 ({}): {}", target.name(), message)
            }
            AotError::NoBackendRegistered => {
                write!(f, "未注册任何 AOT 编译后端")
            }
        }
    }
}

impl std::error::Error for AotError {}
