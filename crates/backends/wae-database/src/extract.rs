//! HTTP 数据库连接提取器
//!
//! 提供从 HTTP 请求扩展中提取数据库连接的提取器。

use crate::connection::{DatabaseBackend, DatabaseConnection};
use std::{fmt, sync::Arc};
use wae_types::WaeError;

/// 数据库连接提取错误
///
/// 当无法从请求中提取数据库连接时返回的错误类型。
#[derive(Debug, Clone)]
pub struct DatabaseRejection {
    inner: WaeError,
}

impl DatabaseRejection {
    fn new(error: WaeError) -> Self {
        Self { inner: error }
    }

    /// 获取内部错误
    pub fn into_inner(self) -> WaeError {
        self.inner
    }
}

impl fmt::Display for DatabaseRejection {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.inner.fmt(f)
    }
}

impl std::error::Error for DatabaseRejection {}

/// 数据库连接提取器
///
/// 用于从 HTTP 请求的扩展中提取数据库连接。
///
/// # 示例
///
/// ```rust,ignore
/// use wae_database::extract::DatabaseConnectionExtractor;
///
/// async fn handler(request: &http::Request<Body>) {
///     let db = DatabaseConnectionExtractor::from_request(request).unwrap();
///     let rows = db.query("SELECT * FROM users").await.unwrap();
/// }
/// ```
#[derive(Clone)]
pub struct DatabaseConnectionExtractor {
    connection: Arc<dyn DatabaseConnection + Send + Sync>,
}

impl fmt::Debug for DatabaseConnectionExtractor {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("DatabaseConnectionExtractor").field("backend", &self.connection.backend()).finish()
    }
}

impl DatabaseConnectionExtractor {
    /// 创建数据库连接提取器
    pub fn new(connection: Arc<dyn DatabaseConnection + Send + Sync>) -> Self {
        Self { connection }
    }

    /// 从请求的扩展中提取数据库连接
    pub fn from_request<B>(request: &http::Request<B>) -> Result<Self, DatabaseRejection> {
        request
            .extensions()
            .get::<Arc<dyn DatabaseConnection + Send + Sync>>()
            .cloned()
            .map(|connection| DatabaseConnectionExtractor { connection })
            .ok_or_else(|| DatabaseRejection::new(WaeError::internal("Database connection not found in request extensions")))
    }

    /// 获取内部数据库连接的引用
    pub fn inner(&self) -> &(dyn DatabaseConnection + Send + Sync) {
        &*self.connection
    }

    /// 获取数据库后端类型
    pub fn backend(&self) -> DatabaseBackend {
        self.connection.backend()
    }

    /// 执行查询，返回结果集
    pub async fn query(&self, sql: &str) -> crate::connection::DatabaseResult<crate::connection::DatabaseRows> {
        self.connection.query(sql).await
    }

    /// 执行带参数的查询
    pub async fn query_with(
        &self,
        sql: &str,
        params: Vec<wae_types::Value>,
    ) -> crate::connection::DatabaseResult<crate::connection::DatabaseRows> {
        self.connection.query_with(sql, params).await
    }

    /// 执行语句，返回影响的行数
    pub async fn execute(&self, sql: &str) -> crate::connection::DatabaseResult<u64> {
        self.connection.execute(sql).await
    }

    /// 执行带参数的语句
    pub async fn execute_with(&self, sql: &str, params: Vec<wae_types::Value>) -> crate::connection::DatabaseResult<u64> {
        self.connection.execute_with(sql, params).await
    }

    /// 准备语句
    pub async fn prepare(&self, sql: &str) -> crate::connection::DatabaseResult<crate::connection::DatabaseStatement> {
        self.connection.prepare(sql).await
    }

    /// 开始一个新事务
    pub async fn begin_transaction(&self) -> crate::connection::DatabaseResult<()> {
        self.connection.begin_transaction().await
    }

    /// 提交当前事务
    pub async fn commit(&self) -> crate::connection::DatabaseResult<()> {
        self.connection.commit().await
    }

    /// 回滚当前事务
    pub async fn rollback(&self) -> crate::connection::DatabaseResult<()> {
        self.connection.rollback().await
    }
}

#[cfg(feature = "limbo")]
/// Limbo 数据库连接提取器
///
/// 用于从 HTTP 请求的扩展中提取 Limbo 数据库连接。
#[derive(Clone)]
pub struct LimboConnectionExtractor {
    connection: Arc<crate::connection::LimboConnection>,
}

#[cfg(feature = "limbo")]
impl fmt::Debug for LimboConnectionExtractor {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("LimboConnectionExtractor").field("backend", &DatabaseBackend::Limbo).finish()
    }
}

