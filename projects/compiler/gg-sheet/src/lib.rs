#![warn(missing_docs)]

//! GG 引擎配置表编译模块
//! 将 Excel/CSV/TSV 配置表编译为 Valkyrie 脚本代码

pub mod cli;
pub mod codegen;
pub mod compiler;
pub mod config;
pub mod error;
pub mod merge;
pub mod reader;
pub mod rust_codegen;
pub mod schema;
pub mod types;
pub mod validate;
pub mod watch;
