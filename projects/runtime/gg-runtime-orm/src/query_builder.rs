//! 查询构建器实现
//!
//! 提供流式 API 构建 SQL 查询语句，
//! 支持条件过滤、JOIN、聚合、分组、HAVING、排序和分页。

use gg_runtime_database::DatabaseValue;

use crate::{AggregateFunction, FilterOp, JoinKind, Pagination};

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

/// JOIN 子句
///
/// 表示查询中的一个 JOIN 条件，
/// 由 JOIN 类型、目标表和连接条件组成。
#[derive(Debug, Clone)]
pub struct JoinClause {
    /// JOIN 类型
    pub kind: JoinKind,
    /// 目标表名
    pub table: String,
    /// 连接条件（ON 子句）
    pub on: String,
}

/// HAVING 条件
///
/// 表示查询中的一个 HAVING 条件，
/// 用于对聚合结果进行过滤。
#[derive(Debug, Clone)]
pub struct HavingCondition {
    /// 聚合函数
    pub function: AggregateFunction,
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
/// 提供流式 API 用于构建 SELECT、INSERT、UPDATE、DELETE 等 SQL 语句。
/// 支持条件过滤、JOIN、聚合、分组、HAVING、排序和分页等功能。
#[derive(Debug, Clone)]
pub struct QueryBuilder {
    /// 表名
    pub table_name: String,
    /// 过滤条件列表
    pub filters: Vec<FilterCondition>,
    /// JOIN 子句列表
    pub join_clauses: Vec<JoinClause>,
    /// GROUP BY 字段列表
    pub group_by_clauses: Vec<String>,
    /// HAVING 条件列表
    pub having_conditions: Vec<HavingCondition>,
    /// 聚合函数
    pub aggregate_function: Option<AggregateFunction>,
    /// 聚合字段
    pub aggregate_field: Option<String>,
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
            join_clauses: Vec::new(),
            group_by_clauses: Vec::new(),
            having_conditions: Vec::new(),
            aggregate_function: None,
            aggregate_field: None,
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

    /// 添加 INNER JOIN
    ///
    /// # 参数
    ///
    /// - `table`: 要连接的表名
    /// - `on`: 连接条件（ON 子句）
    pub fn join(self, table: &str, on: &str) -> Self {
        self.inner_join(table, on)
    }

    /// 添加 INNER JOIN
    ///
    /// # 参数
    ///
    /// - `table`: 要连接的表名
    /// - `on`: 连接条件（ON 子句）
    pub fn inner_join(mut self, table: &str, on: &str) -> Self {
        self.join_clauses.push(JoinClause { kind: JoinKind::Inner, table: table.to_string(), on: on.to_string() });
        self
    }

    /// 添加 LEFT JOIN
    ///
    /// # 参数
    ///
    /// - `table`: 要连接的表名
    /// - `on`: 连接条件（ON 子句）
    pub fn left_join(mut self, table: &str, on: &str) -> Self {
        self.join_clauses.push(JoinClause { kind: JoinKind::Left, table: table.to_string(), on: on.to_string() });
        self
    }

    /// 添加 RIGHT JOIN
    ///
    /// # 参数
    ///
    /// - `table`: 要连接的表名
    /// - `on`: 连接条件（ON 子句）
    pub fn right_join(mut self, table: &str, on: &str) -> Self {
        self.join_clauses.push(JoinClause { kind: JoinKind::Right, table: table.to_string(), on: on.to_string() });
        self
    }

    /// 添加 GROUP BY 字段
    ///
    /// # 参数
    ///
    /// - `field`: 分组字段名
    pub fn group_by(mut self, field: &str) -> Self {
        self.group_by_clauses.push(field.to_string());
        self
    }

    /// 添加 HAVING 条件
    ///
    /// 对聚合结果进行过滤。
    ///
    /// # 参数
    ///
    /// - `function`: 聚合函数类型
    /// - `field`: 聚合字段名
    /// - `op`: 比较操作符
    /// - `value`: 比较值
    pub fn having(mut self, function: AggregateFunction, field: &str, op: FilterOp, value: DatabaseValue) -> Self {
        self.having_conditions.push(HavingCondition {
            function,
            field: field.to_string(),
            op,
            value,
        });
        self
    }

    /// 设置 COUNT 聚合
    ///
    /// 生成 COUNT 聚合查询。
    ///
    /// # 参数
    ///
    /// - `field`: 计数字段名，使用 "*" 表示所有行
    pub fn count(mut self, field: &str) -> Self {
        self.aggregate_function = Some(AggregateFunction::Count);
        self.aggregate_field = Some(field.to_string());
        self
    }

    /// 设置 SUM 聚合
    ///
    /// # 参数
    ///
    /// - `field`: 求和字段名
    pub fn sum(mut self, field: &str) -> Self {
        self.aggregate_function = Some(AggregateFunction::Sum);
        self.aggregate_field = Some(field.to_string());
        self
    }

    /// 设置 AVG 聚合
    ///
    /// # 参数
    ///
    /// - `field`: 平均值字段名
    pub fn avg(mut self, field: &str) -> Self {
        self.aggregate_function = Some(AggregateFunction::Avg);
        self.aggregate_field = Some(field.to_string());
        self
    }

    /// 设置 MIN 聚合
    ///
    /// # 参数
    ///
    /// - `field`: 最小值字段名
    pub fn min(mut self, field: &str) -> Self {
        self.aggregate_function = Some(AggregateFunction::Min);
        self.aggregate_field = Some(field.to_string());
        self
    }

