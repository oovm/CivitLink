//! 条件表达式求值器模块
//! 提供支持逻辑运算和比较运算的条件表达式解析和求值

use crate::schema::GameVariables;

/// 条件表达式求值器
///
/// 支持以下运算：
/// - 比较运算：>=, <=, !=, ==, >, <
/// - 逻辑运算：AND, OR, NOT
/// - 括号分组：(, )
///
/// 运算优先级（从低到高）：OR → AND → NOT → 比较运算
pub struct ExpressionEvaluator;

impl ExpressionEvaluator {
    /// 求值条件表达式
    ///
    /// 支持复合条件，如 "affection >= 5 AND chapter == 2"。
    /// 空表达式返回 true。
    pub fn evaluate(expression: &str, variables: &GameVariables) -> bool {
        let expr = expression.trim();
        if expr.is_empty() {
            return true;
        }
        Self::evaluate_or(expr, variables)
    }

    /// 处理 OR 运算（最低优先级）
    fn evaluate_or(expr: &str, variables: &GameVariables) -> bool {
        let mut depth = 0usize;

        for (i, ch) in expr.char_indices() {
            match ch {
                '(' => depth += 1,
                ')' => depth -= 1,
                _ => {}
            }
            if depth == 0 && expr[i..].starts_with(" OR ") {
                let left = &expr[..i];
                let right = &expr[i + 4..];
                return Self::evaluate_or(left, variables)
                    || Self::evaluate_and(right, variables);
            }
        }
        Self::evaluate_and(expr, variables)
    }

    /// 处理 AND 运算
    fn evaluate_and(expr: &str, variables: &GameVariables) -> bool {
        let mut depth = 0usize;

        for (i, ch) in expr.char_indices() {
            match ch {
                '(' => depth += 1,
                ')' => depth -= 1,
                _ => {}
            }
            if depth == 0 && expr[i..].starts_with(" AND ") {
                let left = &expr[..i];
                let right = &expr[i + 5..];
                return Self::evaluate_and(left, variables)
                    && Self::evaluate_not(right, variables);
            }
        }
        Self::evaluate_not(expr, variables)
    }

    /// 处理 NOT 运算
    fn evaluate_not(expr: &str, variables: &GameVariables) -> bool {
        let trimmed = expr.trim();
        if trimmed.starts_with("NOT ") {
            let inner = &trimmed[4..];
            return !Self::evaluate_not(inner, variables);
        }
        if trimmed.starts_with("not ") {
            let inner = &trimmed[4..];
            return !Self::evaluate_not(inner, variables);
        }
        Self::evaluate_atom(trimmed, variables)
    }

    /// 处理原子表达式（比较运算或括号分组）
    fn evaluate_atom(expr: &str, variables: &GameVariables) -> bool {
        let trimmed = expr.trim();

        if trimmed.starts_with('(') && trimmed.ends_with(')') {
            let inner = &trimmed[1..trimmed.len() - 1];
            return Self::evaluate_or(inner, variables);
        }

        variables.evaluate_condition(trimmed)
    }
}
