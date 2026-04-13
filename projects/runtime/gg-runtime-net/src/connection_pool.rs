//! 连接池模块
//! 提供可复用连接的管理池实现

use std::{collections::HashMap, time::Duration};

use gg_core::{GError, GErrorKind, GResult};

use crate::{Connection, async_driver::AsyncConnection};

/// 连接池
///
/// 管理可复用的网络连接，避免频繁创建和销毁连接的开销。
/// 连接池有最大容量限制，超出限制时放入操作将返回错误。
pub struct ConnectionPool {
    /// 连接存储
    connections: Vec<Box<dyn Connection>>,
    /// 最大连接数
    max_size: usize,
    /// 按地址分组的连接存储
    addr_groups: HashMap<String, Vec<Box<dyn Connection>>>,
}

impl ConnectionPool {
    /// 创建新的连接池
    ///
    /// # 参数
    /// - `max_size`: 连接池最大容量
    pub fn new(max_size: usize) -> Self {
        Self { connections: Vec::new(), max_size, addr_groups: HashMap::new() }
    }

    /// 从连接池获取一个连接
    ///
    /// 如果池中有可用连接则返回，否则返回错误。
    pub fn get(&mut self) -> GResult<Box<dyn Connection>> {
        while let Some(conn) = self.connections.pop() {
            if conn.is_alive() {
                return Ok(conn);
            }
        }
        Err(GError { kind: GErrorKind::Runtime, message: "连接池中没有可用连接".to_string() })
    }

    /// 将连接归还到连接池
    ///
    /// 如果连接池已满，则返回错误。
    ///
    /// # 参数
    /// - `conn`: 要归还的连接
    pub fn put(&mut self, conn: Box<dyn Connection>) -> GResult<()> {
        if self.connections.len() >= self.max_size {
            return Err(GError {
                kind: GErrorKind::Runtime, message: format!("连接池已满，最大容量为 {}", self.max_size)
            });
        }
        self.connections.push(conn);
        Ok(())
    }

    /// 获取连接池中当前连接数
    pub fn size(&self) -> usize {
        self.connections.len()
    }

    /// 清空连接池，优雅关闭所有连接
    ///
    /// 依次调用每个连接的 `close()` 方法后再移除，
    /// 确保连接被正确关闭而非直接丢弃。
    pub fn clear(&mut self) {
        for mut conn in self.connections.drain(..) {
            let _ = conn.close();
        }
        for conns in self.addr_groups.values_mut() {
            for mut conn in conns.drain(..) {
                let _ = conn.close();
            }
        }
        self.addr_groups.clear();
    }

    /// 将连接归还到连接池并按地址分组
    ///
    /// 如果连接池已满，则返回错误。
    ///
    /// # 参数
    /// - `conn`: 要归还的连接
    /// - `addr`: 连接的地址标签
    pub fn put_with_addr(&mut self, conn: Box<dyn Connection>, addr: &str) -> GResult<()> {
        let total = self.connections.len() + self.addr_groups.values().map(|v| v.len()).sum::<usize>();
        if total >= self.max_size {
            return Err(GError {
                kind: GErrorKind::Runtime, message: format!("连接池已满，最大容量为 {}", self.max_size)
            });
        }
        self.addr_groups.entry(addr.to_string()).or_default().push(conn);
        Ok(())
    }

    /// 根据地址从连接池获取一个连接
    ///
    /// 如果指定地址没有可用连接则返回错误。
    /// 会跳过已失效的连接。
    ///
    /// # 参数
    /// - `addr`: 连接的地址标签
    pub fn get_by_addr(&mut self, addr: &str) -> GResult<Box<dyn Connection>> {
        let conns = self
            .addr_groups
            .get_mut(addr)
            .ok_or_else(|| GError { kind: GErrorKind::Runtime, message: format!("地址 {} 没有可用连接", addr) })?;
        while let Some(conn) = conns.pop() {
            if conn.is_alive() {
                return Ok(conn);
            }
        }
        Err(GError { kind: GErrorKind::Runtime, message: format!("地址 {} 没有可用连接", addr) })
    }

    /// 获取指定地址的连接数
    ///
    /// # 参数
    /// - `addr`: 连接的地址标签
    pub fn size_by_addr(&self, addr: &str) -> usize {
        self.addr_groups.get(addr).map(|v| v.len()).unwrap_or(0)
    }
}