#[cfg(feature = "limbo")]
impl LimboConnectionExtractor {
    /// 创建 Limbo 连接提取器
    pub fn new(connection: Arc<crate::connection::LimboConnection>) -> Self {
        Self { connection }
    }

    /// 从请求的扩展中提取 Limbo 连接
    pub fn from_request<B>(request: &http::Request<B>) -> Result<Self, DatabaseRejection> {
        request
            .extensions()
            .get::<Arc<crate::connection::LimboConnection>>()
            .cloned()
            .map(|connection| LimboConnectionExtractor { connection })
            .ok_or_else(|| DatabaseRejection::new(WaeError::internal("Limbo connection not found in request extensions")))
    }

    /// 获取内部 Limbo 连接的引用
    pub fn inner(&self) -> &crate::connection::LimboConnection {
        &*self.connection
    }

    /// 获取数据库后端类型
    pub fn backend(&self) -> DatabaseBackend {
        self.connection.backend()
    }

    /// 执行查询，返回结果集
    pub async fn query(&self, sql: &str) -> crate::connection::DatabaseResult<crate::connection::DatabaseRows> {
        self.connection.query(sql).await
    }

    /// 执行带参数的查询
    pub async fn query_with(
        &self,
        sql: &str,
        params: Vec<wae_types::Value>,
    ) -> crate::connection::DatabaseResult<crate::connection::DatabaseRows> {
        self.connection.query_with(sql, params).await
    }

    /// 执行语句，返回影响的行数
    pub async fn execute(&self, sql: &str) -> crate::connection::DatabaseResult<u64> {
        self.connection.execute(sql).await
    }

    /// 执行带参数的语句
    pub async fn execute_with(&self, sql: &str, params: Vec<wae_types::Value>) -> crate::connection::DatabaseResult<u64> {
        self.connection.execute_with(sql, params).await
    }

    /// 准备语句
    pub async fn prepare(&self, sql: &str) -> crate::connection::DatabaseResult<crate::connection::DatabaseStatement> {
        self.connection.prepare(sql).await
    }

    /// 开始一个新事务
    pub async fn begin_transaction(&self) -> crate::connection::DatabaseResult<()> {
        self.connection.begin_transaction().await
    }

    /// 提交当前事务
    pub async fn commit(&self) -> crate::connection::DatabaseResult<()> {
        self.connection.commit().await
    }

    /// 回滚当前事务
    pub async fn rollback(&self) -> crate::connection::DatabaseResult<()> {
        self.connection.rollback().await
    }

    #[cfg(feature = "limbo")]
    /// 执行带参数的查询 (内部使用 limbo::Value)
    pub async fn query_with_limbo(
        &self,
        sql: &str,
        params: Vec<limbo::Value>,
    ) -> crate::connection::DatabaseResult<crate::connection::DatabaseRows> {
        self.connection.query_with_limbo(sql, params).await
    }

    #[cfg(feature = "limbo")]
    /// 执行带参数的语句 (内部使用 limbo::Value)
    pub async fn execute_with_limbo(&self, sql: &str, params: Vec<limbo::Value>) -> crate::connection::DatabaseResult<u64> {
        self.connection.execute_with_limbo(sql, params).await
    }
}

#[cfg(feature = "postgres")]
/// PostgreSQL 数据库连接提取器
///
/// 用于从 HTTP 请求的扩展中提取 PostgreSQL 数据库连接。
#[derive(Clone)]
pub struct PostgresConnectionExtractor {
    connection: Arc<crate::connection::PostgresConnection>,
}

#[cfg(feature = "postgres")]
impl fmt::Debug for PostgresConnectionExtractor {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("PostgresConnectionExtractor").field("backend", &DatabaseBackend::Postgres).finish()
    }
}

#[cfg(feature = "postgres")]
impl PostgresConnectionExtractor {
    /// 创建 PostgreSQL 连接提取器
    pub fn new(connection: Arc<crate::connection::PostgresConnection>) -> Self {
        Self { connection }
    }

    /// 从请求的扩展中提取 PostgreSQL 连接
    pub fn from_request<B>(request: &http::Request<B>) -> Result<Self, DatabaseRejection> {
        request
            .extensions()
            .get::<Arc<crate::connection::PostgresConnection>>()
            .cloned()
            .map(|connection| PostgresConnectionExtractor { connection })
            .ok_or_else(|| DatabaseRejection::new(WaeError::internal("PostgreSQL connection not found in request extensions")))
    }

    /// 获取内部 PostgreSQL 连接的引用
    pub fn inner(&self) -> &crate::connection::PostgresConnection {
        &*self.connection
    }

