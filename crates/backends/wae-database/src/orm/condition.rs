//! 查询条件模块
//!
//! 提供查询条件构建和排序方式定义

use wae_types::Value;

#[cfg(feature = "limbo")]
use crate::types::from_wae_value;
#[cfg(feature = "limbo")]
use limbo::Value as LimboValue;

#[cfg(feature = "mysql")]
use crate::types::from_wae_to_mysql;
#[cfg(feature = "mysql")]
use mysql_async::Value as MySqlValue;

#[cfg(feature = "postgres")]
use crate::connection::postgres::{PostgresParam, value_to_postgres_param};

/// 查询条件
#[derive(Debug, Clone)]
pub enum Condition {
    /// 相等条件
    Eq {
        /// 列名
        column: String,
        /// 值
        value: Value,
    },
    /// 不相等条件
    Ne {
        /// 列名
        column: String,
        /// 值
        value: Value,
    },
    /// 大于条件
    Gt {
        /// 列名
        column: String,
        /// 值
        value: Value,
    },
    /// 大于等于条件
    Gte {
        /// 列名
        column: String,
        /// 值
        value: Value,
    },
    /// 小于条件
    Lt {
        /// 列名
        column: String,
        /// 值
        value: Value,
    },
    /// 小于等于条件
    Lte {
        /// 列名
        column: String,
        /// 值
        value: Value,
    },
    /// LIKE 条件
    Like {
        /// 列名
        column: String,
        /// 模式
        pattern: String,
    },
    /// IN 条件
    In {
        /// 列名
        column: String,
        /// 值列表
        values: Vec<Value>,
    },
    /// IS NULL 条件
    IsNull {
        /// 列名
        column: String,
    },
    /// IS NOT NULL 条件
    IsNotNull {
        /// 列名
        column: String,
    },
    /// AND 组合
    And(Vec<Condition>),
    /// OR 组合
    Or(Vec<Condition>),
    /// NOT 条件
    Not(Box<Condition>),
    /// 原始 SQL 条件
    Raw {
        /// SQL 片段
        sql: String,
        /// 参数
        params: Vec<Value>,
    },
}

impl Condition {
    /// 创建相等条件
    pub fn eq<C: Into<String>, V: Into<Value>>(column: C, value: V) -> Self {
        Condition::Eq { column: column.into(), value: value.into() }
    }

    /// 创建不相等条件
    pub fn ne<C: Into<String>, V: Into<Value>>(column: C, value: V) -> Self {
        Condition::Ne { column: column.into(), value: value.into() }
    }

    /// 创建大于条件
    pub fn gt<C: Into<String>, V: Into<Value>>(column: C, value: V) -> Self {
        Condition::Gt { column: column.into(), value: value.into() }
    }

    /// 创建大于等于条件
    pub fn gte<C: Into<String>, V: Into<Value>>(column: C, value: V) -> Self {
        Condition::Gte { column: column.into(), value: value.into() }
    }

    /// 创建小于条件
    pub fn lt<C: Into<String>, V: Into<Value>>(column: C, value: V) -> Self {
        Condition::Lt { column: column.into(), value: value.into() }
    }

    /// 创建小于等于条件
    pub fn lte<C: Into<String>, V: Into<Value>>(column: C, value: V) -> Self {
        Condition::Lte { column: column.into(), value: value.into() }
    }

    /// 创建 LIKE 条件
    pub fn like<C: Into<String>, P: Into<String>>(column: C, pattern: P) -> Self {
        Condition::Like { column: column.into(), pattern: pattern.into() }
    }

    /// 创建 IN 条件
    pub fn in_<C: Into<String>, V: Into<Value>>(column: C, values: Vec<V>) -> Self {
        Condition::In { column: column.into(), values: values.into_iter().map(|v| v.into()).collect() }
    }

    /// 创建 IS NULL 条件
    pub fn is_null<C: Into<String>>(column: C) -> Self {
        Condition::IsNull { column: column.into() }
    }

    /// 创建 IS NOT NULL 条件
    pub fn is_not_null<C: Into<String>>(column: C) -> Self {
        Condition::IsNotNull { column: column.into() }
    }

    /// 创建 AND 组合
    pub fn and(conditions: Vec<Condition>) -> Self {
        Condition::And(conditions)
    }

    /// 创建 OR 组合
    pub fn or(conditions: Vec<Condition>) -> Self {
        Condition::Or(conditions)
    }

    /// 创建 NOT 条件
    pub fn negate(condition: Condition) -> Self {
        Condition::Not(Box::new(condition))
    }

    /// 创建原始 SQL 条件
    pub fn raw<S: Into<String>>(sql: S, params: Vec<Value>) -> Self {
        Condition::Raw { sql: sql.into(), params }
    }

