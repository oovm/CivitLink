#![warn(missing_docs)]

//! GG 引擎 AOT 编译器模块
//! 提供 AOT 编译后端，包括 Cranelift 后端实现和目标平台抽象

/// AOT 编译后端 trait 和注册表
pub mod backend;
/// Cranelift AOT 编译后端实现
pub mod cranelift_backend;
/// AOT 编译错误类型
pub mod error;
/// prelude
pub mod prelude;
/// 目标平台定义
pub mod target;

pub use backend::{AotBackend, AotBackendRegistry};
pub use cranelift_backend::CraneliftBackend;
pub use error::AotError;
pub use target::TargetPlatform;