    /// 获取数据库后端类型
    pub fn backend(&self) -> DatabaseBackend {
        self.connection.backend()
    }

    /// 执行查询，返回结果集
    pub async fn query(&self, sql: &str) -> crate::connection::DatabaseResult<crate::connection::DatabaseRows> {
        self.connection.query(sql).await
    }

    /// 执行带参数的查询
    pub async fn query_with(
        &self,
        sql: &str,
        params: Vec<wae_types::Value>,
    ) -> crate::connection::DatabaseResult<crate::connection::DatabaseRows> {
        self.connection.query_with(sql, params).await
    }

    /// 执行语句，返回影响的行数
    pub async fn execute(&self, sql: &str) -> crate::connection::DatabaseResult<u64> {
        self.connection.execute(sql).await
    }

    /// 执行带参数的语句
    pub async fn execute_with(&self, sql: &str, params: Vec<wae_types::Value>) -> crate::connection::DatabaseResult<u64> {
        self.connection.execute_with(sql, params).await
    }

    /// 准备语句
    pub async fn prepare(&self, sql: &str) -> crate::connection::DatabaseResult<crate::connection::DatabaseStatement> {
        self.connection.prepare(sql).await
    }

    /// 开始一个新事务
    pub async fn begin_transaction(&self) -> crate::connection::DatabaseResult<()> {
        self.connection.begin_transaction().await
    }

    /// 提交当前事务
    pub async fn commit(&self) -> crate::connection::DatabaseResult<()> {
        self.connection.commit().await
    }

    /// 回滚当前事务
    pub async fn rollback(&self) -> crate::connection::DatabaseResult<()> {
        self.connection.rollback().await
    }
}

#[cfg(feature = "mysql")]
/// MySQL 数据库连接提取器
///
/// 用于从 HTTP 请求的扩展中提取 MySQL 数据库连接。
#[derive(Clone)]
pub struct MySqlConnectionExtractor {
    connection: Arc<crate::connection::MySqlConnection>,
}

#[cfg(feature = "mysql")]
impl fmt::Debug for MySqlConnectionExtractor {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("MySqlConnectionExtractor").field("backend", &DatabaseBackend::MySql).finish()
    }
}

#[cfg(feature = "mysql")]
impl MySqlConnectionExtractor {
    /// 创建 MySQL 连接提取器
    pub fn new(connection: Arc<crate::connection::MySqlConnection>) -> Self {
        Self { connection }
    }

    /// 从请求的扩展中提取 MySQL 连接
    pub fn from_request<B>(request: &http::Request<B>) -> Result<Self, DatabaseRejection> {
        request
            .extensions()
            .get::<Arc<crate::connection::MySqlConnection>>()
            .cloned()
            .map(|connection| MySqlConnectionExtractor { connection })
            .ok_or_else(|| DatabaseRejection::new(WaeError::internal("MySQL connection not found in request extensions")))
    }

    /// 获取内部 MySQL 连接的引用
    pub fn inner(&self) -> &crate::connection::MySqlConnection {
        &*self.connection
    }

    /// 获取数据库后端类型
    pub fn backend(&self) -> DatabaseBackend {
        self.connection.backend()
    }

    /// 执行查询，返回结果集
    pub async fn query(&self, sql: &str) -> crate::connection::DatabaseResult<crate::connection::DatabaseRows> {
        self.connection.query(sql).await
    }

    /// 执行带参数的查询
    pub async fn query_with(
        &self,
        sql: &str,
        params: Vec<wae_types::Value>,
    ) -> crate::connection::DatabaseResult<crate::connection::DatabaseRows> {
        self.connection.query_with(sql, params).await
    }

    /// 执行语句，返回影响的行数
    pub async fn execute(&self, sql: &str) -> crate::connection::DatabaseResult<u64> {
        self.connection.execute(sql).await
    }

    /// 执行带参数的语句
    pub async fn execute_with(&self, sql: &str, params: Vec<wae_types::Value>) -> crate::connection::DatabaseResult<u64> {
        self.connection.execute_with(sql, params).await
    }

    /// 准备语句
    pub async fn prepare(&self, sql: &str) -> crate::connection::DatabaseResult<crate::connection::DatabaseStatement> {
        self.connection.prepare(sql).await
    }

    /// 开始一个新事务
    pub async fn begin_transaction(&self) -> crate::connection::DatabaseResult<()> {
        self.connection.begin_transaction().await
    }

    /// 提交当前事务
    pub async fn commit(&self) -> crate::connection::DatabaseResult<()> {
        self.connection.commit().await
    }

    /// 回滚当前事务
    pub async fn rollback(&self) -> crate::connection::DatabaseResult<()> {
        self.connection.rollback().await
    }
}
