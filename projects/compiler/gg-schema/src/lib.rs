#![warn(missing_docs)]

//! GG 引擎 Schema DSL 编译器模块
//! 负责将 .schema 文件编译为 GG IR、Valkyrie 类型绑定和数据库迁移文件

pub mod codegen;
pub mod compiler;
pub mod error;
pub mod ir;
pub mod prelude;
pub mod transformer;
pub mod validator;
