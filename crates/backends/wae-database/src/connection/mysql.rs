//! MySQL 数据库实现

use crate::connection::{
    config::{DatabaseConfig, DatabaseResult, PoolConfig},
    row::DatabaseRows,
    statement::DatabaseStatement,
    trait_impl::{DatabaseBackend, DatabaseConnection},
};
use async_trait::async_trait;
use mysql_async::prelude::*;
use std::sync::{
    Arc,
    atomic::{AtomicU64, Ordering},
};
use tokio::sync::Mutex;
use wae_types::{WaeError, WaeErrorKind};

/// MySQL 连接池监控指标
#[derive(Debug, Clone)]
pub struct MySqlPoolMetrics {
    /// 成功获取连接次数
    pub get_success: u64,
    /// 获取连接超时次数
    pub get_timeout: u64,
    /// 获取连接错误次数
    pub get_error: u64,
    /// 最大连接数
    pub max_connections: usize,
}

/// 原子 MySQL 连接池监控指标
#[derive(Debug)]
struct AtomicMySqlPoolMetrics {
    get_success: AtomicU64,
    get_timeout: AtomicU64,
    get_error: AtomicU64,
}

impl AtomicMySqlPoolMetrics {
    fn new() -> Self {
        Self { get_success: AtomicU64::new(0), get_timeout: AtomicU64::new(0), get_error: AtomicU64::new(0) }
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

/// MySQL 连接包装
pub struct MySqlConnection {
    conn: Mutex<Option<mysql_async::Conn>>,
}

impl MySqlConnection {
    pub(crate) fn new(conn: mysql_async::Conn) -> Self {
        Self { conn: Mutex::new(Some(conn)) }
    }

    /// 检查连接是否健康
    pub async fn health_check(&self) -> DatabaseResult<()> {
        let mut guard = self.conn.lock().await;
        let conn = guard.as_mut().ok_or_else(|| {
            WaeError::database(WaeErrorKind::DatabaseConnectionFailed { reason: "Connection closed".to_string() })
        })?;
        conn.query_drop("SELECT 1").await.map_err(|e| {
            WaeError::database(WaeErrorKind::DatabaseConnectionFailed { reason: format!("Health check failed: {}", e) })
        })?;
        Ok(())
    }
}

#[async_trait]
impl DatabaseConnection for MySqlConnection {
    fn backend(&self) -> DatabaseBackend {
        DatabaseBackend::MySql
    }

    async fn query(&self, sql: &str) -> DatabaseResult<DatabaseRows> {
        let mut guard = self.conn.lock().await;
        let conn = guard.as_mut().ok_or_else(|| {
            WaeError::database(WaeErrorKind::DatabaseConnectionFailed { reason: "Connection closed".to_string() })
        })?;
        let result = conn.query_iter(sql).await.map_err(|e| {
            WaeError::database(WaeErrorKind::QueryFailed { query: Some(sql.to_string()), reason: e.to_string() })
        })?;
        let rows: Vec<mysql_async::Row> = result.collect_and_drop().await.map_err(|e| {
            WaeError::database(WaeErrorKind::QueryFailed {
                query: Some(sql.to_string()),
                reason: format!("Failed to collect rows: {}", e),
            })
        })?;
        Ok(DatabaseRows::new_mysql(rows))
    }

    async fn query_with(&self, sql: &str, params: Vec<wae_types::Value>) -> DatabaseResult<DatabaseRows> {
        let mut guard = self.conn.lock().await;
        let conn = guard.as_mut().ok_or_else(|| {
            WaeError::database(WaeErrorKind::DatabaseConnectionFailed { reason: "Connection closed".to_string() })
        })?;
        let mysql_params = crate::types::to_mysql_params(params);
        let result = conn.exec_iter(sql, mysql_params).await.map_err(|e| {
            WaeError::database(WaeErrorKind::QueryFailed { query: Some(sql.to_string()), reason: e.to_string() })
        })?;
        let rows: Vec<mysql_async::Row> = result.collect_and_drop().await.map_err(|e| {
            WaeError::database(WaeErrorKind::QueryFailed {
                query: Some(sql.to_string()),
                reason: format!("Failed to collect rows: {}", e),
            })
        })?;
        Ok(DatabaseRows::new_mysql(rows))
    }

    async fn execute(&self, sql: &str) -> DatabaseResult<u64> {
        let mut guard = self.conn.lock().await;
        let conn = guard.as_mut().ok_or_else(|| {
            WaeError::database(WaeErrorKind::DatabaseConnectionFailed { reason: "Connection closed".to_string() })
        })?;
        let result = conn.query_iter(sql).await.map_err(|e| {
            WaeError::database(WaeErrorKind::ExecuteFailed { statement: Some(sql.to_string()), reason: e.to_string() })
        })?;
        let affected = result.affected_rows();
        result.drop_result().await.map_err(|e| {
            WaeError::database(WaeErrorKind::ExecuteFailed {
                statement: Some(sql.to_string()),
                reason: format!("Failed to drop result: {}", e),
            })
        })?;
        Ok(affected)
    }

    async fn execute_with(&self, sql: &str, params: Vec<wae_types::Value>) -> DatabaseResult<u64> {
        let mut guard = self.conn.lock().await;
        let conn = guard.as_mut().ok_or_else(|| {
            WaeError::database(WaeErrorKind::DatabaseConnectionFailed { reason: "Connection closed".to_string() })
        })?;
        let mysql_params = crate::types::to_mysql_params(params);
        let result = conn.exec_iter(sql, mysql_params).await.map_err(|e| {
            WaeError::database(WaeErrorKind::ExecuteFailed { statement: Some(sql.to_string()), reason: e.to_string() })
        })?;
        let affected = result.affected_rows();
        result.drop_result().await.map_err(|e| {
            WaeError::database(WaeErrorKind::ExecuteFailed {
                statement: Some(sql.to_string()),
                reason: format!("Failed to drop result: {}", e),
            })
        })?;
        Ok(affected)
    }

    async fn prepare(&self, sql: &str) -> DatabaseResult<DatabaseStatement> {
        Ok(DatabaseStatement::new_mysql(sql.to_string()))
    }

    async fn begin_transaction(&self) -> DatabaseResult<()> {
        let mut guard = self.conn.lock().await;
        let conn = guard.as_mut().ok_or_else(|| {
            WaeError::database(WaeErrorKind::DatabaseConnectionFailed { reason: "Connection closed".to_string() })
        })?;
        conn.query_drop("START TRANSACTION").await.map_err(|e| {
            WaeError::database(WaeErrorKind::DatabaseConnectionFailed { reason: format!("Failed to begin transaction: {}", e) })
        })?;
        Ok(())
    }

    async fn commit(&self) -> DatabaseResult<()> {
        let mut guard = self.conn.lock().await;
        let conn = guard.as_mut().ok_or_else(|| {
            WaeError::database(WaeErrorKind::DatabaseConnectionFailed { reason: "Connection closed".to_string() })
        })?;
        conn.query_drop("COMMIT").await.map_err(|e| {
            WaeError::database(WaeErrorKind::DatabaseConnectionFailed {
                reason: format!("Failed to commit transaction: {}", e),
            })
        })?;
        Ok(())
    }

    async fn rollback(&self) -> DatabaseResult<()> {
        let mut guard = self.conn.lock().await;
        let conn = guard.as_mut().ok_or_else(|| {
            WaeError::database(WaeErrorKind::DatabaseConnectionFailed { reason: "Connection closed".to_string() })
        })?;
        conn.query_drop("ROLLBACK").await.map_err(|e| {
            WaeError::database(WaeErrorKind::DatabaseConnectionFailed {
                reason: format!("Failed to rollback transaction: {}", e),
            })
        })?;
        Ok(())
    }
}

/// MySQL 数据库服务
pub struct MySqlDatabaseService {
    pool: mysql_async::Pool,
    metrics: Arc<AtomicMySqlPoolMetrics>,
    pool_config: PoolConfig,
}

impl MySqlDatabaseService {
    /// 从配置创建数据库服务
    pub async fn new(config: &DatabaseConfig) -> DatabaseResult<Self> {
        match config {
            #[cfg(feature = "postgres")]
            DatabaseConfig::Postgres { .. } => Err(WaeError::database(WaeErrorKind::DatabaseConnectionFailed {
                reason: "Use PostgresDatabaseService for Postgres".to_string(),
            })),
            DatabaseConfig::MySql { connection_string, pool_config } => {
                let opts = mysql_async::Opts::from_url(connection_string).map_err(|e| {
                    WaeError::database(WaeErrorKind::DatabaseConnectionFailed {
                        reason: format!("Invalid connection string: {}", e),
                    })
                })?;

                let min_idle = pool_config.min_idle.unwrap_or(1);
                let constraints =
                    mysql_async::PoolConstraints::new(min_idle, pool_config.max_connections).ok_or_else(|| {
                        WaeError::database(WaeErrorKind::DatabaseConnectionFailed {
                            reason: "Invalid pool constraints: min must be <= max".to_string(),
                        })
                    })?;

                let pool_opts = mysql_async::PoolOpts::default().with_constraints(constraints);

                let opts = mysql_async::OptsBuilder::from_opts(opts).pool_opts(pool_opts);
                let pool = mysql_async::Pool::new(opts);
                let metrics = Arc::new(AtomicMySqlPoolMetrics::new());

                Ok(Self { pool, metrics, pool_config: pool_config.clone() })
            },
            DatabaseConfig::Limbo { .. } => Err(WaeError::database(WaeErrorKind::DatabaseConnectionFailed {
                reason: "Use LimboDatabaseService for Limbo".to_string(),
            })),
        }
    }

    /// 获取连接
    pub async fn connect(&self) -> DatabaseResult<MySqlConnection> {
        match self.pool.get_conn().await {
            Ok(conn) => {
                self.metrics.increment_get_success();
                let connection = MySqlConnection::new(conn);

                if self.pool_config.health_check_enabled {
                    if let Err(e) = connection.health_check().await {
                        self.metrics.increment_get_error();
                        return Err(e);
                    }
                }

                Ok(connection)
            }
            Err(e) => {
                self.metrics.increment_get_error();
                Err(WaeError::database(WaeErrorKind::DatabaseConnectionFailed {
                    reason: format!("Failed to get connection: {}", e),
                }))
            }
        }
    }

    /// 获取连接池监控指标
    pub fn metrics(&self) -> MySqlPoolMetrics {
        MySqlPoolMetrics {
            get_success: self.metrics.get_success.load(Ordering::Relaxed),
            get_timeout: self.metrics.get_timeout.load(Ordering::Relaxed),
            get_error: self.metrics.get_error.load(Ordering::Relaxed),
            max_connections: self.pool_config.max_connections,
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
