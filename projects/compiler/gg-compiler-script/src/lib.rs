#![warn(missing_docs)]

//! GG 剧本编译转换器 crate
//! 提供 .gscript 剧本脚本的解析、编译、增量编译、Valkyrie 脚本转换和 prelude

pub mod compiler;
pub mod incremental;
pub mod parser;
pub mod prelude;
pub mod transformer;
pub mod valkyrie_transformer;
