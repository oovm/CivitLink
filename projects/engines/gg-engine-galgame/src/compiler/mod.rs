#![warn(missing_docs)]

//! GG Galgame 对话脚本编译器模块
//! 提供 .script 剧本脚本的编译、增量编译和转换功能

pub mod codegen;
pub mod compiler;
pub mod error;
pub mod incremental;
pub mod ir;
pub mod parser;
pub mod prelude;
pub mod transformer;