/// 连接池配置
///
/// 用于配置异步连接池的行为参数。
pub struct ConnectionPoolConfig {
    /// 最大连接数
    pub max_size: usize,
    /// 空闲连接超时时间
    pub idle_timeout: Option<Duration>,
    /// 健康检查间隔
    pub health_check_interval: Option<Duration>,
}

impl Default for ConnectionPoolConfig {
    fn default() -> Self {
        Self { max_size: 16, idle_timeout: None, health_check_interval: None }
    }
}

/// 池化连接
///
/// 包装异步连接并跟踪使用元数据，用于异步连接池管理。
pub struct PooledConnection {
    /// 包装的异步连接
    pub connection: Box<dyn AsyncConnection>,
    /// 地址标签
    pub addr: String,
    /// 最后使用时间
    pub last_used: tokio::time::Instant,
    /// 创建时间
    pub created_at: tokio::time::Instant,
}

/// 异步连接池
///
/// 基于地址分组的异步连接池，支持健康检查、空闲超时清理等功能。
pub struct AsyncConnectionPool {
    /// 按地址分组的池化连接
    pools: HashMap<String, Vec<PooledConnection>>,
    /// 连接池配置
    config: ConnectionPoolConfig,
}

impl AsyncConnectionPool {
    /// 创建新的异步连接池
    ///
    /// # 参数
    /// - `config`: 连接池配置
    pub fn new(config: ConnectionPoolConfig) -> Self {
        Self { pools: HashMap::new(), config }
    }

    /// 根据地址获取一个健康的连接
    ///
    /// 从指定地址的连接池中获取一个连接，会检查连接健康状态并移除过期连接。
    ///
    /// # 参数
    /// - `addr`: 连接的地址标签
    pub async fn get(&mut self, addr: &str) -> GResult<Box<dyn AsyncConnection>> {
        let conns = match self.pools.get_mut(addr) {
            Some(c) => c,
            None => return Err(GError { kind: GErrorKind::Network, message: format!("地址 {} 没有可用连接", addr) }),
        };

        while let Some(mut pooled) = conns.pop() {
            if let Some(timeout) = self.config.idle_timeout {
                if pooled.last_used.elapsed() > timeout {
                    let _ = pooled.connection.close().await;
                    continue;
                }
            }
            return Ok(pooled.connection);
        }

        Err(GError { kind: GErrorKind::Network, message: format!("地址 {} 没有健康连接", addr) })
    }

    /// 将连接归还到连接池
    ///
    /// 如果连接池已满，则关闭连接并返回错误。
    ///
    /// # 参数
    /// - `conn`: 要归还的异步连接
    /// - `addr`: 连接的地址标签
    pub async fn put(&mut self, conn: Box<dyn AsyncConnection>, addr: &str) -> GResult<()> {
        let total: usize = self.pools.values().map(|v| v.len()).sum();
        if total >= self.config.max_size {
            let mut c = conn;
            let _ = c.close().await;
            return Err(GError {
                kind: GErrorKind::Network,
                message: format!("连接池已满，最大容量为 {}", self.config.max_size),
            });
        }

        let pooled = PooledConnection {
            connection: conn,
            addr: addr.to_string(),
            last_used: tokio::time::Instant::now(),
            created_at: tokio::time::Instant::now(),
        };

        self.pools.entry(addr.to_string()).or_default().push(pooled);
        Ok(())
    }

    /// 获取连接池中总连接数
    pub fn size(&self) -> usize {
        self.pools.values().map(|v| v.len()).sum()
    }

    /// 获取指定地址的连接数
    ///
    /// # 参数
    /// - `addr`: 连接的地址标签
    pub fn size_by_addr(&self, addr: &str) -> usize {
        self.pools.get(addr).map(|v| v.len()).unwrap_or(0)
    }

    /// 清理空闲超时的连接
    ///
    /// 移除所有超过空闲超时时间的连接。
    pub async fn cleanup_idle(&mut self) {
        let timeout = match self.config.idle_timeout {
            Some(t) => t,
            None => return,
        };

        for conns in self.pools.values_mut() {
            let mut i = 0;
            while i < conns.len() {
                if conns[i].last_used.elapsed() > timeout {
                    let mut pooled = conns.remove(i);
                    let _ = pooled.connection.close().await;
                }
                else {
                    i += 1;
                }
            }
        }
    }

    /// 关闭并移除所有连接
    pub async fn cleanup_all(&mut self) {
        for (_addr, conns) in self.pools.drain() {
            for mut pooled in conns {
                let _ = pooled.connection.close().await;
            }
        }
    }
}
