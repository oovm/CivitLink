//! GG 对话系统插件核心类型模块
//! Re-export gg-galgame-schema 的类型，并提供对话插件特有的扩展

pub use pleroma::{components::*, resources::*};

/// GameVariables 扩展 trait
///
/// 为 GameVariables 提供对话插件特有的复合表达式求值功能。
pub trait GameVariablesExt {
    /// 使用表达式求值器评估复合条件
    ///
    /// 支持逻辑运算（AND、OR、NOT）和比较运算的复合表达式。
    fn evaluate_expression(&self, expression: &str) -> bool;
}

impl GameVariablesExt for GameVariables {
    fn evaluate_expression(&self, expression: &str) -> bool {
        crate::expression::ExpressionEvaluator::evaluate(expression, self)
    }
}
