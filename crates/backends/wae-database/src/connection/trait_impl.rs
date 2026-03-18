//! 数据库连接抽象 trait

use crate::connection::{config::DatabaseResult, row::DatabaseRows, statement::DatabaseStatement};
use async_trait::async_trait;

/// 数据库后端类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DatabaseBackend {
    /// Limbo (SQLite) backend
    Limbo,
    /// PostgreSQL backend
    Postgres,
    /// MySQL backend
    MySql,
}

/// 数据库连接抽象
#[async_trait]
pub trait DatabaseConnection: Send + Sync {
    /// 获取数据库后端类型
    fn backend(&self) -> DatabaseBackend;

    /// 执行查询，返回结果集
    async fn query(&self, sql: &str) -> DatabaseResult<DatabaseRows>;

    /// 执行带参数的查询 (使用 wae_types::Value)
    async fn query_with(&self, sql: &str, params: Vec<wae_types::Value>) -> DatabaseResult<DatabaseRows>;

    /// 执行语句，返回影响的行数
    async fn execute(&self, sql: &str) -> DatabaseResult<u64>;

    /// 执行带参数的语句 (使用 wae_types::Value)
    async fn execute_with(&self, sql: &str, params: Vec<wae_types::Value>) -> DatabaseResult<u64>;

    /// 准备语句
    async fn prepare(&self, sql: &str) -> DatabaseResult<DatabaseStatement>;

    /// 开始一个新事务
    async fn begin_transaction(&self) -> DatabaseResult<()>;

    /// 提交当前事务
    async fn commit(&self) -> DatabaseResult<()>;

    /// 回滚当前事务
    async fn rollback(&self) -> DatabaseResult<()>;

    #[cfg(feature = "limbo")]
    /// 执行带参数的查询 (内部使用 limbo::Value)
    async fn query_with_limbo(&self, _sql: &str, _params: Vec<limbo::Value>) -> DatabaseResult<DatabaseRows> {
        unimplemented!("query_with_limbo is only implemented for Limbo connections")
    }

    #[cfg(feature = "limbo")]
    /// 执行带参数的语句 (内部使用 limbo::Value)
    async fn execute_with_limbo(&self, _sql: &str, _params: Vec<limbo::Value>) -> DatabaseResult<u64> {
        unimplemented!("execute_with_limbo is only implemented for Limbo connections")
    }
}
