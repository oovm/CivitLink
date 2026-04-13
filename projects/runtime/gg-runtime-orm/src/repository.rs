//! 仓库模式实现
//!
//! 提供基于 Repository 模式的数据访问抽象，
//! 封装数据库驱动的 CRUD 操作，支持批量操作和分页查询。

use gg_core::{GError, GErrorKind, GResult};
use gg_runtime_database::{DatabaseDriver, DatabaseValue, Row};

use crate::{PaginatedResult, Pagination, QueryBuilder};

/// 实体映射器 trait
///
/// 定义领域实体与数据库行之间的转换接口。
/// 每种实体类型需要实现此 trait 以支持 Repository 的自动映射。
pub trait EntityMapper {
    /// 获取表名
    ///
    /// 返回实体对应的数据库表名。
    fn table_name(&self) -> &str;

    /// 获取实体类型名称
    ///
    /// 返回实体的类型标识名称，用于错误信息和日志。
    fn entity_type_name(&self) -> &str;

    /// 将实体转换为数据库行
    ///
    /// 将领域实体转换为可插入数据库的行表示。
    ///
    /// # 参数
    ///
    /// - `entity`: 要转换的实体值
    fn to_row(&self, entity: &DatabaseValue) -> GResult<Row>;

    /// 将数据库行转换为实体
    ///
    /// 将数据库查询结果行转换为领域实体。
    ///
    /// # 参数
    ///
    /// - `row`: 数据库查询结果行
    fn from_row(&self, row: &Row) -> GResult<DatabaseValue>;
}

/// 仓库
///
/// 基于 Repository 模式的数据访问对象，
/// 封装了针对单个数据库表的 CRUD 操作，
/// 支持批量操作和分页查询。
pub struct Repository {
    /// 数据库驱动
    driver: Box<dyn DatabaseDriver>,
    /// 实体映射器
    mapper: Box<dyn EntityMapper>,
}

impl Repository {
    /// 创建新的仓库
    ///
    /// # 参数
    ///
    /// - `driver`: 已连接的数据库驱动实例
    /// - `mapper`: 实体映射器实例
    pub fn new(driver: Box<dyn DatabaseDriver>, mapper: Box<dyn EntityMapper>) -> Self {
        Self { driver, mapper }
    }

    /// 根据主键查询
    ///
    /// 以 "id" 列作为主键，查询指定主键值的记录。
    ///
    /// # 参数
    ///
    /// - `id`: 主键值
    pub fn find_by_id(&mut self, id: DatabaseValue) -> GResult<Option<Row>> {
        let sql = format!("SELECT * FROM {} WHERE id = ? LIMIT 1", self.mapper.table_name());
        let rows = self.driver.query(&sql, &[id])?;
        Ok(rows.into_iter().next())
    }

    /// 创建查询构建器
    ///
    /// 返回一个针对当前表的查询构建器实例，
    /// 可用于构建复杂的查询条件。
    pub fn query(&self) -> QueryBuilder {
        QueryBuilder::new(self.mapper.table_name())
    }

    /// 插入记录
    ///
    /// 将给定的行插入到数据库表中。
    /// 自动根据行的列信息生成 INSERT 语句。
    ///
    /// # 参数
    ///
    /// - `row`: 要插入的数据行
    pub fn insert(&mut self, row: &Row) -> GResult<()> {
        let columns = row.columns();
        if columns.is_empty() {
            return Err(GError {
                kind: GErrorKind::Runtime, message: "无法插入空行：行中没有任何列数据".to_string()
            });
        }

        let placeholders: Vec<&str> = columns.iter().map(|_| "?").collect();
        let column_names: Vec<&str> = columns.iter().map(|s| s.as_str()).collect();

        let sql = format!(
            "INSERT INTO {} ({}) VALUES ({})",
            self.mapper.table_name(),
            column_names.join(", "),
            placeholders.join(", ")
        );

        let mut params = Vec::with_capacity(columns.len());
        for col in &columns {
            match row.get(col) {
                Some(value) => params.push(value.clone()),
                None => params.push(DatabaseValue::Null),
            }
        }

        self.driver.execute(&sql, &params)?;
        Ok(())
    }

    /// 批量插入记录
    ///
    /// 将多行数据插入到数据库表中，每行单独执行 INSERT 语句。
    ///
    /// # 参数
    ///
    /// - `rows`: 要插入的数据行列表
    pub fn batch_insert(&mut self, rows: &[Row]) -> GResult<()> {
        for row in rows {
            self.insert(row)?;
        }
        Ok(())
    }

    /// 更新记录
    ///
    /// 根据指定的键列更新数据库中的记录。
    /// 自动根据行的列信息生成 UPDATE 语句，
    /// 键列用于 WHERE 条件，其余列用于 SET 子句。
    ///
    /// # 参数
    ///
    /// - `row`: 包含更新数据的行
    /// - `key_column`: 用于定位记录的键列名
    ///
    /// # 返回
    ///
    /// 受影响的行数
    pub fn update(&mut self, row: &Row, key_column: &str) -> GResult<u64> {
        let columns = row.columns();
        if columns.is_empty() {
            return Err(GError {
                kind: GErrorKind::Runtime, message: "无法更新空行：行中没有任何列数据".to_string()
            });
        }

        let mut set_clauses = Vec::new();
        let mut params = Vec::new();
        let mut key_value = None;

        for col in &columns {
            let value = match row.get(col) {
                Some(v) => v.clone(),
                None => DatabaseValue::Null,
            };

            if col.as_str() == key_column {
                key_value = Some(value);
            }
            else {
                set_clauses.push(format!("{} = ?", col));
                params.push(value);
            }
        }

        if set_clauses.is_empty() {
            return Err(GError {
                kind: GErrorKind::Runtime,
                message: format!("更新操作缺少 SET 子句：除键列 '{}' 外没有其他列", key_column),
            });
        }

        let key_val = key_value
            .ok_or_else(|| GError {
                kind: GErrorKind::Runtime, message: format!("更新操作缺少键列 '{}' 的值", key_column)
            })?;

        params.push(key_val);

        let sql = format!("UPDATE {} SET {} WHERE {} = ?", self.mapper.table_name(), set_clauses.join(", "), key_column);

        self.driver.execute(&sql, &params)
    }

