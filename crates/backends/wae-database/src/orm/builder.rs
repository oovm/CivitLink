//! 查询构建器模块
//!
//! 提供 SELECT、INSERT、UPDATE、DELETE 查询构建器

use super::{
    condition::{Condition, Order},
    entity::{Entity, ToRow},
};
use std::marker::PhantomData;
use wae_types::Value;

#[cfg(feature = "limbo")]
use crate::types::from_wae_value;
#[cfg(feature = "limbo")]
use limbo::Value as LimboValue;

#[cfg(feature = "mysql")]
use crate::types::from_wae_to_mysql;
#[cfg(feature = "mysql")]
use mysql_async::Value as MySqlValue;

/// JOIN 类型
#[derive(Debug, Clone, Copy)]
pub enum JoinType {
    /// 内连接
    Inner,
    /// 左连接
    Left,
    /// 右连接
    Right,
    /// 全连接
    Full,
}

/// JOIN 定义
#[derive(Debug, Clone)]
pub struct Join {
    /// JOIN 类型
    join_type: JoinType,
    /// 表名
    table: String,
    /// ON 条件
    on: Condition,
}

impl Join {
    /// 创建新的 JOIN
    pub fn new(join_type: JoinType, table: String, on: Condition) -> Self {
        Self { join_type, table, on }
    }
}

/// SELECT 查询构建器
pub struct SelectBuilder<E: Entity> {
    columns: Vec<String>,
    joins: Vec<Join>,
    conditions: Vec<Condition>,
    group_by: Vec<String>,
    having: Vec<Condition>,
    order_by: Vec<(String, Order)>,
    limit: Option<u64>,
    offset: Option<u64>,
    _marker: PhantomData<E>,
}

impl<E: Entity> SelectBuilder<E> {
    pub(crate) fn new() -> Self {
        Self {
            columns: Vec::new(),
            joins: Vec::new(),
            conditions: Vec::new(),
            group_by: Vec::new(),
            having: Vec::new(),
            order_by: Vec::new(),
            limit: None,
            offset: None,
            _marker: PhantomData,
        }
    }

    /// 选择指定列
    pub fn columns(mut self, columns: &[&str]) -> Self {
        self.columns = columns.iter().map(|s| s.to_string()).collect();
        self
    }

    /// 添加 JOIN
    pub fn join(mut self, join_type: JoinType, table: &str, on: Condition) -> Self {
        self.joins.push(Join::new(join_type, table.to_string(), on));
        self
    }

    /// 添加 INNER JOIN
    pub fn inner_join(mut self, table: &str, on: Condition) -> Self {
        self.joins.push(Join::new(JoinType::Inner, table.to_string(), on));
        self
    }

    /// 添加 LEFT JOIN
    pub fn left_join(mut self, table: &str, on: Condition) -> Self {
        self.joins.push(Join::new(JoinType::Left, table.to_string(), on));
        self
    }

    /// 添加 RIGHT JOIN
    pub fn right_join(mut self, table: &str, on: Condition) -> Self {
        self.joins.push(Join::new(JoinType::Right, table.to_string(), on));
        self
    }

    /// 添加 FULL JOIN
    pub fn full_join(mut self, table: &str, on: Condition) -> Self {
        self.joins.push(Join::new(JoinType::Full, table.to_string(), on));
        self
    }

    /// 添加 WHERE 条件
    pub fn where_(mut self, condition: Condition) -> Self {
        self.conditions.push(condition);
        self
    }

    /// 添加多个 WHERE 条件 (AND 连接)
    pub fn where_all(mut self, conditions: Vec<Condition>) -> Self {
        self.conditions.extend(conditions);
        self
    }

    /// 添加 GROUP BY
    pub fn group_by(mut self, columns: &[&str]) -> Self {
        self.group_by = columns.iter().map(|s| s.to_string()).collect();
        self
    }

    /// 添加 HAVING 条件
    pub fn having(mut self, condition: Condition) -> Self {
        self.having.push(condition);
        self
    }

    /// 添加多个 HAVING 条件 (AND 连接)
    pub fn having_all(mut self, conditions: Vec<Condition>) -> Self {
        self.having.extend(conditions);
        self
    }

    /// 添加排序
    pub fn order_by<C: Into<String>>(mut self, column: C, order: Order) -> Self {
        self.order_by.push((column.into(), order));
        self
    }