    #[cfg(feature = "limbo")]
    /// 构建 SQL 和参数 (内部使用 limbo::Value)
    pub(crate) fn build_limbo(&self) -> (String, Vec<LimboValue>) {
        match self {
            Condition::Eq { column, value } => (format!("{} = ?", column), vec![from_wae_value(value.clone())]),
            Condition::Ne { column, value } => (format!("{} != ?", column), vec![from_wae_value(value.clone())]),
            Condition::Gt { column, value } => (format!("{} > ?", column), vec![from_wae_value(value.clone())]),
            Condition::Gte { column, value } => (format!("{} >= ?", column), vec![from_wae_value(value.clone())]),
            Condition::Lt { column, value } => (format!("{} < ?", column), vec![from_wae_value(value.clone())]),
            Condition::Lte { column, value } => (format!("{} <= ?", column), vec![from_wae_value(value.clone())]),
            Condition::Like { column, pattern } => (format!("{} LIKE ?", column), vec![LimboValue::Text(pattern.clone())]),
            Condition::In { column, values } => {
                let placeholders: Vec<&str> = values.iter().map(|_| "?").collect();
                let limbo_values: Vec<LimboValue> = values.iter().map(|v| from_wae_value(v.clone())).collect();
                (format!("{} IN ({})", column, placeholders.join(", ")), limbo_values)
            }
            Condition::IsNull { column } => (format!("{} IS NULL", column), vec![]),
            Condition::IsNotNull { column } => (format!("{} IS NOT NULL", column), vec![]),
            Condition::And(conditions) => {
                let mut sql_parts = Vec::new();
                let mut all_params = Vec::new();
                for cond in conditions {
                    let (sql, params) = cond.build_limbo();
                    sql_parts.push(format!("({})", sql));
                    all_params.extend(params);
                }
                (sql_parts.join(" AND "), all_params)
            }
            Condition::Or(conditions) => {
                let mut sql_parts = Vec::new();
                let mut all_params = Vec::new();
                for cond in conditions {
                    let (sql, params) = cond.build_limbo();
                    sql_parts.push(format!("({})", sql));
                    all_params.extend(params);
                }
                (sql_parts.join(" OR "), all_params)
            }
            Condition::Not(cond) => {
                let (sql, params) = cond.build_limbo();
                (format!("NOT ({})", sql), params)
            }
            Condition::Raw { sql, params } => {
                let limbo_params: Vec<LimboValue> = params.iter().map(|v| from_wae_value(v.clone())).collect();
                (sql.clone(), limbo_params)
            }
        }
    }

    #[cfg(feature = "mysql")]
    /// 构建 SQL 和参数 (内部使用 MySQL Value)
    pub(crate) fn build_mysql(&self) -> (String, Vec<MySqlValue>) {
        match self {
            Condition::Eq { column, value } => (format!("{} = ?", column), vec![from_wae_to_mysql(value.clone())]),
            Condition::Ne { column, value } => (format!("{} != ?", column), vec![from_wae_to_mysql(value.clone())]),
            Condition::Gt { column, value } => (format!("{} > ?", column), vec![from_wae_to_mysql(value.clone())]),
            Condition::Gte { column, value } => (format!("{} >= ?", column), vec![from_wae_to_mysql(value.clone())]),
            Condition::Lt { column, value } => (format!("{} < ?", column), vec![from_wae_to_mysql(value.clone())]),
            Condition::Lte { column, value } => (format!("{} <= ?", column), vec![from_wae_to_mysql(value.clone())]),
            Condition::Like { column, pattern } => {
                (format!("{} LIKE ?", column), vec![MySqlValue::Bytes(pattern.clone().into_bytes())])
            }
            Condition::In { column, values } => {
                let placeholders: Vec<&str> = values.iter().map(|_| "?").collect();
                let mysql_values: Vec<MySqlValue> = values.iter().map(|v| from_wae_to_mysql(v.clone())).collect();
                (format!("{} IN ({})", column, placeholders.join(", ")), mysql_values)
            }
            Condition::IsNull { column } => (format!("{} IS NULL", column), vec![]),
            Condition::IsNotNull { column } => (format!("{} IS NOT NULL", column), vec![]),
            Condition::And(conditions) => {
                let mut sql_parts = Vec::new();
                let mut all_params = Vec::new();
                for cond in conditions {
                    let (sql, params) = cond.build_mysql();
                    sql_parts.push(format!("({})", sql));
                    all_params.extend(params);
                }
                (sql_parts.join(" AND "), all_params)
            }
            Condition::Or(conditions) => {
                let mut sql_parts = Vec::new();
                let mut all_params = Vec::new();
                for cond in conditions {
                    let (sql, params) = cond.build_mysql();
                    sql_parts.push(format!("({})", sql));
                    all_params.extend(params);
                }
                (sql_parts.join(" OR "), all_params)
            }
            Condition::Not(cond) => {
                let (sql, params) = cond.build_mysql();
                (format!("NOT ({})", sql), params)
            }
            Condition::Raw { sql, params } => {
                let mysql_params: Vec<MySqlValue> = params.iter().map(|v| from_wae_to_mysql(v.clone())).collect();
                (sql.clone(), mysql_params)
            }
        }
    }

