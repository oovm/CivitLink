#![warn(missing_docs)]

//! GG Tools 共享逻辑模块
//! 
//! 提供 gg-tools 命令行工具的共享功能和工具函数

pub mod cmds;
pub mod commands;
pub mod platform;

pub use cmds::*;
pub use commands::*;
pub use gg_core::{GError, GErrorKind, GResult};
pub use gg_runtime_core::{EngineHost, ScriptEngine};
pub use gg_vm::{Vm, VmResult};
