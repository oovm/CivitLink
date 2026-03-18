//! 数据库行和结果集模块

use crate::connection::{config::DatabaseResult, value_impl::FromDatabaseValue};
#[cfg(feature = "limbo")]
use limbo::Row;
#[cfg(feature = "mysql")]
use mysql_async::Row as MySqlRow;
#[cfg(feature = "postgres")]
use tokio_postgres::Row as PostgresRow;
use wae_types::{WaeError, WaeErrorKind};

/// 查询结果行
pub enum DatabaseRow {
    /// Limbo 数据库行
    #[cfg(feature = "limbo")]
    Limbo(Row),
    /// PostgreSQL 数据库行
    #[cfg(feature = "postgres")]
    Postgres(PostgresRow),
    /// MySQL 数据库行
    #[cfg(feature = "mysql")]
    MySql(MySqlRow),
}

impl DatabaseRow {
    #[cfg(feature = "limbo")]
    pub(crate) fn new_limbo(row: Row) -> Self {
        Self::Limbo(row)
    }

    #[cfg(feature = "postgres")]
    pub(crate) fn new_postgres(row: PostgresRow) -> Self {
        Self::Postgres(row)
    }

    #[cfg(feature = "mysql")]
    pub(crate) fn new_mysql(row: MySqlRow) -> Self {
        Self::MySql(row)
    }

    /// 获取列值
    pub fn get<T: FromDatabaseValue>(&self, index: usize) -> DatabaseResult<T> {
        match self {
            #[cfg(feature = "limbo")]
            Self::Limbo(row) => {
                let value = row.get_value(index).map_err(|e| {
                    WaeError::database(WaeErrorKind::QueryFailed {
                        query: None,
                        reason: format!("Failed to get column {}: {}", index, e),
                    })
                })?;
                T::from_limbo_value(value)
            }
            #[cfg(feature = "postgres")]
            Self::Postgres(row) => T::from_postgres_row(row, index),
            #[cfg(feature = "mysql")]
            Self::MySql(row) => T::from_mysql_row(row, index),
        }
    }

    /// 获取列数量
    pub fn column_count(&self) -> usize {
        match self {
            #[cfg(feature = "limbo")]
            Self::Limbo(row) => row.column_count(),
            #[cfg(feature = "postgres")]
            Self::Postgres(row) => row.len(),
            #[cfg(feature = "mysql")]
            Self::MySql(row) => row.len(),
        }
    }
}

/// 查询结果集迭代器
pub enum DatabaseRows {
    /// Limbo 数据库结果集
    #[cfg(feature = "limbo")]
    Limbo(limbo::Rows),
    /// PostgreSQL 数据库结果集
    #[cfg(feature = "postgres")]
    Postgres(Vec<PostgresRow>, usize),
    /// MySQL 数据库结果集
    #[cfg(feature = "mysql")]
    MySql(Vec<MySqlRow>, usize),
}

impl DatabaseRows {
    #[cfg(feature = "limbo")]
    pub(crate) fn new_limbo(rows: limbo::Rows) -> Self {
        Self::Limbo(rows)
    }

    #[cfg(feature = "postgres")]
    pub(crate) fn new_postgres(rows: Vec<PostgresRow>) -> Self {
        Self::Postgres(rows, 0)
    }

    #[cfg(feature = "mysql")]
    pub(crate) fn new_mysql(rows: Vec<MySqlRow>) -> Self {
        Self::MySql(rows, 0)
    }

    /// 获取下一行
    pub async fn next(&mut self) -> DatabaseResult<Option<DatabaseRow>> {
        match self {
            #[cfg(feature = "limbo")]
            Self::Limbo(rows) => rows
                .next()
                .await
                .map_err(|e| {
                    WaeError::database(WaeErrorKind::QueryFailed { query: None, reason: format!("Failed to fetch row: {}", e) })
                })
                .map(|opt| opt.map(DatabaseRow::new_limbo)),
            #[cfg(feature = "postgres")]
            Self::Postgres(rows, index) => {
                if *index < rows.len() {
                    let row = DatabaseRow::new_postgres(rows[*index].clone());
                    *index += 1;
                    Ok(Some(row))
                }
                else {
                    Ok(None)
                }
            }
            #[cfg(feature = "mysql")]
            Self::MySql(rows, index) => {
                if *index < rows.len() {
                    let row = DatabaseRow::new_mysql(rows[*index].clone());
                    *index += 1;
                    Ok(Some(row))
                }
                else {
                    Ok(None)
                }
            }
        }
    }

    /// 收集所有行
    pub async fn collect(mut self) -> DatabaseResult<Vec<DatabaseRow>> {
        let mut rows = Vec::new();
        while let Some(row) = self.next().await? {
            rows.push(row);
        }
        Ok(rows)
    }
}
