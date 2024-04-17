#![warn(missing_docs)]

//! GG 引擎 Galgame MDX 编译器模块
//! 负责将 .galgame 文件（基于 MDX 格式）编译为可执行字节码

pub mod codegen;
pub mod compiler;
pub mod error;
pub mod ir;
pub mod parser;
pub mod prelude;
pub mod transformer;
