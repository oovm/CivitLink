//! SQLite 数据库驱动实现
//!
//! 基于 rusqlite 提供的 SQLite 数据库驱动，实现 `DatabaseDriver` trait。

use std::collections::HashMap;

use gg_core::{GError, GErrorKind, GResult};
use rusqlite::types::Type;

use crate::{DatabaseConfig, DatabaseDriver, DatabaseValue, Row, Transaction};

/// SQLite 数据库驱动
///
/// 基于 rusqlite 实现的 SQLite 数据库驱动，
/// 支持基本的 SQL 执行、查询和事务操作。
pub struct SqliteDriver {
    /// 数据库连接
    conn: Option<rusqlite::Connection>,
    /// 数据库文件路径
    path: String,
}

impl DatabaseDriver for SqliteDriver {
    fn connect(config: &DatabaseConfig) -> GResult<Self> {
        let path = if config.path.is_empty() { ":memory:".to_string() } else { config.path.clone() };

        let conn = rusqlite::Connection::open(&path)
            .map_err(|e| GError {
                kind: GErrorKind::Io, message: format!("无法打开 SQLite 数据库 '{}': {}", path, e)
            })?;

        Ok(Self { conn: Some(conn), path })
    }

    fn execute(&mut self, query: &str, params: &[DatabaseValue]) -> GResult<u64> {
        let conn = self
            .conn
            .as_mut()
            .ok_or_else(|| GError { kind: GErrorKind::Runtime, message: "数据库连接已关闭".to_string() })?;

        let rusqlite_params = convert_params(params);
        let param_refs: Vec<&dyn rusqlite::types::ToSql> = rusqlite_params.iter().map(|p| p.as_ref()).collect();

        let affected = conn
            .execute(query, param_refs.as_slice())
            .map_err(|e| GError { kind: GErrorKind::Io, message: format!("执行 SQL 失败: {}", e) })?;

        Ok(affected as u64)
    }

    fn query(&mut self, query: &str, params: &[DatabaseValue]) -> GResult<Vec<Row>> {
        let conn = self
            .conn
            .as_mut()
            .ok_or_else(|| GError { kind: GErrorKind::Runtime, message: "数据库连接已关闭".to_string() })?;

        let rusqlite_params = convert_params(params);
        let param_refs: Vec<&dyn rusqlite::types::ToSql> = rusqlite_params.iter().map(|p| p.as_ref()).collect();

        let mut stmt = conn
            .prepare(query)
            .map_err(|e| GError { kind: GErrorKind::Io, message: format!("准备 SQL 语句失败: {}", e) })?;

        let column_count = stmt.column_count();
        let column_names: Vec<String> =
            (0..column_count).map(|i| stmt.column_name(i).unwrap_or("unknown").to_string()).collect();

        let mut rows = stmt
            .query(param_refs.as_slice())
            .map_err(|e| GError { kind: GErrorKind::Io, message: format!("执行查询失败: {}", e) })?;

        let mut result = Vec::new();

        while let Some(row) =
            rows.next().map_err(|e| GError { kind: GErrorKind::Io, message: format!("读取查询结果失败: {}", e) })?
        {
            let mut map = HashMap::new();
            for (i, col_name) in column_names.iter().enumerate() {
                let value = row_to_database_value(row, i)?;
                map.insert(col_name.clone(), value);
            }
            result.push(Row::from_map(map));
        }

        Ok(result)
    }

    fn begin_transaction(&mut self) -> GResult<Transaction> {
        let conn = self
            .conn
            .as_mut()
            .ok_or_else(|| GError { kind: GErrorKind::Runtime, message: "数据库连接已关闭".to_string() })?;

        conn.execute_batch("BEGIN")
            .map_err(|e| GError { kind: GErrorKind::Io, message: format!("开始事务失败: {}", e) })?;

        Ok(Transaction::new(self.path.clone()))
    }

    fn close(&mut self) -> GResult<()> {
        if self.conn.is_some() {
            self.conn = None;
        }
        Ok(())
    }
}

impl SqliteDriver {
    /// 提交当前事务
    ///
    /// 向数据库发送 COMMIT 指令，提交当前活跃的事务。
    pub fn commit_transaction(&mut self) -> GResult<()> {
        let conn = self
            .conn
            .as_mut()
            .ok_or_else(|| GError { kind: GErrorKind::Runtime, message: "数据库连接已关闭".to_string() })?;

        conn.execute_batch("COMMIT")
            .map_err(|e| GError { kind: GErrorKind::Io, message: format!("提交事务失败: {}", e) })?;

        Ok(())
    }

    /// 回滚当前事务
    ///
    /// 向数据库发送 ROLLBACK 指令，回滚当前活跃的事务。
    pub fn rollback_transaction(&mut self) -> GResult<()> {
        let conn = self
            .conn
            .as_mut()
            .ok_or_else(|| GError { kind: GErrorKind::Runtime, message: "数据库连接已关闭".to_string() })?;

        conn.execute_batch("ROLLBACK")
            .map_err(|e| GError { kind: GErrorKind::Io, message: format!("回滚事务失败: {}", e) })?;

        Ok(())
    }

    /// 获取数据库文件路径
    pub fn path(&self) -> &str {
        &self.path
    }

    /// 判断连接是否已打开
    pub fn is_connected(&self) -> bool {
        self.conn.is_some()
    }
}

/// 将 `DatabaseValue` 列表转换为 rusqlite 参数列表
fn convert_params(params: &[DatabaseValue]) -> Vec<Box<dyn rusqlite::types::ToSql>> {
    params
        .iter()
        .map(|v| match v {
            DatabaseValue::Null => Box::new(Option::<String>::None) as Box<dyn rusqlite::types::ToSql>,
            DatabaseValue::Integer(i) => Box::new(*i) as Box<dyn rusqlite::types::ToSql>,
            DatabaseValue::Real(f) => Box::new(*f) as Box<dyn rusqlite::types::ToSql>,
            DatabaseValue::Text(s) => Box::new(s.clone()) as Box<dyn rusqlite::types::ToSql>,
            DatabaseValue::Blob(b) => Box::new(b.clone()) as Box<dyn rusqlite::types::ToSql>,
        })
        .collect()
}

/// 将 rusqlite 行中的指定列转换为 `DatabaseValue`
fn row_to_database_value(row: &rusqlite::Row<'_>, index: usize) -> GResult<DatabaseValue> {
    let col_type =
        row.get_ref(index).map_err(|e| GError { kind: GErrorKind::Io, message: format!("读取列类型失败: {}", e) })?;

    match col_type.data_type() {
        Type::Null => Ok(DatabaseValue::Null),
        Type::Integer => row
            .get(index)
            .map(DatabaseValue::Integer)
            .map_err(|e| GError { kind: GErrorKind::Io, message: format!("读取整数值失败: {}", e) }),
        Type::Real => row
            .get(index)
            .map(DatabaseValue::Real)
            .map_err(|e| GError { kind: GErrorKind::Io, message: format!("读取浮点数值失败: {}", e) }),
        Type::Text => row
            .get::<_, String>(index)
            .map(DatabaseValue::Text)
            .map_err(|e| GError { kind: GErrorKind::Io, message: format!("读取文本值失败: {}", e) }),
        Type::Blob => row
            .get::<_, Vec<u8>>(index)
            .map(DatabaseValue::Blob)
            .map_err(|e| GError { kind: GErrorKind::Io, message: format!("读取二进制数据失败: {}", e) }),
    }
}
