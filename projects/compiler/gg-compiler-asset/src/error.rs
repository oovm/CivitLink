//! 资产管线错误类型模块
//! 定义统一的资产管线错误类型，覆盖注册、依赖、编译、缓存等各环节

use std::fmt;

use crate::types::Guid;

/// 资产管线错误类型
#[derive(Debug)]
pub enum AssetPipelineError {
    /// 资产注册失败
    RegistrationFailed {
        /// 资产 GUID
        guid: Guid,
        /// 失败原因
        reason: String,
    },
    /// 循环依赖检测
    CircularDependency {
        /// 循环路径
        cycle_path: Vec<Guid>,
    },
    /// 依赖边不存在
    DependencyNotFound {
        /// 源资产 GUID
        source: Guid,
        /// 目标资产 GUID
        target: Guid,
    },
    /// 编译失败
    CompileFailed {
        /// 资产 GUID
        guid: Guid,
        /// 失败原因
        reason: String,
    },
    /// 缓存失效
    CacheInvalidated {
        /// 被失效的资产 GUID 列表
        guids: Vec<Guid>,
    },
    /// 磁盘 IO 错误
    IoError {
        /// 错误描述
        message: String,
    },
    /// 序列化/反序列化错误
    SerializeError {
        /// 错误描述
        message: String,
    },
}

impl fmt::Display for AssetPipelineError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AssetPipelineError::RegistrationFailed { guid, reason } => {
                write!(f, "资产注册失败 [{}]: {}", guid, reason)
            }
            AssetPipelineError::CircularDependency { cycle_path } => {
                let path_display = cycle_path.join(" -> ");
                write!(f, "检测到循环依赖: {}", path_display)
            }
            AssetPipelineError::DependencyNotFound { source, target } => {
                write!(f, "依赖边不存在: {} -> {}", source, target)
            }
            AssetPipelineError::CompileFailed { guid, reason } => {
                write!(f, "资产编译失败 [{}]: {}", guid, reason)
            }
            AssetPipelineError::CacheInvalidated { guids } => {
                write!(f, "缓存失效，受影响资产数: {}", guids.len())
            }
            AssetPipelineError::IoError { message } => {
                write!(f, "磁盘 IO 错误: {}", message)
            }
            AssetPipelineError::SerializeError { message } => {
                write!(f, "序列化/反序列化错误: {}", message)
            }
        }
    }
}

impl std::error::Error for AssetPipelineError {}

impl From<std::io::Error> for AssetPipelineError {
    fn from(err: std::io::Error) -> Self {
        AssetPipelineError::IoError { message: err.to_string() }
    }
}

impl From<serde_json::Error> for AssetPipelineError {
    fn from(err: serde_json::Error) -> Self {
        AssetPipelineError::SerializeError { message: err.to_string() }
    }
}
