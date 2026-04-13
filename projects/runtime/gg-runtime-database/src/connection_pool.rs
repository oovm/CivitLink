//! 数据库连接池实现
//!
//! 提供泛型数据库连接池，管理数据库连接的获取和归还。

use gg_core::{GError, GErrorKind, GResult};

use crate::{DatabaseConfig, DatabaseDriver};

/// 数据库连接池
///
/// 管理数据库连接的连接池，支持获取和归还连接。
/// 通过泛型参数 `D` 支持不同的数据库驱动类型。
///
/// # 示例
///
/// ```ignore
/// use gg_database::{DatabaseConfig, DatabaseType, DatabaseConnectionPool, SqliteDriver};
///
/// let config = DatabaseConfig {
///     db_type: DatabaseType::Sqlite,
///     host: String::new(),
///     port: 0,
///     database: String::new(),
///     username: String::new(),
///     password: String::new(),
///     path: ":memory:".to_string(),
/// };
///
/// let mut pool = DatabaseConnectionPool::<SqliteDriver>::new(config, 10)?;
/// let conn = pool.get()?;
/// // 使用连接...
/// pool.put(conn);
/// ```
pub struct DatabaseConnectionPool<D: DatabaseDriver> {
    /// 可用连接列表
    available: Vec<D>,
    /// 数据库配置
    config: DatabaseConfig,
    /// 最大连接数
    max_size: usize,
    /// 已创建的连接总数（含已借出的连接）
    total: usize,
}

impl<D: DatabaseDriver> DatabaseConnectionPool<D> {
    /// 创建新的连接池
    ///
    /// # 参数
    ///
    /// - `config`: 数据库配置，用于创建新连接
    /// - `max_size`: 最大连接数限制
    pub fn new(config: DatabaseConfig, max_size: usize) -> GResult<Self> {
        Ok(Self { available: Vec::new(), config, max_size, total: 0 })
    }

    /// 从连接池获取一个连接
    ///
    /// 优先返回池中已有的可用连接；
    /// 如果没有可用连接且未达到最大连接数限制，则创建新连接；
    /// 如果已达上限，返回运行时错误。
    pub fn get(&mut self) -> GResult<D> {
        if let Some(conn) = self.available.pop() {
            return Ok(conn);
        }

        if self.total < self.max_size {
            let conn = D::connect(&self.config)?;
            self.total += 1;
            return Ok(conn);
        }

        Err(GError {
            kind: GErrorKind::Runtime,
            message: format!("连接池已满，无法创建新连接（最大连接数: {}）", self.max_size),
        })
    }

    /// 将连接归还到连接池
    ///
    /// 如果池中可用连接数未达到最大限制，将连接放回池中复用；
    /// 否则连接将被丢弃。
    pub fn put(&mut self, conn: D) {
        if self.available.len() < self.max_size {
            self.available.push(conn);
        }
    }

    /// 获取可用连接数
    pub fn available(&self) -> usize {
        self.available.len()
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