    /// 设置 LIMIT
    pub fn limit(mut self, limit: u64) -> Self {
        self.limit = Some(limit);
        self
    }

    /// 设置 OFFSET
    pub fn offset(mut self, offset: u64) -> Self {
        self.offset = Some(offset);
        self
    }

    #[cfg(feature = "limbo")]
    /// 构建 SQL 和参数 (内部使用)
    pub(crate) fn build_limbo(&self) -> (String, Vec<LimboValue>) {
        let columns = if self.columns.is_empty() { "*".to_string() } else { self.columns.join(", ") };

        let mut sql = format!("SELECT {} FROM {}", columns, E::table_name());
        let mut params = Vec::new();

        for join in &self.joins {
            let (on_sql, on_params) = join.on.build_limbo();
            let join_type_str = match join.join_type {
                JoinType::Inner => "INNER JOIN",
                JoinType::Left => "LEFT JOIN",
                JoinType::Right => "RIGHT JOIN",
                JoinType::Full => "FULL JOIN",
            };
            sql.push_str(&format!(" {} {} ON {}", join_type_str, join.table, on_sql));
            params.extend(on_params);
        }

        if !self.conditions.is_empty() {
            let where_conditions = self.conditions.to_vec();
            if where_conditions.len() == 1 {
                let (cond_sql, cond_params) = where_conditions[0].build_limbo();
                sql.push_str(&format!(" WHERE {}", cond_sql));
                params.extend(cond_params);
            }
            else {
                let (cond_sql, cond_params) = Condition::and(where_conditions).build_limbo();
                sql.push_str(&format!(" WHERE {}", cond_sql));
                params.extend(cond_params);
            }
        }

        if !self.group_by.is_empty() {
            sql.push_str(&format!(" GROUP BY {}", self.group_by.join(", ")));
        }

        if !self.having.is_empty() {
            let having_conditions = self.having.to_vec();
            if having_conditions.len() == 1 {
                let (cond_sql, cond_params) = having_conditions[0].build_limbo();
                sql.push_str(&format!(" HAVING {}", cond_sql));
                params.extend(cond_params);
            }
            else {
                let (cond_sql, cond_params) = Condition::and(having_conditions).build_limbo();
                sql.push_str(&format!(" HAVING {}", cond_sql));
                params.extend(cond_params);
            }
        }

        if !self.order_by.is_empty() {
            let order_parts: Vec<String> = self
                .order_by
                .iter()
                .map(|(col, order)| {
                    format!(
                        "{} {}",
                        col,
                        match order {
                            Order::Asc => "ASC",
                            Order::Desc => "DESC",
                        }
                    )
                })
                .collect();
            sql.push_str(&format!(" ORDER BY {}", order_parts.join(", ")));
        }

        if let Some(limit) = self.limit {
            sql.push_str(&format!(" LIMIT {}", limit));
        }

        if let Some(offset) = self.offset {
            sql.push_str(&format!(" OFFSET {}", offset));
        }

        (sql, params)
    }

    #[cfg(feature = "mysql")]
    #[allow(dead_code)]
    /// 构建 SQL 和参数 (内部使用 MySQL)
    pub(crate) fn build_mysql(&self) -> (String, Vec<MySqlValue>) {
        let columns = if self.columns.is_empty() { "*".to_string() } else { self.columns.join(", ") };

        let mut sql = format!("SELECT {} FROM {}", columns, E::table_name());
        let mut params = Vec::new();

        for join in &self.joins {
            let (on_sql, on_params) = join.on.build_mysql();
            let join_type_str = match join.join_type {
                JoinType::Inner => "INNER JOIN",
                JoinType::Left => "LEFT JOIN",
                JoinType::Right => "RIGHT JOIN",
                JoinType::Full => "FULL JOIN",
            };
            sql.push_str(&format!(" {} {} ON {}", join_type_str, join.table, on_sql));
            params.extend(on_params);
        }

        if !self.conditions.is_empty() {
            let where_conditions = self.conditions.to_vec();
            if where_conditions.len() == 1 {
                let (cond_sql, cond_params) = where_conditions[0].build_mysql();
                sql.push_str(&format!(" WHERE {}", cond_sql));
                params.extend(cond_params);
            }
            else {
                let (cond_sql, cond_params) = Condition::and(where_conditions).build_mysql();
                sql.push_str(&format!(" WHERE {}", cond_sql));
                params.extend(cond_params);
            }
        }

        if !self.group_by.is_empty() {
            sql.push_str(&format!(" GROUP BY {}", self.group_by.join(", ")));
        }

        if !self.having.is_empty() {
            let having_conditions = self.having.to_vec();
            if having_conditions.len() == 1 {
                let (cond_sql, cond_params) = having_conditions[0].build_mysql();
                sql.push_str(&format!(" HAVING {}", cond_sql));
                params.extend(cond_params);
            }
            else {
                let (cond_sql, cond_params) = Condition::and(having_conditions).build_mysql();
                sql.push_str(&format!(" HAVING {}", cond_sql));
                params.extend(cond_params);
            }
        }

        if !self.order_by.is_empty() {
            let order_parts: Vec<String> = self
                .order_by
                .iter()
                .map(|(col, order)| {
                    format!(
                        "{} {}",
                        col,
                        match order {
                            Order::Asc => "ASC",
                            Order::Desc => "DESC",
                        }
                    )
                })
                .collect();
            sql.push_str(&format!(" ORDER BY {}", order_parts.join(", ")));
        }

        if let Some(limit) = self.limit {
            sql.push_str(&format!(" LIMIT {}", limit));
        }

        if let Some(offset) = self.offset {
            sql.push_str(&format!(" OFFSET {}", offset));
        }

        (sql, params)
    }