    /// 设置 MAX 聚合
    ///
    /// # 参数
    ///
    /// - `field`: 最大值字段名
    pub fn max(mut self, field: &str) -> Self {
        self.aggregate_function = Some(AggregateFunction::Max);
        self.aggregate_field = Some(field.to_string());
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

    /// 设置分页参数
    ///
    /// 根据 Pagination 对象自动设置 LIMIT 和 OFFSET。
    ///
    /// # 参数
    ///
    /// - `pagination`: 分页参数
    pub fn paginate(mut self, pagination: &Pagination) -> Self {
        self.limit_value = Some(pagination.limit());
        self.offset_value = Some(pagination.offset());
        self
    }

    /// 生成 SELECT SQL 语句
    ///
    /// 根据构建器状态生成完整的 SELECT 查询语句，
    /// 包含 JOIN、WHERE、GROUP BY、HAVING、ORDER BY、LIMIT 和 OFFSET 子句。
    pub fn build_select_sql(&self) -> String {
        let select_part = match &self.aggregate_function {
            Some(func) => {
                let field = self.aggregate_field.as_deref().unwrap_or("*");
                format!("SELECT {}({}) FROM {}", Self::aggregate_to_sql(*func), field, self.table_name)
            }
            None => format!("SELECT * FROM {}", self.table_name),
        };
        let mut sql = select_part;
        self.append_joins(&mut sql);
        self.append_where(&mut sql);
        self.append_group_by(&mut sql);
        self.append_having(&mut sql);
        self.append_order_by(&mut sql);
        self.append_limit_offset(&mut sql);
        sql
    }

    /// 生成 COUNT SQL 语句
    ///
    /// 根据构建器状态生成 COUNT 查询语句，
    /// 包含 JOIN 和 WHERE 子句，忽略排序和分页。
    pub fn build_count_sql(&self) -> String {
        let mut sql = format!("SELECT COUNT(*) FROM {}", self.table_name);
        self.append_joins(&mut sql);
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

    /// 生成 INSERT SQL 语句
    ///
    /// 根据列名列表生成 INSERT 语句模板。
    ///
    /// # 参数
    ///
    /// - `columns`: 列名列表
    pub fn build_insert_sql(&self, columns: &[&str]) -> String {
        let placeholders: Vec<&str> = columns.iter().map(|_| "?").collect();
        format!(
            "INSERT INTO {} ({}) VALUES ({})",
            self.table_name,
            columns.join(", "),
            placeholders.join(", ")
        )
    }

    /// 生成 UPDATE SQL 语句
    ///
    /// 根据列名列表和键列生成 UPDATE 语句模板。
    ///
    /// # 参数
    ///
    /// - `columns`: 要更新的列名列表
    /// - `key_column`: 用于 WHERE 条件的键列名
    pub fn build_update_sql(&self, columns: &[&str], key_column: &str) -> String {
        let set_clauses: Vec<String> = columns.iter().map(|c| format!("{} = ?", c)).collect();
        format!(
            "UPDATE {} SET {} WHERE {} = ?",
            self.table_name,
            set_clauses.join(", "),
            key_column
        )
    }

    /// 提取所有过滤条件的参数值
    ///
    /// 按照 WHERE 子句中占位符的顺序返回所有参数值，
    /// 可直接传递给数据库驱动的执行方法。
    pub fn params(&self) -> Vec<DatabaseValue> {
        self.filters.iter().map(|f| f.value.clone()).collect()
    }

    /// 提取 HAVING 条件的参数值
    ///
    /// 按照 HAVING 子句中占位符的顺序返回所有参数值。
    pub fn having_params(&self) -> Vec<DatabaseValue> {
        self.having_conditions.iter().map(|h| h.value.clone()).collect()
    }

    /// 将聚合函数转换为 SQL 字符串
    fn aggregate_to_sql(func: AggregateFunction) -> &'static str {
        match func {
            AggregateFunction::Count => "COUNT",
            AggregateFunction::Sum => "SUM",
            AggregateFunction::Avg => "AVG",
            AggregateFunction::Min => "MIN",
            AggregateFunction::Max => "MAX",
        }
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

    /// 拼接 JOIN 子句
    fn append_joins(&self, sql: &mut String) {
        for join in &self.join_clauses {
            let join_type = match join.kind {
                JoinKind::Inner => "INNER JOIN",
                JoinKind::Left => "LEFT JOIN",
                JoinKind::Right => "RIGHT JOIN",
            };
            sql.push_str(&format!(" {} {} ON {}", join_type, join.table, join.on));
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

    /// 拼接 GROUP BY 子句
    fn append_group_by(&self, sql: &mut String) {
        if self.group_by_clauses.is_empty() {
            return;
        }
        sql.push_str(" GROUP BY ");
        sql.push_str(&self.group_by_clauses.join(", "));
    }

    /// 拼接 HAVING 子句
    fn append_having(&self, sql: &mut String) {
        if self.having_conditions.is_empty() {
            return;
        }
        sql.push_str(" HAVING ");
        let conditions: Vec<String> = self
            .having_conditions
            .iter()
            .map(|h| {
                let func = Self::aggregate_to_sql(h.function);
                format!("{}({}) {} ?", func, h.field, Self::op_to_sql(h.op))
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
