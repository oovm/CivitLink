#![warn(missing_docs)]

//! GG 引擎 AOT 编译器模块
//! 提供 AOT 编译后端骨架，包括 trait 定义和目标平台抽象

pub mod backend;
pub mod error;
pub mod target;

pub use backend::{AotBackend, AotBackendRegistry};
pub use error::AotError;
pub use target::TargetPlatform;
