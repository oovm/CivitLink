//! PostgreSQL 数据库实现

use crate::connection::{
    config::{DatabaseConfig, DatabaseResult, PoolConfig, RecyclingMethod},
    row::DatabaseRows,
    statement::DatabaseStatement,
    trait_impl::{DatabaseBackend, DatabaseConnection},
};
use async_trait::async_trait;
use deadpool_postgres::{Manager, ManagerConfig, Pool, PoolError, RecyclingMethod as DeadpoolRecyclingMethod};
use std::sync::{
    Arc,
    atomic::{AtomicU64, Ordering},
};
use tokio_postgres::{
    Config, NoTls,
    types::{ToSql, private::BytesMut},
};
use wae_types::{WaeError, WaeErrorKind};

/// 连接池监控指标
#[derive(Debug, Clone)]
pub struct PoolMetrics {
    /// 总连接创建次数
    pub connections_created: u64,
    /// 总连接销毁次数
    pub connections_destroyed: u64,
    /// 成功获取连接次数
    pub get_success: u64,
    /// 获取连接超时次数
    pub get_timeout: u64,
    /// 获取连接错误次数
    pub get_error: u64,
    /// 当前空闲连接数
    pub idle_connections: usize,
    /// 当前使用中的连接数
    pub active_connections: usize,
    /// 最大连接数
    pub max_connections: usize,
}

/// 原子连接池监控指标
#[derive(Debug)]
struct AtomicPoolMetrics {
    connections_created: AtomicU64,
    connections_destroyed: AtomicU64,
    get_success: AtomicU64,
    get_timeout: AtomicU64,
    get_error: AtomicU64,
}

impl AtomicPoolMetrics {
    fn new() -> Self {
        Self {
            connections_created: AtomicU64::new(0),
            connections_destroyed: AtomicU64::new(0),
            get_success: AtomicU64::new(0),
            get_timeout: AtomicU64::new(0),
            get_error: AtomicU64::new(0),
        }
    }

    fn increment_connections_created(&self) {
        self.connections_created.fetch_add(1, Ordering::Relaxed);
    }

    fn increment_connections_destroyed(&self) {
        self.connections_destroyed.fetch_add(1, Ordering::Relaxed);
    }

    fn increment_get_success(&self) {
        self.get_success.fetch_add(1, Ordering::Relaxed);
    }

    fn increment_get_timeout(&self) {
        self.get_timeout.fetch_add(1, Ordering::Relaxed);
    }

    fn increment_get_error(&self) {
        self.get_error.fetch_add(1, Ordering::Relaxed);
    }
}

/// PostgreSQL 参数值
///
/// 使用枚举来避免 trait object 的 Send 问题
#[derive(Debug, Clone)]
pub(crate) enum PostgresParam {
    Null,
    Bool(bool),
    Int(i64),
    Float(f64),
    String(String),
    Bytes(Vec<u8>),
}

impl ToSql for PostgresParam {
    fn to_sql(
        &self,
        ty: &tokio_postgres::types::Type,
        out: &mut BytesMut,
    ) -> std::result::Result<tokio_postgres::types::IsNull, Box<dyn std::error::Error + Sync + Send>> {
        match self {
            PostgresParam::Null => Option::<i32>::None.to_sql(ty, out),
            PostgresParam::Bool(b) => b.to_sql(ty, out),
            PostgresParam::Int(i) => i.to_sql(ty, out),
            PostgresParam::Float(f) => f.to_sql(ty, out),
            PostgresParam::String(s) => s.to_sql(ty, out),
            PostgresParam::Bytes(b) => b.to_sql(ty, out),
        }
    }

    fn accepts(_ty: &tokio_postgres::types::Type) -> bool {
        true
    }

    tokio_postgres::types::to_sql_checked!();
}

pub(crate) fn value_to_postgres_param(value: wae_types::Value) -> PostgresParam {
    match value {
        wae_types::Value::Null => PostgresParam::Null,
        wae_types::Value::Bool(b) => PostgresParam::Bool(b),
        wae_types::Value::Integer(i) => PostgresParam::Int(i),
        wae_types::Value::Float(f) => PostgresParam::Float(f),
        wae_types::Value::String(s) => PostgresParam::String(s),
        wae_types::Value::Bytes(b) => PostgresParam::Bytes(b),
        wae_types::Value::Array(l) => PostgresParam::String(serde_json::to_string(&l).unwrap_or_default()),
        wae_types::Value::Object(m) => PostgresParam::String(serde_json::to_string(&m).unwrap_or_default()),
    }
}

