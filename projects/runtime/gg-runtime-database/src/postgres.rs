//! PostgreSQL 数据库驱动实现
//!
//! 基于 postgres crate 提供的 PostgreSQL 数据库驱动，实现 `DatabaseDriver` trait。

use std::collections::HashMap;

use gg_core::{GError, GErrorKind, GResult};

use crate::{DatabaseConfig, DatabaseDriver, DatabaseValue, Row, Transaction};

/// PostgreSQL 数据库驱动
///
/// 基于 postgres crate 实现的 PostgreSQL 数据库驱动，
/// 支持基本的 SQL 执行、查询、事务操作和自增主键获取。
pub struct PostgresDriver {
    /// 数据库连接
    client: Option<postgres::Client>,
    /// 连接字符串
    conn_str: String,
}

impl DatabaseDriver for PostgresDriver {
    fn connect(config: &DatabaseConfig) -> GResult<Self> {
        let conn_str = if config.password.is_empty() {
            format!(
                "postgresql://{}@{}:{}/{}",
                config.username, config.host, config.port, config.database
            )
        }
        else {
            format!(
                "postgresql://{}:{}@{}:{}/{}",
                config.username, config.password, config.host, config.port, config.database
            )
        };

        let client = postgres::Client::connect(&conn_str, postgres::NoTls).map_err(|e| GError {
            kind: GErrorKind::Io,
            message: format!("无法连接 PostgreSQL 数据库: {}", e),
        })?;

        Ok(Self { client: Some(client), conn_str })
    }

    fn execute(&mut self, query: &str, params: &[DatabaseValue]) -> GResult<u64> {
        let client = self
            .client
            .as_mut()
            .ok_or_else(|| GError { kind: GErrorKind::Runtime, message: "数据库连接已关闭".to_string() })?;

        let pg_params: Vec<Box<dyn postgres::types::ToSqlSync>> = convert_params(params);
        let param_refs: Vec<&dyn postgres::types::ToSql> = pg_params.iter().map(|p| p.as_ref()).collect();

        let result = client.execute(query, param_refs.as_slice()).map_err(|e| GError {
            kind: GErrorKind::Io,
            message: format!("执行 SQL 失败: {}", e),
        })?;

        Ok(result)
    }

    fn query(&mut self, query: &str, params: &[DatabaseValue]) -> GResult<Vec<Row>> {
        let client = self
            .client
            .as_mut()
            .ok_or_else(|| GError { kind: GErrorKind::Runtime, message: "数据库连接已关闭".to_string() })?;

        let pg_params: Vec<Box<dyn postgres::types::ToSqlSync>> = convert_params(params);
        let param_refs: Vec<&dyn postgres::types::ToSql> = pg_params.iter().map(|p| p.as_ref()).collect();

        let rows = client.query(query, param_refs.as_slice()).map_err(|e| GError {
            kind: GErrorKind::Io,
            message: format!("执行查询失败: {}", e),
        })?;

        let mut result = Vec::new();
        for row in &rows {
            let mut map = HashMap::new();
            for (i, column) in row.columns().iter().enumerate() {
                let value = pg_row_to_database_value(row, i)?;
                map.insert(column.name().to_string(), value);
            }
            result.push(Row::from_map(map));
        }

        Ok(result)
    }

    fn begin_transaction(&mut self) -> GResult<Transaction> {
        let client = self
            .client
            .as_mut()
            .ok_or_else(|| GError { kind: GErrorKind::Runtime, message: "数据库连接已关闭".to_string() })?;

        client.batch_execute("BEGIN").map_err(|e| GError {
            kind: GErrorKind::Io,
            message: format!("开始事务失败: {}", e),
        })?;

        Ok(Transaction::new(self.conn_str.clone()))
    }

    fn close(&mut self) -> GResult<()> {
        if self.client.is_some() {
            self.client = None;
        }
        Ok(())
    }

    fn last_insert_id(&mut self) -> GResult<Option<i64>> {
        let rows = self.query("SELECT lastval() AS id", &[])?;
        if let Some(row) = rows.first() {
            if let Some(DatabaseValue::Integer(id)) = row.get("id") {
                return Ok(Some(*id));
            }
        }
        Ok(None)
    }
}

/// 将 `DatabaseValue` 列表转换为 PostgreSQL 参数列表
fn convert_params(params: &[DatabaseValue]) -> Vec<Box<dyn postgres::types::ToSqlSync>> {
    params
        .iter()
        .map(|v| match v {
            DatabaseValue::Null => Box::new(Option::<String>::None) as Box<dyn postgres::types::ToSqlSync>,
            DatabaseValue::Integer(i) => Box::new(*i) as Box<dyn postgres::types::ToSqlSync>,
            DatabaseValue::Real(f) => Box::new(*f) as Box<dyn postgres::types::ToSqlSync>,
            DatabaseValue::Text(s) => Box::new(s.clone()) as Box<dyn postgres::types::ToSqlSync>,
            DatabaseValue::Blob(b) => Box::new(b.clone()) as Box<dyn postgres::types::ToSqlSync>,
        })
        .collect()
}

/// 将 PostgreSQL 行中的指定列转换为 `DatabaseValue`
fn pg_row_to_database_value(row: &postgres::Row, index: usize) -> GResult<DatabaseValue> {
    let col_type = row.columns()[index].type_();

    if col_type == &postgres::types::Type::INT2
        || col_type == &postgres::types::Type::INT4
        || col_type == &postgres::types::Type::INT8
    {
        row.try_get::<_, Option<i64>>(index)
            .map(|v| v.map(DatabaseValue::Integer).unwrap_or(DatabaseValue::Null))
            .map_err(|e| GError { kind: GErrorKind::Io, message: format!("读取整数值失败: {}", e) })
    }
    else if col_type == &postgres::types::Type::FLOAT4
        || col_type == &postgres::types::Type::FLOAT8
    {
        row.try_get::<_, Option<f64>>(index)
            .map(|v| v.map(DatabaseValue::Real).unwrap_or(DatabaseValue::Null))
            .map_err(|e| GError { kind: GErrorKind::Io, message: format!("读取浮点数值失败: {}", e) })
    }
    else if col_type == &postgres::types::Type::TEXT || col_type == &postgres::types::Type::VARCHAR {
        row.try_get::<_, Option<String>>(index)
            .map(|v| v.map(DatabaseValue::Text).unwrap_or(DatabaseValue::Null))
            .map_err(|e| GError { kind: GErrorKind::Io, message: format!("读取文本值失败: {}", e) })
    }
    else if col_type == &postgres::types::Type::BYTEA {
        row.try_get::<_, Option<Vec<u8>>>(index)
            .map(|v| v.map(DatabaseValue::Blob).unwrap_or(DatabaseValue::Null))
            .map_err(|e| GError { kind: GErrorKind::Io, message: format!("读取二进制数据失败: {}", e) })
    }
    else {
        row.try_get::<_, Option<String>>(index)
            .map(|v| v.map(DatabaseValue::Text).unwrap_or(DatabaseValue::Null))
            .unwrap_or(Ok(DatabaseValue::Null))
    }
}
