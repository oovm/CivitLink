//! 日志模块
//!
//! 提供 JSON 结构化日志记录功能。

pub mod init;
pub mod json;

pub use self::{init::*, json::*};
