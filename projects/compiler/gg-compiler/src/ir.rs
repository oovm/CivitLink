//! IR 中间表示模块
//! 重新导出 gg-ir 的核心类型，提供统一的中间表示接口

pub use gg_ir::{
    IrFunction, IrModule, IrValue, OpCode, common_subexpr, constant_fold, dead_code, default_optimizer, inline_expand, pass,
};

pub use crate::transformer_adapter;
