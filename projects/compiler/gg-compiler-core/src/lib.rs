#![warn(missing_docs)]

//! GG 引擎编译器核心模块
//! 提供编译流水线架构，包括产物管理、编译上下文、转换器 trait、IR 中间表示和 DAG 调度

pub mod artifact;
pub mod context;
pub mod ir;
pub mod pipeline;
pub mod prelude;
pub mod transformer;
pub mod transformer_adapter;
