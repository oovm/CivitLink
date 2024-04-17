//! GG AOT 编译器 prelude 模块
//! 导出最常用的 AOT 编译器类型

pub use crate::{
    backend::{AotBackend, AotBackendRegistry},
    cranelift_backend::CraneliftBackend,
    error::AotError,
    target::TargetPlatform,
};
