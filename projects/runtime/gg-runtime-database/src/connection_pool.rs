//! 数据库连接池实现
//!
//! 提供泛型数据库连接池，管理数据库连接的获取、归还和健康检查。

use gg_core::{GError, GErrorKind, GResult};
use std::time::{Duration, Instant};

use crate::{DatabaseConfig, DatabaseDriver};

/// 连接池配置
///
/// 包含连接池的所有可配置参数。
#[derive(Debug, Clone)]
pub struct PoolConfig {
    /// 最大连接数
    pub max_size: usize,
    /// 连接空闲超时时间
    ///
    /// 超过此时间的空闲连接将被自动回收。
    pub idle_timeout: Option<Duration>,
    /// 连接健康检查间隔
    ///
    /// 定期检查连接是否仍然有效。
    pub health_check_interval: Option<Duration>,
}

impl PoolConfig {
    /// 创建新的连接池配置
    ///
    /// # 参数
    ///
    /// - `max_size`: 最大连接数
    pub fn new(max_size: usize) -> Self {
        Self { max_size, idle_timeout: None, health_check_interval: None }
    }

    /// 设置连接空闲超时
    pub fn with_idle_timeout(mut self, timeout: Duration) -> Self {
        self.idle_timeout = Some(timeout);
        self
    }

    /// 设置健康检查间隔
    pub fn with_health_check_interval(mut self, interval: Duration) -> Self {
        self.health_check_interval = Some(interval);
        self
    }
}

/// 池化连接包装
///
/// 包装数据库驱动实例，记录归还时间用于空闲超时检测。
pub struct PooledConnection<D: DatabaseDriver> {
    /// 内部数据库驱动
    pub inner: D,
    /// 归还到连接池的时间
    pub returned_at: Instant,
}

impl<D: DatabaseDriver> PooledConnection<D> {
    /// 创建新的池化连接
    ///
    /// # 参数
    ///
    /// - `inner`: 数据库驱动实例
    pub fn new(inner: D) -> Self {
        Self { inner, returned_at: Instant::now() }
    }
}

/// 数据库连接池
///
/// 管理数据库连接的连接池，支持获取、归还、健康检查和空闲连接回收。
/// 通过泛型参数 `D` 支持不同的数据库驱动类型。
pub struct DatabaseConnectionPool<D: DatabaseDriver> {
    /// 可用连接列表
    available: Vec<PooledConnection<D>>,
    /// 数据库配置
    config: DatabaseConfig,
    /// 连接池配置
    pool_config: PoolConfig,
    /// 已创建的连接总数（含已借出的连接）
    total: usize,
    /// 当前活跃（已借出）的连接数
    active: usize,
}

impl<D: DatabaseDriver> DatabaseConnectionPool<D> {
    /// 创建新的连接池
    ///
    /// # 参数
    ///
    /// - `config`: 数据库配置，用于创建新连接
    /// - `max_size`: 最大连接数限制
    pub fn new(config: DatabaseConfig, max_size: usize) -> GResult<Self> {
        Ok(Self { available: Vec::new(), config, pool_config: PoolConfig::new(max_size), total: 0, active: 0 })
    }

    /// 使用连接池配置创建连接池
    ///
    /// # 参数
    ///
    /// - `config`: 数据库配置
    /// - `pool_config`: 连接池配置
    pub fn with_pool_config(config: DatabaseConfig, pool_config: PoolConfig) -> GResult<Self> {
        Ok(Self { available: Vec::new(), config, pool_config, total: 0, active: 0 })
    }

    /// 从连接池获取一个连接
    ///
    /// 优先返回池中已有的可用连接；
    /// 如果没有可用连接且未达到最大连接数限制，则创建新连接；
    /// 如果已达上限，返回运行时错误。
    pub fn get(&mut self) -> GResult<D> {
        self.evict_idle_connections();

        if let Some(pooled) = self.available.pop() {
            self.active += 1;
            return Ok(pooled.inner);
        }

        if self.total < self.pool_config.max_size {
            let conn = D::connect(&self.config)?;
            self.total += 1;
            self.active += 1;
            return Ok(conn);
        }

        Err(GError {
            kind: GErrorKind::Runtime,
            message: format!("连接池已满，无法创建新连接（最大连接数: {}）", self.pool_config.max_size),
        })
    }

    /// 将连接归还到连接池
    ///
    /// 如果池中可用连接数未达到最大限制，将连接放回池中复用；
    /// 否则连接将被丢弃。
    pub fn put(&mut self, conn: D) {
        if self.available.len() < self.pool_config.max_size {
            self.available.push(PooledConnection::new(conn));
        }
        if self.active > 0 {
            self.active -= 1;
        }
    }

    /// 获取可用连接数
    pub fn available(&self) -> usize {
        self.available.len()
    }

    /// 获取已创建的连接总数
    pub fn total_connections(&self) -> usize {
        self.total
    }

    /// 获取当前活跃连接数
    pub fn active_connections(&self) -> usize {
        self.active
    }

    /// 对指定连接执行健康检查
    ///
    /// 通过执行简单查询验证连接是否仍然有效。
    ///
    /// # 参数
    ///
    /// - `conn`: 要检查的数据库连接
    pub fn health_check(&self, conn: &mut D) -> GResult<bool> {
        match conn.query("SELECT 1", &[]) {
            Ok(_) => Ok(true),
            Err(_) => Ok(false),
        }
    }

    /// 验证连接并返回，无效连接将被丢弃
    ///
    /// # 参数
    ///
    /// - `conn`: 要验证的数据库连接
    pub fn validate_connection(&mut self, mut conn: D) -> GResult<Option<D>> {
        if self.health_check(&mut conn)? {
            Ok(Some(conn))
        }
        else {
            self.total = self.total.saturating_sub(1);
            if self.active > 0 {
                self.active -= 1;
            }
            Ok(None)
        }
    }

    /// 回收空闲超时的连接
    ///
    /// 遍历可用连接列表，移除超过空闲超时时间的连接。
    pub fn evict_idle_connections(&mut self) {
        if let Some(idle_timeout) = self.pool_config.idle_timeout {
            let now = Instant::now();
            self.available.retain(|pooled| {
                let elapsed = now.duration_since(pooled.returned_at);
                if elapsed > idle_timeout {
                    self.total = self.total.saturating_sub(1);
                    false
                }
                else {
                    true
                }
            });
        }
    }

    /// 清空连接池
    ///
    /// 移除所有可用连接并重置连接计数。
    pub fn clear(&mut self) {
        let removed = self.available.len();
        self.available.clear();
        self.total = self.total.saturating_sub(removed);
    }
}