    /// 批量更新记录
    ///
    /// 根据指定的键列批量更新数据库中的记录。
    ///
    /// # 参数
    ///
    /// - `rows`: 包含更新数据的行列表
    /// - `key_column`: 用于定位记录的键列名
    ///
    /// # 返回
    ///
    /// 受影响的总行数
    pub fn batch_update(&mut self, rows: &[Row], key_column: &str) -> GResult<u64> {
        let mut total_affected = 0u64;
        for row in rows {
            total_affected += self.update(row, key_column)?;
        }
        Ok(total_affected)
    }

    /// 删除记录
    ///
    /// 根据指定的键列和键值删除数据库中的记录。
    ///
    /// # 参数
    ///
    /// - `key_column`: 用于定位记录的键列名
    /// - `key_value`: 键值
    ///
    /// # 返回
    ///
    /// 受影响的行数
    pub fn delete(&mut self, key_column: &str, key_value: &DatabaseValue) -> GResult<u64> {
        let sql = format!("DELETE FROM {} WHERE {} = ?", self.mapper.table_name(), key_column);
        self.driver.execute(&sql, &[key_value.clone()])
    }

    /// 查询所有记录
    ///
    /// 返回表中所有行的数据。
    pub fn find_all(&mut self) -> GResult<Vec<Row>> {
        let sql = format!("SELECT * FROM {}", self.mapper.table_name());
        self.driver.query(&sql, &[])
    }

    /// 使用查询构建器执行查询，返回所有匹配行
    ///
    /// 根据 QueryBuilder 的状态生成 SQL 并执行查询。
    ///
    /// # 参数
    ///
    /// - `builder`: 查询构建器实例
    pub fn query_all(&mut self, builder: &QueryBuilder) -> GResult<Vec<Row>> {
        let sql = builder.build_select_sql();
        let params = builder.params();
        self.driver.query(&sql, &params)
    }

    /// 使用查询构建器执行查询，返回第一条匹配行
    ///
    /// 根据 QueryBuilder 的状态生成 SQL 并执行查询，
    /// 仅返回第一条匹配的记录。
    ///
    /// # 参数
    ///
    /// - `builder`: 查询构建器实例
    pub fn query_first(&mut self, builder: &QueryBuilder) -> GResult<Option<Row>> {
        let limited = builder.clone().first();
        let sql = limited.build_select_sql();
        let params = limited.params();
        let rows = self.driver.query(&sql, &params)?;
        Ok(rows.into_iter().next())
    }

    /// 统计记录总数
    ///
    /// 返回表中所有记录的数量。
    pub fn count(&mut self) -> GResult<usize> {
        let sql = format!("SELECT COUNT(*) AS cnt FROM {}", self.mapper.table_name());
        let rows = self.driver.query(&sql, &[])?;
        if let Some(row) = rows.first() {
            if let Some(DatabaseValue::Integer(n)) = row.get("cnt") {
                return Ok(*n as usize);
            }
        }
        Ok(0)
    }

    /// 使用查询构建器统计记录数
    ///
    /// 根据 QueryBuilder 的 WHERE 条件统计匹配记录的数量。
    ///
    /// # 参数
    ///
    /// - `builder`: 查询构建器实例
    pub fn count_with_query(&mut self, builder: &QueryBuilder) -> GResult<usize> {
        let sql = builder.build_count_sql();
        let params = builder.params();
        let rows = self.driver.query(&sql, &params)?;
        if let Some(row) = rows.first() {
            if let Some(DatabaseValue::Integer(n)) = row.get("COUNT(*)") {
                return Ok(*n as usize);
            }
        }
        Ok(0)
    }

    /// 检查记录是否存在
    ///
    /// 根据指定的键列和键值判断记录是否存在。
    ///
    /// # 参数
    ///
    /// - `key_column`: 键列名
    /// - `key_value`: 键值
    pub fn exists(&mut self, key_column: &str, key_value: &DatabaseValue) -> GResult<bool> {
        let sql = format!("SELECT 1 FROM {} WHERE {} = ? LIMIT 1", self.mapper.table_name(), key_column);
        let rows = self.driver.query(&sql, &[key_value.clone()])?;
        Ok(!rows.is_empty())
    }

    /// 分页查询
    ///
    /// 根据分页参数和查询构建器执行分页查询。
    ///
    /// # 参数
    ///
    /// - `builder`: 查询构建器实例
    /// - `pagination`: 分页参数
    pub fn query_paginated(
        &mut self,
        builder: &QueryBuilder,
        pagination: &Pagination,
    ) -> GResult<PaginatedResult<Row>> {
        let total = self.count_with_query(builder)?;

        let paginated_builder = builder.clone().paginate(pagination);
        let sql = paginated_builder.build_select_sql();
        let params = paginated_builder.params();
        let rows = self.driver.query(&sql, &params)?;

        Ok(PaginatedResult::new(rows, pagination.clone(), total))
    }
}
