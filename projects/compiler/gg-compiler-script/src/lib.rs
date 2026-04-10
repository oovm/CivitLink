#![warn(missing_docs)]

//! GG 统一脚本编译器 crate
//! 提供 .v（Valkyrie 脚本）、.vx（ValkyrieX 单文件组件）、.shader（GG Shader）三种脚本类型的编译功能
//! 三种脚本共享同一套核心编译管线（源码 → AST → IR → 字节码），借助 VM 互通

pub mod prelude;
pub mod shader;
pub mod valkyrie_transformer;
pub mod vx;