/// PostgreSQL 连接包装
pub struct PostgresConnection {
    client: deadpool_postgres::Client,
}

impl PostgresConnection {
    pub(crate) fn new(client: deadpool_postgres::Client) -> Self {
        Self { client }
    }

    /// 检查连接是否健康
    pub async fn health_check(&self) -> DatabaseResult<()> {
        self.client.simple_query("SELECT 1").await.map_err(|e| {
            WaeError::database(WaeErrorKind::DatabaseConnectionFailed { reason: format!("Health check failed: {}", e) })
        })?;
        Ok(())
    }
}

#[async_trait]
impl DatabaseConnection for PostgresConnection {
    fn backend(&self) -> DatabaseBackend {
        DatabaseBackend::Postgres
    }

    async fn query(&self, sql: &str) -> DatabaseResult<DatabaseRows> {
        let rows = self.client.query(sql, &[]).await.map_err(|e| {
            WaeError::database(WaeErrorKind::QueryFailed { query: Some(sql.to_string()), reason: e.to_string() })
        })?;
        Ok(DatabaseRows::new_postgres(rows))
    }

    async fn query_with(&self, sql: &str, params: Vec<wae_types::Value>) -> DatabaseResult<DatabaseRows> {
        let postgres_params: Vec<PostgresParam> = params.into_iter().map(value_to_postgres_param).collect();
        let postgres_refs: Vec<&(dyn ToSql + Sync)> = postgres_params.iter().map(|p| p as &(dyn ToSql + Sync)).collect();
        let rows = self.client.query(sql, postgres_refs.as_slice()).await.map_err(|e| {
            WaeError::database(WaeErrorKind::QueryFailed { query: Some(sql.to_string()), reason: e.to_string() })
        })?;
        Ok(DatabaseRows::new_postgres(rows))
    }

    async fn execute(&self, sql: &str) -> DatabaseResult<u64> {
        let affected = self.client.execute(sql, &[]).await.map_err(|e| {
            WaeError::database(WaeErrorKind::ExecuteFailed { statement: Some(sql.to_string()), reason: e.to_string() })
        })?;
        Ok(affected as u64)
    }

    async fn execute_with(&self, sql: &str, params: Vec<wae_types::Value>) -> DatabaseResult<u64> {
        let postgres_params: Vec<PostgresParam> = params.into_iter().map(value_to_postgres_param).collect();
        let postgres_refs: Vec<&(dyn ToSql + Sync)> = postgres_params.iter().map(|p| p as &(dyn ToSql + Sync)).collect();
        let affected = self.client.execute(sql, postgres_refs.as_slice()).await.map_err(|e| {
            WaeError::database(WaeErrorKind::ExecuteFailed { statement: Some(sql.to_string()), reason: e.to_string() })
        })?;
        Ok(affected as u64)
    }

    async fn prepare(&self, sql: &str) -> DatabaseResult<DatabaseStatement> {
        Ok(DatabaseStatement::new_postgres(sql.to_string()))
    }

    async fn begin_transaction(&self) -> DatabaseResult<()> {
        self.client.execute("BEGIN TRANSACTION", &[]).await.map_err(|e| {
            WaeError::database(WaeErrorKind::TransactionFailed { reason: format!("Failed to begin transaction: {}", e) })
        })?;
        Ok(())
    }

    async fn commit(&self) -> DatabaseResult<()> {
        self.client.execute("COMMIT", &[]).await.map_err(|e| {
            WaeError::database(WaeErrorKind::TransactionFailed { reason: format!("Failed to commit transaction: {}", e) })
        })?;
        Ok(())
    }

    async fn rollback(&self) -> DatabaseResult<()> {
        self.client.execute("ROLLBACK", &[]).await.map_err(|e| {
            WaeError::database(WaeErrorKind::TransactionFailed { reason: format!("Failed to rollback transaction: {}", e) })
        })?;
        Ok(())
    }
}