    #[cfg(feature = "postgres")]
    /// 构建 SQL 和参数 (内部使用 PostgreSQL)
    pub(crate) fn build_postgres(&self) -> (String, Vec<crate::connection::postgres::PostgresParam>) {
        let columns = if self.columns.is_empty() { "*".to_string() } else { self.columns.join(", ") };

        let mut sql = format!("SELECT {} FROM {}", columns, E::table_name());
        let mut params = Vec::new();
        let mut param_offset = 0;

        for join in &self.joins {
            let (on_sql, on_params) = join.on.build_postgres();
            let on_sql = replace_placeholders_for_query(&on_sql, param_offset + 1);
            let join_type_str = match join.join_type {
                JoinType::Inner => "INNER JOIN",
                JoinType::Left => "LEFT JOIN",
                JoinType::Right => "RIGHT JOIN",
                JoinType::Full => "FULL JOIN",
            };
            sql.push_str(&format!(" {} {} ON {}", join_type_str, join.table, on_sql));
            let on_params_len = on_params.len();
            params.extend(on_params);
            param_offset += on_params_len;
        }

        if !self.conditions.is_empty() {
            let where_conditions = self.conditions.to_vec();
            let (cond_sql, cond_params) = if where_conditions.len() == 1 {
                where_conditions[0].build_postgres()
            }
            else {
                Condition::and(where_conditions).build_postgres()
            };
            let cond_sql = replace_placeholders_for_query(&cond_sql, param_offset + 1);
            sql.push_str(&format!(" WHERE {}", cond_sql));
            let cond_params_len = cond_params.len();
            params.extend(cond_params);
            param_offset += cond_params_len;
        }

        if !self.group_by.is_empty() {
            sql.push_str(&format!(" GROUP BY {}", self.group_by.join(", ")));
        }

        if !self.having.is_empty() {
            let having_conditions = self.having.to_vec();
            let (cond_sql, cond_params) = if having_conditions.len() == 1 {
                having_conditions[0].build_postgres()
            }
            else {
                Condition::and(having_conditions).build_postgres()
            };
            let cond_sql = replace_placeholders_for_query(&cond_sql, param_offset + 1);
            sql.push_str(&format!(" HAVING {}", cond_sql));
            params.extend(cond_params);
        }

        if !self.order_by.is_empty() {
            let order_parts: Vec<String> = self
                .order_by
                .iter()
                .map(|(col, order)| {
                    format!(
                        "{} {}",
                        col,
                        match order {
                            Order::Asc => "ASC",
                            Order::Desc => "DESC",
                        }
                    )
                })
                .collect();
            sql.push_str(&format!(" ORDER BY {}", order_parts.join(", ")));
        }

        if let Some(limit) = self.limit {
            sql.push_str(&format!(" LIMIT {}", limit));
        }

        if let Some(offset) = self.offset {
            sql.push_str(&format!(" OFFSET {}", offset));
        }

        (sql, params)
    }
}

