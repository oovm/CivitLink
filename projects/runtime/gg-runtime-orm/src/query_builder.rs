//! 查询构建器实现
//!
//! 提供流式 API 构建 SQL 查询语句，
//! 支持条件过滤、排序和分页。

use gg_runtime_database::DatabaseValue;

use crate::FilterOp;

/// 过滤条件
///
/// 表示查询中的一个 WHERE 条件，
/// 由字段名、比较操作符和比较值组成。
#[derive(Debug, Clone)]
pub struct FilterCondition {
    /// 字段名
    pub field: String,
    /// 比较操作符
    pub op: FilterOp,
    /// 比较值
    pub value: DatabaseValue,
}

/// 排序子句
///
/// 表示查询中的一个 ORDER BY 条件，
/// 由字段名和排序方向组成。
#[derive(Debug, Clone)]
pub struct OrderBy {
    /// 字段名
    pub field: String,
    /// 是否降序排列
    pub desc: bool,
}

/// 查询构建器
///
/// 提供流式 API 用于构建 SELECT、COUNT、DELETE 等 SQL 语句。
/// 支持条件过滤、排序、分页等功能。
///
/// # 示例
///
/// ```ignore
/// use gg_runtime_orm::{QueryBuilder, FilterOp};
/// use gg_runtime_database::DatabaseValue;
///
/// let sql = QueryBuilder::new("users")
///     .filter("age", FilterOp::Gt, DatabaseValue::Integer(18))
///     .order_by("name", false)
///     .limit(10)
///     .build_select_sql();
/// ```
#[derive(Debug, Clone)]
pub struct QueryBuilder {
    /// 表名
    pub table_name: String,
    /// 过滤条件列表
    pub filters: Vec<FilterCondition>,
    /// 排序子句列表
    pub order_by_clauses: Vec<OrderBy>,
    /// LIMIT 值
    pub limit_value: Option<usize>,
    /// OFFSET 值
    pub offset_value: Option<usize>,
}

impl QueryBuilder {
    /// 创建新的查询构建器
    ///
    /// # 参数
    ///
    /// - `table_name`: 要查询的表名
    pub fn new(table_name: &str) -> Self {
        Self {
            table_name: table_name.to_string(),
            filters: Vec::new(),
            order_by_clauses: Vec::new(),
            limit_value: None,
            offset_value: None,
        }
    }

    /// 添加过滤条件
    ///
    /// 向查询中添加一个 WHERE 条件，
    /// 多个条件之间以 AND 连接。
    ///
    /// # 参数
    ///
    /// - `field`: 字段名
    /// - `op`: 比较操作符
    /// - `value`: 比较值
    pub fn filter(mut self, field: &str, op: FilterOp, value: DatabaseValue) -> Self {
        self.filters.push(FilterCondition { field: field.to_string(), op, value });
        self
    }

    /// 添加排序子句
    ///
    /// 向查询中添加一个 ORDER BY 条件，
    /// 多个排序条件按添加顺序排列。
    ///
    /// # 参数
    ///
    /// - `field`: 字段名
    /// - `desc`: 是否降序排列，true 为降序，false 为升序
    pub fn order_by(mut self, field: &str, desc: bool) -> Self {
        self.order_by_clauses.push(OrderBy { field: field.to_string(), desc });
        self
    }

    /// 设置返回行数限制
    ///
    /// # 参数
    ///
    /// - `n`: 最大返回行数
    pub fn limit(mut self, n: usize) -> Self {
        self.limit_value = Some(n);
        self
    }

    /// 设置结果偏移量
    ///
    /// # 参数
    ///
    /// - `n`: 跳过的行数
    pub fn offset(mut self, n: usize) -> Self {
        self.offset_value = Some(n);
        self
    }

    /// 设置只返回第一条记录
    ///
    /// 等价于 `limit(1)`，用于查询单条记录的场景。
    pub fn first(self) -> Self {
        self.limit(1)
    }

    /// 返回所有匹配记录
    ///
    /// 不设置限制条件，返回所有匹配的记录。
    /// 此方法为链式终端标记，不修改查询状态。
    pub fn all(self) -> Self {
        self
    }

    /// 生成 SELECT SQL 语句
    ///
    /// 根据构建器状态生成完整的 SELECT 查询语句，
    /// 包含 WHERE、ORDER BY、LIMIT 和 OFFSET 子句。
    pub fn build_select_sql(&self) -> String {
        let mut sql = format!("SELECT * FROM {}", self.table_name);
        self.append_where(&mut sql);
        self.append_order_by(&mut sql);
        self.append_limit_offset(&mut sql);
        sql
    }

    /// 生成 COUNT SQL 语句
    ///
    /// 根据构建器状态生成 COUNT 查询语句，
    /// 仅包含 WHERE 子句，忽略排序和分页。
    pub fn build_count_sql(&self) -> String {
        let mut sql = format!("SELECT COUNT(*) FROM {}", self.table_name);
        self.append_where(&mut sql);
        sql
    }

    /// 生成 DELETE SQL 语句
    ///
    /// 根据构建器状态生成 DELETE 语句，
    /// 仅包含 WHERE 子句，忽略排序和分页。
    pub fn build_delete_sql(&self) -> String {
        let mut sql = format!("DELETE FROM {}", self.table_name);
        self.append_where(&mut sql);
        sql
    }

    /// 提取所有过滤条件的参数值
    ///
    /// 按照 WHERE 子句中占位符的顺序返回所有参数值，
    /// 可直接传递给数据库驱动的执行方法。
    pub fn params(&self) -> Vec<DatabaseValue> {
        self.filters.iter().map(|f| f.value.clone()).collect()
    }

    /// 将过滤操作符转换为 SQL 运算符字符串
    fn op_to_sql(op: FilterOp) -> &'static str {
        match op {
            FilterOp::Eq => "=",
            FilterOp::Ne => "<>",
            FilterOp::Lt => "<",
            FilterOp::Le => "<=",
            FilterOp::Gt => ">",
            FilterOp::Ge => ">=",
            FilterOp::Like => "LIKE",
            FilterOp::In => "IN",
        }
    }

    /// 拼接 WHERE 子句
    fn append_where(&self, sql: &mut String) {
        if self.filters.is_empty() {
            return;
        }
        sql.push_str(" WHERE ");
        let conditions: Vec<String> = self
            .filters
            .iter()
            .map(|f| {
                if f.op == FilterOp::In {
                    format!("{} IN (?)", f.field)
                }
                else {
                    format!("{} {} ?", f.field, Self::op_to_sql(f.op))
                }
            })
            .collect();
        sql.push_str(&conditions.join(" AND "));
    }

    /// 拼接 ORDER BY 子句
    fn append_order_by(&self, sql: &mut String) {
        if self.order_by_clauses.is_empty() {
            return;
        }
        sql.push_str(" ORDER BY ");
        let clauses: Vec<String> = self
            .order_by_clauses
            .iter()
            .map(|o| if o.desc { format!("{} DESC", o.field) } else { format!("{} ASC", o.field) })
            .collect();
        sql.push_str(&clauses.join(", "));
    }

    /// 拼接 LIMIT 和 OFFSET 子句
    fn append_limit_offset(&self, sql: &mut String) {
        if let Some(limit) = self.limit_value {
            sql.push_str(&format!(" LIMIT {}", limit));
        }
        if let Some(offset) = self.offset_value {
            sql.push_str(&format!(" OFFSET {}", offset));
        }
    }
}