/// PostgreSQL 数据库服务
pub struct PostgresDatabaseService {
    pool: Pool,
    metrics: Arc<AtomicPoolMetrics>,
    pool_config: PoolConfig,
}

impl PostgresDatabaseService {
    /// 创建 PostgreSQL 数据库服务实例
    pub async fn new(config: &DatabaseConfig) -> DatabaseResult<Self> {
        match config {
            DatabaseConfig::Postgres { connection_string, pool_config } => {
                let pg_config: Config = connection_string.parse().map_err(|e| {
                    WaeError::database(WaeErrorKind::DatabaseConnectionFailed {
                        reason: format!("Invalid connection string: {}", e),
                    })
                })?;

                let recycling_method = match pool_config.recycling_method {
                    RecyclingMethod::Fast => DeadpoolRecyclingMethod::Fast,
                    RecyclingMethod::Verified => DeadpoolRecyclingMethod::Verified,
                    RecyclingMethod::Clean => DeadpoolRecyclingMethod::Clean,
                };

                let manager_config = ManagerConfig { recycling_method };
                let manager = Manager::from_config(pg_config, NoTls, manager_config);

                let mut pool_builder = Pool::builder(manager).max_size(pool_config.max_connections);

                if let Some(wait_timeout) = pool_config.wait_timeout() {
                    pool_builder = pool_builder.wait_timeout(Some(wait_timeout));
                }

                let pool = pool_builder.build().map_err(|e| {
                    WaeError::database(WaeErrorKind::DatabaseConnectionFailed {
                        reason: format!("Failed to create pool: {}", e),
                    })
                })?;

                let metrics = Arc::new(AtomicPoolMetrics::new());

                Ok(Self { pool, metrics, pool_config: pool_config.clone() })
            }
            #[cfg(feature = "mysql")]
            DatabaseConfig::MySql { .. } => Err(WaeError::database(WaeErrorKind::DatabaseConnectionFailed {
                reason: "Use MySqlDatabaseService for MySQL".to_string(),
            })),
            DatabaseConfig::Limbo { .. } => Err(WaeError::database(WaeErrorKind::DatabaseConnectionFailed {
                reason: "Use LimboDatabaseService for Limbo".to_string(),
            })),
        }
    }

    /// 从连接池获取数据库连接
    pub async fn connect(&self) -> DatabaseResult<PostgresConnection> {
        match self.pool.get().await {
            Ok(client) => {
                self.metrics.increment_get_success();
                let conn = PostgresConnection::new(client);

                if self.pool_config.health_check_enabled {
                    if let Err(e) = conn.health_check().await {
                        self.metrics.increment_get_error();
                        return Err(e);
                    }
                }

                Ok(conn)
            }
            Err(e) => {
                if let PoolError::Timeout(_) = e {
                    self.metrics.increment_get_timeout();
                }
                else {
                    self.metrics.increment_get_error();
                }
                Err(WaeError::database(WaeErrorKind::DatabaseConnectionFailed {
                    reason: format!("Failed to get connection: {}", e),
                }))
            }
        }
    }

    /// 获取连接池监控指标
    pub fn metrics(&self) -> PoolMetrics {
        let status = self.pool.status();
        PoolMetrics {
            connections_created: self.metrics.connections_created.load(Ordering::Relaxed),
            connections_destroyed: self.metrics.connections_destroyed.load(Ordering::Relaxed),
            get_success: self.metrics.get_success.load(Ordering::Relaxed),
            get_timeout: self.metrics.get_timeout.load(Ordering::Relaxed),
            get_error: self.metrics.get_error.load(Ordering::Relaxed),
            idle_connections: status.available,
            active_connections: status.size - status.available,
            max_connections: status.max_size,
        }
    }

    /// 获取连接池配置
    pub fn pool_config(&self) -> &PoolConfig {
        &self.pool_config
    }

    /// 预热连接池，创建最小数量的连接
    pub async fn warmup(&self) -> DatabaseResult<()> {
        if let Some(min_idle) = self.pool_config.min_idle {
            let mut connections = Vec::with_capacity(min_idle);
            for _ in 0..min_idle {
                connections.push(self.connect().await?);
            }
        }
        Ok(())
    }
}
