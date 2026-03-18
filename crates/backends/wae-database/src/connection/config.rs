//! 数据库配置模块

use serde::{Deserialize, Serialize};
#[cfg(feature = "limbo")]
use std::path::PathBuf;
use std::time::Duration;
use wae_types::{WaeError, WaeResult};

/// 数据库操作结果类型
pub type DatabaseResult<T> = WaeResult<T>;

/// 数据库错误类型
pub type DatabaseError = WaeError;

/// 连接回收方法
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RecyclingMethod {
    /// 快速回收，不执行任何检查
    Fast,
    /// 验证连接，执行简单的查询检查
    Verified,
    /// 清理连接，执行重置操作
    Clean,
}

impl Default for RecyclingMethod {
    fn default() -> Self {
        Self::Verified
    }
}

/// 连接池配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PoolConfig {
    /// 最大连接数
    pub max_connections: usize,
    /// 最小空闲连接数
    pub min_idle: Option<usize>,
    /// 连接超时时间（毫秒）
    pub connect_timeout_ms: Option<u64>,
    /// 等待连接超时时间（毫秒）
    pub wait_timeout_ms: Option<u64>,
    /// 连接最大空闲时间（毫秒）
    pub idle_timeout_ms: Option<u64>,
    /// 连接最大生命周期（毫秒）
    pub max_lifetime_ms: Option<u64>,
    /// 连接回收方法
    pub recycling_method: RecyclingMethod,
    /// 是否启用健康检查
    pub health_check_enabled: bool,
    /// 健康检查间隔（毫秒）
    pub health_check_interval_ms: Option<u64>,
}

impl Default for PoolConfig {
    fn default() -> Self {
        Self {
            max_connections: 10,
            min_idle: None,
            connect_timeout_ms: Some(5000),
            wait_timeout_ms: Some(30000),
            idle_timeout_ms: Some(600000),
            max_lifetime_ms: Some(1800000),
            recycling_method: RecyclingMethod::default(),
            health_check_enabled: true,
            health_check_interval_ms: Some(30000),
        }
    }
}

impl PoolConfig {
    /// 创建新的连接池配置
    pub fn new(max_connections: usize) -> Self {
        Self { max_connections, ..Default::default() }
    }

    /// 设置最小空闲连接数
    pub fn with_min_idle(mut self, min_idle: usize) -> Self {
        self.min_idle = Some(min_idle);
        self
    }

    /// 设置连接超时时间
    pub fn with_connect_timeout(mut self, timeout: Duration) -> Self {
        self.connect_timeout_ms = Some(timeout.as_millis() as u64);
        self
    }

    /// 设置等待连接超时时间
    pub fn with_wait_timeout(mut self, timeout: Duration) -> Self {
        self.wait_timeout_ms = Some(timeout.as_millis() as u64);
        self
    }

    /// 设置连接最大空闲时间
    pub fn with_idle_timeout(mut self, timeout: Duration) -> Self {
        self.idle_timeout_ms = Some(timeout.as_millis() as u64);
        self
    }

    /// 设置连接最大生命周期
    pub fn with_max_lifetime(mut self, lifetime: Duration) -> Self {
        self.max_lifetime_ms = Some(lifetime.as_millis() as u64);
        self
    }

    /// 设置连接回收方法
    pub fn with_recycling_method(mut self, method: RecyclingMethod) -> Self {
        self.recycling_method = method;
        self
    }

    /// 设置是否启用健康检查
    pub fn with_health_check(mut self, enabled: bool) -> Self {
        self.health_check_enabled = enabled;
        self
    }

    /// 设置健康检查间隔
    pub fn with_health_check_interval(mut self, interval: Duration) -> Self {
        self.health_check_interval_ms = Some(interval.as_millis() as u64);
        self
    }

    /// 获取连接超时时间
    pub fn connect_timeout(&self) -> Option<Duration> {
        self.connect_timeout_ms.map(Duration::from_millis)
    }

    /// 获取等待连接超时时间
    pub fn wait_timeout(&self) -> Option<Duration> {
        self.wait_timeout_ms.map(Duration::from_millis)
    }

    /// 获取连接最大空闲时间
    pub fn idle_timeout(&self) -> Option<Duration> {
        self.idle_timeout_ms.map(Duration::from_millis)
    }

    /// 获取连接最大生命周期
    pub fn max_lifetime(&self) -> Option<Duration> {
        self.max_lifetime_ms.map(Duration::from_millis)
    }

    /// 获取健康检查间隔
    pub fn health_check_interval(&self) -> Option<Duration> {
        self.health_check_interval_ms.map(Duration::from_millis)
    }
}

/// 数据库配置枚举
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DatabaseConfig {
    /// Limbo 配置
    #[cfg(feature = "limbo")]
    Limbo {
        /// 数据库文件路径，为 None 时使用内存数据库
        path: Option<PathBuf>,
    },
    /// PostgreSQL 配置
    #[cfg(feature = "postgres")]
    Postgres {
        /// 连接字符串
        connection_string: String,
        /// 连接池配置
        pool_config: PoolConfig,
    },
    /// MySQL 配置
    #[cfg(feature = "mysql")]
    MySql {
        /// 连接字符串
        connection_string: String,
        /// 连接池配置
        pool_config: PoolConfig,
    },
}

impl DatabaseConfig {
    /// 创建 Limbo 内存数据库配置
    #[cfg(feature = "limbo")]
    pub fn limbo_in_memory() -> Self {
        Self::Limbo { path: None }
    }

    /// 创建 Limbo 文件数据库配置
    #[cfg(feature = "limbo")]
    pub fn limbo_file<P: Into<PathBuf>>(path: P) -> Self {
        Self::Limbo { path: Some(path.into()) }
    }

    /// 创建 PostgreSQL 配置
    #[cfg(feature = "postgres")]
    pub fn postgres<S: Into<String>>(connection_string: S) -> Self {
        Self::Postgres { connection_string: connection_string.into(), pool_config: PoolConfig::default() }
    }

    /// 创建 PostgreSQL 配置，带最大连接数
    #[cfg(feature = "postgres")]
    pub fn postgres_with_max_connections<S: Into<String>>(connection_string: S, max_connections: usize) -> Self {
        Self::Postgres { connection_string: connection_string.into(), pool_config: PoolConfig::new(max_connections) }
    }

    /// 创建 PostgreSQL 配置，带完整的连接池配置
    #[cfg(feature = "postgres")]
    pub fn postgres_with_pool_config<S: Into<String>>(connection_string: S, pool_config: PoolConfig) -> Self {
        Self::Postgres { connection_string: connection_string.into(), pool_config }
    }

    /// 创建 MySQL 配置
    #[cfg(feature = "mysql")]
    pub fn mysql<S: Into<String>>(connection_string: S) -> Self {
        Self::MySql { connection_string: connection_string.into(), pool_config: PoolConfig::default() }
    }

    /// 创建 MySQL 配置，带最大连接数
    #[cfg(feature = "mysql")]
    pub fn mysql_with_max_connections<S: Into<String>>(connection_string: S, max_connections: usize) -> Self {
        Self::MySql { connection_string: connection_string.into(), pool_config: PoolConfig::new(max_connections) }
    }

    /// 创建 MySQL 配置，带完整的连接池配置
    #[cfg(feature = "mysql")]
    pub fn mysql_with_pool_config<S: Into<String>>(connection_string: S, pool_config: PoolConfig) -> Self {
        Self::MySql { connection_string: connection_string.into(), pool_config }
    }
}