    #[cfg(feature = "postgres")]
    /// 构建 SQL 和参数 (内部使用 PostgresParam)
    pub(crate) fn build_postgres(&self) -> (String, Vec<PostgresParam>) {
        let param_index = 1;
        match self {
            Condition::Eq { column, value } => {
                let sql = format!("{} = ${}", column, param_index);
                let params = vec![value_to_postgres_param(value.clone())];
                (sql, params)
            }
            Condition::Ne { column, value } => {
                let sql = format!("{} != ${}", column, param_index);
                let params = vec![value_to_postgres_param(value.clone())];
                (sql, params)
            }
            Condition::Gt { column, value } => {
                let sql = format!("{} > ${}", column, param_index);
                let params = vec![value_to_postgres_param(value.clone())];
                (sql, params)
            }
            Condition::Gte { column, value } => {
                let sql = format!("{} >= ${}", column, param_index);
                let params = vec![value_to_postgres_param(value.clone())];
                (sql, params)
            }
            Condition::Lt { column, value } => {
                let sql = format!("{} < ${}", column, param_index);
                let params = vec![value_to_postgres_param(value.clone())];
                (sql, params)
            }
            Condition::Lte { column, value } => {
                let sql = format!("{} <= ${}", column, param_index);
                let params = vec![value_to_postgres_param(value.clone())];
                (sql, params)
            }
            Condition::Like { column, pattern } => {
                let sql = format!("{} LIKE ${}", column, param_index);
                let params = vec![value_to_postgres_param(Value::String(pattern.clone()))];
                (sql, params)
            }
            Condition::In { column, values } => {
                let placeholders: Vec<String> = (0..values.len()).map(|i| format!("${}", param_index + i)).collect();
                let postgres_params: Vec<PostgresParam> = values.iter().map(|v| value_to_postgres_param(v.clone())).collect();
                (format!("{} IN ({})", column, placeholders.join(", ")), postgres_params)
            }
            Condition::IsNull { column } => (format!("{} IS NULL", column), vec![]),
            Condition::IsNotNull { column } => (format!("{} IS NOT NULL", column), vec![]),
            Condition::And(conditions) => {
                let mut sql_parts = Vec::new();
                let mut all_params = Vec::new();
                let mut current_index = 1;
                for cond in conditions {
                    let (sql, params) = cond.build_postgres();
                    let sql = replace_placeholders(&sql, current_index);
                    sql_parts.push(format!("({})", sql));
                    let params_len = params.len();
                    all_params.extend(params);
                    current_index += params_len;
                }
                (sql_parts.join(" AND "), all_params)
            }
            Condition::Or(conditions) => {
                let mut sql_parts = Vec::new();
                let mut all_params = Vec::new();
                let mut current_index = 1;
                for cond in conditions {
                    let (sql, params) = cond.build_postgres();
                    let sql = replace_placeholders(&sql, current_index);
                    sql_parts.push(format!("({})", sql));
                    let params_len = params.len();
                    all_params.extend(params);
                    current_index += params_len;
                }
                (sql_parts.join(" OR "), all_params)
            }
            Condition::Not(cond) => {
                let (sql, params) = cond.build_postgres();
                (format!("NOT ({})", sql), params)
            }
            Condition::Raw { sql, params } => {
                let postgres_params: Vec<PostgresParam> = params.iter().map(|v| value_to_postgres_param(v.clone())).collect();
                (sql.clone(), postgres_params)
            }
        }
    }
}

#[cfg(feature = "postgres")]
fn replace_placeholders(sql: &str, start_index: usize) -> String {
    let mut result = String::new();
    let mut chars = sql.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '$' {
            let mut num_str = String::new();
            while let Some(&next_c) = chars.peek() {
                if next_c.is_ascii_digit() {
                    num_str.push(next_c);
                    chars.next();
                }
                else {
                    break;
                }
            }
            if !num_str.is_empty() {
                if let Ok(num) = num_str.parse::<usize>() {
                    result.push_str(&format!("${}", num + start_index - 1));
                }
            }
            else {
                result.push(c);
            }
        }
        else {
            result.push(c);
        }
    }
    result
}

/// 排序方式
#[derive(Debug, Clone, Copy)]
pub enum Order {
    /// 升序
    Asc,
    /// 降序
    Desc,
}