#[cfg(feature = "postgres")]
fn replace_placeholders_for_query(sql: &str, start_index: usize) -> String {
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

/// INSERT 查询构建器
pub struct InsertBuilder<E: Entity> {
    data: Vec<(&'static str, Value)>,
    _marker: PhantomData<E>,
}

impl<E: Entity> InsertBuilder<E> {
    pub(crate) fn new() -> Self {
        Self { data: Vec::new(), _marker: PhantomData }
    }

    /// 从实体创建
    pub fn from_entity<T: ToRow>(entity: &T) -> Self {
        Self { data: entity.to_row(), _marker: PhantomData }
    }

    /// 添加列值
    pub fn value(mut self, column: &'static str, value: Value) -> Self {
        self.data.push((column, value));
        self
    }

    /// 批量添加列值
    pub fn values(mut self, data: Vec<(&'static str, Value)>) -> Self {
        self.data.extend(data);
        self
    }

    #[cfg(feature = "limbo")]
    /// 构建 SQL 和参数 (内部使用)
    pub(crate) fn build_limbo(&self) -> (String, Vec<LimboValue>) {
        let columns: Vec<&str> = self.data.iter().map(|(col, _)| *col).collect();
        let placeholders: Vec<&str> = self.data.iter().map(|_| "?").collect();
        let params: Vec<LimboValue> = self.data.iter().map(|(_, val)| from_wae_value(val.clone())).collect();

        let sql = format!("INSERT INTO {} ({}) VALUES ({})", E::table_name(), columns.join(", "), placeholders.join(", "));

        (sql, params)
    }

    #[cfg(feature = "mysql")]
    /// 构建 SQL 和参数 (内部使用 MySQL)
    pub(crate) fn build_mysql(&self) -> (String, Vec<MySqlValue>) {
        let columns: Vec<&str> = self.data.iter().map(|(col, _)| *col).collect();
        let placeholders: Vec<&str> = self.data.iter().map(|_| "?").collect();
        let params: Vec<MySqlValue> = self.data.iter().map(|(_, val)| from_wae_to_mysql(val.clone())).collect();

        let sql = format!("INSERT INTO {} ({}) VALUES ({})", E::table_name(), columns.join(", "), placeholders.join(", "));

        (sql, params)
    }
}

/// UPDATE 查询构建器
pub struct UpdateBuilder<E: Entity> {
    data: Vec<(&'static str, Value)>,
    conditions: Vec<Condition>,
    _marker: PhantomData<E>,
}

impl<E: Entity> UpdateBuilder<E> {
    pub(crate) fn new() -> Self {
        Self { data: Vec::new(), conditions: Vec::new(), _marker: PhantomData }
    }

    /// 设置列值
    pub fn set(mut self, column: &'static str, value: Value) -> Self {
        self.data.push((column, value));
        self
    }

    /// 批量设置列值
    pub fn set_all(mut self, data: Vec<(&'static str, Value)>) -> Self {
        self.data.extend(data);
        self
    }

    /// 从实体设置值 (排除主键)
    pub fn from_entity<T: ToRow + Entity>(entity: &T) -> Self {
        let id_col = T::id_column();
        let data: Vec<(&'static str, Value)> = entity.to_row().into_iter().filter(|(col, _)| *col != id_col).collect();
        Self { data, conditions: Vec::new(), _marker: PhantomData }
    }

    /// 添加 WHERE 条件
    pub fn where_(mut self, condition: Condition) -> Self {
        self.conditions.push(condition);
        self
    }

    /// 按主键更新
    pub fn where_id(mut self, id: E::Id) -> Self {
        self.conditions.push(Condition::eq(E::id_column(), id));
        self
    }

    #[cfg(feature = "limbo")]
    /// 构建 SQL 和参数 (内部使用)
    pub(crate) fn build_limbo(&self) -> (String, Vec<LimboValue>) {
        let set_parts: Vec<String> = self.data.iter().map(|(col, _)| format!("{} = ?", col)).collect();
        let mut params: Vec<LimboValue> = self.data.iter().map(|(_, val)| from_wae_value(val.clone())).collect();

        let mut sql = format!("UPDATE {} SET {}", E::table_name(), set_parts.join(", "));

        if !self.conditions.is_empty() {
            let where_conditions = self.conditions.to_vec();
            if where_conditions.len() == 1 {
                let (cond_sql, cond_params) = where_conditions[0].build_limbo();
                sql.push_str(&format!(" WHERE {}", cond_sql));
                params.extend(cond_params);
            }
            else {
                let (cond_sql, cond_params) = Condition::and(where_conditions).build_limbo();
                sql.push_str(&format!(" WHERE {}", cond_sql));
                params.extend(cond_params);
            }
        }

        (sql, params)
    }

    #[cfg(feature = "mysql")]
    /// 构建 SQL 和参数 (内部使用 MySQL)
    pub(crate) fn build_mysql(&self) -> (String, Vec<MySqlValue>) {
        let set_parts: Vec<String> = self.data.iter().map(|(col, _)| format!("{} = ?", col)).collect();
        let mut params: Vec<MySqlValue> = self.data.iter().map(|(_, val)| from_wae_to_mysql(val.clone())).collect();

        let mut sql = format!("UPDATE {} SET {}", E::table_name(), set_parts.join(", "));

        if !self.conditions.is_empty() {
            let where_conditions = self.conditions.to_vec();
            if where_conditions.len() == 1 {
                let (cond_sql, cond_params) = where_conditions[0].build_mysql();
                sql.push_str(&format!(" WHERE {}", cond_sql));
                params.extend(cond_params);
            }
            else {
                let (cond_sql, cond_params) = Condition::and(where_conditions).build_mysql();
                sql.push_str(&format!(" WHERE {}", cond_sql));
                params.extend(cond_params);
            }
        }

        (sql, params)
    }
}

/// DELETE 查询构建器
pub struct DeleteBuilder<E: Entity> {
    conditions: Vec<Condition>,
    _marker: PhantomData<E>,
}

impl<E: Entity> DeleteBuilder<E> {
    pub(crate) fn new() -> Self {
        Self { conditions: Vec::new(), _marker: PhantomData }
    }

    /// 添加 WHERE 条件
    pub fn where_(mut self, condition: Condition) -> Self {
        self.conditions.push(condition);
        self
    }

    /// 按主键删除
    pub fn where_id(mut self, id: E::Id) -> Self {
        self.conditions.push(Condition::eq(E::id_column(), id));
        self
    }

    #[cfg(feature = "limbo")]
    /// 构建 SQL 和参数 (内部使用)
    pub(crate) fn build_limbo(&self) -> (String, Vec<LimboValue>) {
        let mut sql = format!("DELETE FROM {}", E::table_name());
        let mut params = Vec::new();

        if !self.conditions.is_empty() {
            let where_conditions = self.conditions.to_vec();
            if where_conditions.len() == 1 {
                let (cond_sql, cond_params) = where_conditions[0].build_limbo();
                sql.push_str(&format!(" WHERE {}", cond_sql));
                params.extend(cond_params);
            }
            else {
                let (cond_sql, cond_params) = Condition::and(where_conditions).build_limbo();
                sql.push_str(&format!(" WHERE {}", cond_sql));
                params.extend(cond_params);
            }
        }

        (sql, params)
    }

    #[cfg(feature = "mysql")]
    /// 构建 SQL 和参数 (内部使用 MySQL)
    pub(crate) fn build_mysql(&self) -> (String, Vec<MySqlValue>) {
        let mut sql = format!("DELETE FROM {}", E::table_name());
        let mut params = Vec::new();

        if !self.conditions.is_empty() {
            let where_conditions = self.conditions.to_vec();
            if where_conditions.len() == 1 {
                let (cond_sql, cond_params) = where_conditions[0].build_mysql();
                sql.push_str(&format!(" WHERE {}", cond_sql));
                params.extend(cond_params);
            }
            else {
                let (cond_sql, cond_params) = Condition::and(where_conditions).build_mysql();
                sql.push_str(&format!(" WHERE {}", cond_sql));
                params.extend(cond_params);
            }
        }

        (sql, params)
    }
}

/// 查询构建器入口
pub struct QueryBuilder;

impl QueryBuilder {
    /// 创建 SELECT 构建器
    pub fn select<E: Entity>() -> SelectBuilder<E> {
        SelectBuilder::new()
    }

    /// 创建 INSERT 构建器
    pub fn insert<E: Entity>() -> InsertBuilder<E> {
        InsertBuilder::new()
    }

    /// 创建 UPDATE 构建器
    pub fn update<E: Entity>() -> UpdateBuilder<E> {
        UpdateBuilder::new()
    }

    /// 创建 DELETE 构建器
    pub fn delete<E: Entity>() -> DeleteBuilder<E> {
        DeleteBuilder::new()
    }
}
