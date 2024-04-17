#![warn(missing_docs)]

//! GG 引擎 Widget 编译器模块
//! 负责将 .widget 文件（Editor UI 单文件组件）编译为可运行时加载的 Widget 产物

pub mod artifact;
pub mod compiler;
pub mod error;
pub mod parser;
pub mod prelude;
pub mod registry;
pub mod style;
pub mod template;
