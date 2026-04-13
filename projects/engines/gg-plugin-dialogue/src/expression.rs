//! 条件表达式求值器模块
//! 提供支持逻辑运算、比较运算和算术运算的条件表达式解析和求值

use crate::schema::GameVariables;

/// 算术值
#[derive(Debug, Clone)]
enum ArithmeticValue {
    /// 整数值
    Integer(i64),
    /// 浮点数值
    Float(f64),
}

impl ArithmeticValue {
    /// 加法运算
    fn add(&self, other: &ArithmeticValue) -> ArithmeticValue {
        match (self, other) {
            (ArithmeticValue::Integer(a), ArithmeticValue::Integer(b)) => ArithmeticValue::Integer(a + b),
            _ => ArithmeticValue::Float(self.as_f64() + other.as_f64()),
        }
    }

    /// 减法运算
    fn sub(&self, other: &ArithmeticValue) -> ArithmeticValue {
        match (self, other) {
            (ArithmeticValue::Integer(a), ArithmeticValue::Integer(b)) => ArithmeticValue::Integer(a - b),
            _ => ArithmeticValue::Float(self.as_f64() - other.as_f64()),
        }
    }

    /// 乘法运算
    fn mul(&self, other: &ArithmeticValue) -> ArithmeticValue {
        match (self, other) {
            (ArithmeticValue::Integer(a), ArithmeticValue::Integer(b)) => ArithmeticValue::Integer(a * b),
            _ => ArithmeticValue::Float(self.as_f64() * other.as_f64()),
        }
    }

    /// 除法运算
    fn div(&self, other: &ArithmeticValue) -> ArithmeticValue {
        ArithmeticValue::Float(self.as_f64() / other.as_f64())
    }

    /// 转换为 f64
    fn as_f64(&self) -> f64 {
        match self {
            ArithmeticValue::Integer(i) => *i as f64,
            ArithmeticValue::Float(f) => *f,
        }
    }
}

/// 条件表达式求值器
///
/// 支持以下运算：
/// - 比较运算：>=, <=, !=, ==, >, <
/// - 逻辑运算：AND, OR, NOT
/// - 算术运算：+, -, *, /
/// - 括号分组：(, )
/// - 变量间比较
///
/// 运算优先级（从低到高）：OR → AND → NOT → 比较运算 → 算术运算
pub struct ExpressionEvaluator;

impl ExpressionEvaluator {
    /// 求值条件表达式
    ///
    /// 支持复合条件，如 "affection >= 5 AND chapter == 2"。
    /// 支持算术表达式，如 "affection + 1 >= 5"。
    /// 支持变量间比较，如 "sakura_affection > ren_affection"。
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
                return Self::evaluate_or(left, variables) || Self::evaluate_and(right, variables);
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
                return Self::evaluate_and(left, variables) && Self::evaluate_not(right, variables);
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

        Self::evaluate_comparison(trimmed, variables)
    }

    /// 处理比较表达式（含算术运算）
    fn evaluate_comparison(expr: &str, variables: &GameVariables) -> bool {
        let trimmed = expr.trim();

        for op in &["==", "!=", ">=", "<=", ">", "<"] {
            if let Some(pos) = Self::find_operator_outside_parens(trimmed, op) {
                let left = trimmed[..pos].trim();
                let right = trimmed[pos + op.len()..].trim();
                let left_val = Self::evaluate_arithmetic(left, variables);
                let right_val = Self::evaluate_arithmetic(right, variables);
                return Self::compare_values(&left_val, &right_val, op);
            }
        }

        variables.evaluate_condition(trimmed)
    }

    /// 查找不在括号内的运算符位置
    fn find_operator_outside_parens(expr: &str, op: &str) -> Option<usize> {
        let mut depth = 0usize;
        let chars: Vec<char> = expr.chars().collect();
        let op_chars: Vec<char> = op.chars().collect();

        for i in 0..chars.len() {
            if chars[i] == '(' {
                depth += 1;
            }
            else if chars[i] == ')' {
                depth -= 1;
            }
            else if depth == 0 && i + op_chars.len() <= chars.len() {
                let slice: String = chars[i..i + op_chars.len()].iter().collect();
                if slice == op {
                    return Some(i);
                }
            }
        }
        None
    }

    /// 计算算术表达式的值
    fn evaluate_arithmetic(expr: &str, variables: &GameVariables) -> ArithmeticValue {
        let trimmed = expr.trim();

        if let Some(pos) = Self::find_arithmetic_op(trimmed, &['+', '-']) {
            let left = &trimmed[..pos];
            let right = &trimmed[pos + 1..];
            let op_char = trimmed.chars().nth(pos).unwrap();
            let left_val = Self::evaluate_arithmetic(left, variables);
            let right_val = Self::evaluate_arithmetic(right, variables);
            return match op_char {
                '+' => left_val.add(&right_val),
                '-' => left_val.sub(&right_val),
                _ => ArithmeticValue::Float(0.0),
            };
        }

        if let Some(pos) = Self::find_arithmetic_op(trimmed, &['*', '/']) {
            let left = &trimmed[..pos];
            let right = &trimmed[pos + 1..];
            let op_char = trimmed.chars().nth(pos).unwrap();
            let left_val = Self::evaluate_arithmetic(left, variables);
            let right_val = Self::evaluate_arithmetic(right, variables);
            return match op_char {
                '*' => left_val.mul(&right_val),
                '/' => left_val.div(&right_val),
                _ => ArithmeticValue::Float(0.0),
            };
        }

        if let Ok(i) = trimmed.parse::<i64>() {
            return ArithmeticValue::Integer(i);
        }
        if let Ok(f) = trimmed.parse::<f64>() {
            return ArithmeticValue::Float(f);
        }

        if let Some(val) = variables.get_variable(trimmed) {
            return match val {
                gg_engine::VariableValue::Integer(i) => ArithmeticValue::Integer(i),
                gg_engine::VariableValue::Float(f) => ArithmeticValue::Float(f),
                _ => ArithmeticValue::Float(0.0),
            };
        }

        ArithmeticValue::Float(0.0)
    }

    /// 查找不在括号内的算术运算符位置（从右向左搜索以支持左结合）
    fn find_arithmetic_op(expr: &str, ops: &[char]) -> Option<usize> {
        let mut depth = 0usize;
        let chars: Vec<char> = expr.chars().collect();

        for i in (0..chars.len()).rev() {
            if chars[i] == ')' {
                depth += 1;
            }
            else if chars[i] == '(' {
                depth -= 1;
            }
            else if depth == 0 && ops.contains(&chars[i]) {
                if chars[i] == '-' && (i == 0 || "+-*/(".contains(chars[i - 1])) {
                    continue;
                }
                return Some(i);
            }
        }
        None
    }

    /// 比较两个算术值
    fn compare_values(left: &ArithmeticValue, right: &ArithmeticValue, op: &str) -> bool {
        let (l, r) = match (left, right) {
            (ArithmeticValue::Integer(l), ArithmeticValue::Integer(r)) => (*l as f64, *r as f64),
            (ArithmeticValue::Float(l), ArithmeticValue::Integer(r)) => (*l, *r as f64),
            (ArithmeticValue::Integer(l), ArithmeticValue::Float(r)) => (*l as f64, *r),
            (ArithmeticValue::Float(l), ArithmeticValue::Float(r)) => (*l, *r),
        };
        match op {
            ">=" => l >= r,
            "<=" => l <= r,
            ">" => l > r,
            "<" => l < r,
            "==" => (l - r).abs() < f64::EPSILON,
            "!=" => (l - r).abs() >= f64::EPSILON,
            _ => false,
        }
    }
}
