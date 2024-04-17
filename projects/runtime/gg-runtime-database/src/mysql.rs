//! MySQL 数据库驱动实现
//!
//! 基于 mysql crate 提供的 MySQL 数据库驱动，实现 `DatabaseDriver` trait。

use std::collections::HashMap;

use gg_core::{GError, GErrorKind, GResult};

use crate::{DatabaseConfig, DatabaseDriver, DatabaseValue, Row, Transaction};

/// MySQL 数据库驱动
///
/// 基于 mysql crate 实现的 MySQL 数据库驱动，
/// 支持基本的 SQL 执行、查询、事务操作和自增主键获取。
pub struct MySqlDriver {
    /// 数据库连接
    conn: Option<mysql::Pool>,
    /// 连接字符串
    conn_str: String,
}

impl DatabaseDriver for MySqlDriver {
    fn connect(config: &DatabaseConfig) -> GResult<Self> {
        let conn_str = if config.password.is_empty() {
            format!("mysql://{}@{}:{}/{}", config.username, config.host, config.port, config.database)
        }
        else {
            format!(
                "mysql://{}:{}@{}:{}/{}",
                config.username, config.password, config.host, config.port, config.database
            )
        };

        let pool = mysql::Pool::new(&conn_str).map_err(|e| GError {
            kind: GErrorKind::Io,
            message: format!("无法连接 MySQL 数据库: {}", e),
        })?;

        Ok(Self { conn: Some(pool), conn_str })
    }

    fn execute(&mut self, query: &str, params: &[DatabaseValue]) -> GResult<u64> {
        let pool = self
            .conn
            .as_ref()
            .ok_or_else(|| GError { kind: GErrorKind::Runtime, message: "数据库连接已关闭".to_string() })?;

        let mut conn = pool.get_conn().map_err(|e| GError {
            kind: GErrorKind::Io,
            message: format!("获取 MySQL 连接失败: {}", e),
        })?;

        let mysql_params: Vec<mysql::Value> = convert_params(params);
        let stmt = conn.prep(query).map_err(|e| GError {
            kind: GErrorKind::Io,
            message: format!("准备 SQL 语句失败: {}", e),
        })?;

        let result = conn.exec_iter(stmt, mysql_params).map_err(|e| GError {
            kind: GErrorKind::Io,
            message: format!("执行 SQL 失败: {}", e),
        })?;

        Ok(result.affected_rows())
    }

    fn query(&mut self, query: &str, params: &[DatabaseValue]) -> GResult<Vec<Row>> {
        let pool = self
            .conn
            .as_ref()
            .ok_or_else(|| GError { kind: GErrorKind::Runtime, message: "数据库连接已关闭".to_string() })?;

        let mut conn = pool.get_conn().map_err(|e| GError {
            kind: GErrorKind::Io,
            message: format!("获取 MySQL 连接失败: {}", e),
        })?;

        let mysql_params: Vec<mysql::Value> = convert_params(params);
        let stmt = conn.prep(query).map_err(|e| GError {
            kind: GErrorKind::Io,
            message: format!("准备 SQL 语句失败: {}", e),
        })?;

        let result = conn.exec_iter(stmt, mysql_params).map_err(|e| GError {
            kind: GErrorKind::Io,
            message: format!("执行查询失败: {}", e),
        })?;

        let mut rows_result = Vec::new();
        let column_names: Vec<String> = result
            .columns()
            .iter()
            .map(|col| col.name_str().to_string())
            .collect();

        for row in result {
            let row = row.map_err(|e| GError {
                kind: GErrorKind::Io,
                message: format!("读取查询结果失败: {}", e),
            })?;
            let mut map = HashMap::new();
            for (i, col_name) in column_names.iter().enumerate() {
                let value = row.get::<mysql::Value, usize>(i).unwrap_or(mysql::Value::NULL);
                map.insert(col_name.clone(), mysql_value_to_database_value(value));
            }
            rows_result.push(Row::from_map(map));
        }

        Ok(rows_result)
    }

    fn begin_transaction(&mut self) -> GResult<Transaction> {
        let pool = self
            .conn
            .as_ref()
            .ok_or_else(|| GError { kind: GErrorKind::Runtime, message: "数据库连接已关闭".to_string() })?;

        let mut conn = pool.get_conn().map_err(|e| GError {
            kind: GErrorKind::Io,
            message: format!("获取 MySQL 连接失败: {}", e),
        })?;

        conn.query_drop("START TRANSACTION").map_err(|e| GError {
            kind: GErrorKind::Io,
            message: format!("开始事务失败: {}", e),
        })?;

        Ok(Transaction::new(self.conn_str.clone()))
    }

    fn close(&mut self) -> GResult<()> {
        if self.conn.is_some() {
            self.conn = None;
        }
        Ok(())
    }

    fn last_insert_id(&mut self) -> GResult<Option<i64>> {
        let rows = self.query("SELECT LAST_INSERT_ID() AS id", &[])?;
        if let Some(row) = rows.first() {
            if let Some(DatabaseValue::Integer(id)) = row.get("id") {
                return Ok(Some(*id));
            }
        }
        Ok(None)
    }
}

/// 将 `DatabaseValue` 列表转换为 MySQL 参数列表
fn convert_params(params: &[DatabaseValue]) -> Vec<mysql::Value> {
    params
        .iter()
        .map(|v| match v {
            DatabaseValue::Null => mysql::Value::NULL,
            DatabaseValue::Integer(i) => mysql::Value::Int(*i),
            DatabaseValue::Real(f) => mysql::Value::Float(*f as f32),
            DatabaseValue::Text(s) => mysql::Value::Bytes(s.as_bytes().to_vec()),
            DatabaseValue::Blob(b) => mysql::Value::Bytes(b.clone()),
        })
        .collect()
}

/// 将 MySQL 值转换为 `DatabaseValue`
fn mysql_value_to_database_value(value: mysql::Value) -> DatabaseValue {
    match value {
        mysql::Value::NULL => DatabaseValue::Null,
        mysql::Value::Int(i) => DatabaseValue::Integer(i),
        mysql::Value::UInt(u) => DatabaseValue::Integer(u as i64),
        mysql::Value::Float(f) => DatabaseValue::Real(f as f64),
        mysql::Value::Double(d) => DatabaseValue::Real(d),
        mysql::Value::Bytes(b) => {
            match String::from_utf8(b) {
                Ok(s) => DatabaseValue::Text(s),
                Err(e) => DatabaseValue::Blob(e.into_bytes()),
            }
        }
        _ => DatabaseValue::Null,
    }
}
