//! TCP 网络驱动模块
//! 提供基于标准库同步 TCP 和 tokio 异步 TCP 的网络驱动和连接实现

use std::{
    future::Future,
    io::{Read, Write},
    net::{TcpListener, TcpStream},
    pin::Pin,
    time::Duration,
};

use gg_core::{GError, GErrorKind, GResult};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

use crate::{
    Connection, NetDriver,
    async_driver::{AsyncConnection, AsyncNetDriver},
};

/// TCP 监听器配置
///
/// 配置 TCP 监听器的超时和连接数限制等参数。
/// 所有字段均为 `Option` 类型，未设置时使用默认行为。
pub struct TcpListenerConfig {
    /// 连接空闲超时时间
    pub connection_timeout: Option<Duration>,
    /// 读取超时时间
    pub read_timeout: Option<Duration>,
    /// 写入超时时间
    pub write_timeout: Option<Duration>,
    /// 最大连接数限制
    pub max_connections: Option<usize>,
}

impl Default for TcpListenerConfig {
    fn default() -> Self {
        Self { connection_timeout: None, read_timeout: None, write_timeout: None, max_connections: None }
    }
}

/// TCP 网络驱动
///
/// 基于标准库 `std::net::TcpListener` 的同步 TCP 实现。
/// 适用于不需要异步 IO 的场景，例如游戏脚本 VM 的同步执行环境。
pub struct TcpDriver {
    /// TCP 监听器
    listener: Option<TcpListener>,
}

impl TcpDriver {
    /// 创建新的 TCP 驱动
    pub fn new() -> Self {
        Self { listener: None }
    }

    /// 同步连接到指定地址
    ///
    /// 使用标准库 `std::net::TcpStream::connect` 建立同步 TCP 客户端连接。
    ///
    /// # 参数
    /// - `addr`: 目标服务器地址，格式为 "host:port"
    pub fn connect(addr: &str) -> GResult<TcpConnection> {
        let stream = TcpStream::connect(addr)
            .map_err(|e| GError { kind: GErrorKind::Network, message: format!("TCP 连接失败: {}", e) })?;
        Ok(TcpConnection { stream: Some(stream) })
    }
}

impl NetDriver for TcpDriver {
    fn listen(&mut self, addr: &str) -> GResult<()> {
        let listener = TcpListener::bind(addr)
            .map_err(|e| GError { kind: GErrorKind::Io, message: format!("TCP 监听绑定失败: {}", e) })?;
        self.listener = Some(listener);
        Ok(())
    }

    fn accept(&mut self) -> GResult<Box<dyn Connection>> {
        let listener = self
            .listener
            .as_ref()
            .ok_or_else(|| GError { kind: GErrorKind::Runtime, message: "TCP 驱动尚未开始监听".to_string() })?;
        let (stream, _) = listener
            .accept()
            .map_err(|e| GError { kind: GErrorKind::Io, message: format!("TCP 接受连接失败: {}", e) })?;
        Ok(Box::new(TcpConnection { stream: Some(stream) }))
    }

    fn shutdown(&mut self) -> GResult<()> {
        self.listener = None;
        Ok(())
    }
}

/// TCP 连接
///
/// 基于标准库 `std::net::TcpStream` 的同步 TCP 连接实现。
pub struct TcpConnection {
    /// TCP 数据流
    stream: Option<TcpStream>,
}

impl TcpConnection {
    /// 从 TcpStream 创建 TCP 连接
    pub fn new(stream: TcpStream) -> Self {
        Self { stream: Some(stream) }
    }
}

impl Connection for TcpConnection {
    fn send(&mut self, data: &[u8]) -> GResult<()> {
        let stream = self
            .stream
            .as_mut()
            .ok_or_else(|| GError { kind: GErrorKind::Runtime, message: "TCP 连接已关闭".to_string() })?;
        stream
            .write_all(data)
            .map_err(|e| GError { kind: GErrorKind::Io, message: format!("TCP 发送数据失败: {}", e) })?;
        Ok(())
    }

    fn recv(&mut self) -> GResult<Vec<u8>> {
        let stream = self
            .stream
            .as_mut()
            .ok_or_else(|| GError { kind: GErrorKind::Runtime, message: "TCP 连接已关闭".to_string() })?;
        let mut buf = vec![0u8; 4096];
        let n = stream
            .read(&mut buf)
            .map_err(|e| GError { kind: GErrorKind::Io, message: format!("TCP 接收数据失败: {}", e) })?;
        buf.truncate(n);
        Ok(buf)
    }

    fn close(&mut self) -> GResult<()> {
        self.stream = None;
        Ok(())
    }

    fn peer_addr(&self) -> Option<String> {
        self.stream.as_ref().and_then(|s| s.peer_addr().ok()).map(|a| a.to_string())
    }
}

/// 异步 TCP 网络驱动
///
/// 基于 `tokio::net::TcpListener` 的异步 TCP 实现。
/// 支持连接超时、读写超时和最大连接数限制等配置。
pub struct AsyncTcpDriver {
    /// TCP 异步监听器
    listener: Option<tokio::net::TcpListener>,
    /// 监听器配置
    config: TcpListenerConfig,
    /// 当前连接数
    connection_count: usize,
}

impl AsyncTcpDriver {
    /// 创建新的异步 TCP 驱动
    pub fn new() -> Self {
        Self { listener: None, config: TcpListenerConfig::default(), connection_count: 0 }
    }

    /// 使用指定配置创建异步 TCP 驱动
    ///
    /// # 参数
    /// - `config`: TCP 监听器配置
    pub fn with_config(config: TcpListenerConfig) -> Self {
        Self { listener: None, config, connection_count: 0 }
    }
}

impl AsyncNetDriver for AsyncTcpDriver {
    fn listen<'a>(&'a mut self, addr: &'a str) -> Pin<Box<dyn Future<Output = GResult<()>> + Send + 'a>> {
        Box::pin(async move {
            let listener = tokio::net::TcpListener::bind(addr)
                .await
                .map_err(|e| GError { kind: GErrorKind::Network, message: format!("异步 TCP 监听绑定失败: {}", e) })?;
            self.listener = Some(listener);
            Ok(())
        })
    }

    fn accept(&mut self) -> Pin<Box<dyn Future<Output = GResult<Box<dyn AsyncConnection>>> + Send + '_>> {
        let max_connections = self.config.max_connections;
        let connection_count = self.connection_count;
        let read_timeout = self.config.read_timeout;
        let write_timeout = self.config.write_timeout;
        let connection_timeout = self.config.connection_timeout;
        Box::pin(async move {
            if let Some(max) = max_connections {
                if connection_count >= max {
                    return Err(GError {
                        kind: GErrorKind::Network, message: format!("已达到最大连接数限制: {}", max)
                    });
                }
            }
            let listener = self
                .listener
                .as_ref()
                .ok_or_else(|| GError {
                    kind: GErrorKind::Network, message: "异步 TCP 驱动尚未开始监听".to_string()
                })?;
            let accept_result = if let Some(timeout) = connection_timeout {
                tokio::time::timeout(timeout, listener.accept())
                    .await
                    .map_err(|_| GError { kind: GErrorKind::Network, message: "异步 TCP 接受连接超时".to_string() })?
            }
            else {
                listener.accept().await
            };
            let (stream, _) = accept_result
                .map_err(|e| GError { kind: GErrorKind::Network, message: format!("异步 TCP 接受连接失败: {}", e) })?;
            self.connection_count += 1;
            let conn = AsyncTcpConnection::new(stream, read_timeout, write_timeout);
            Ok(Box::new(conn) as Box<dyn AsyncConnection>)
        })
    }

    fn shutdown(&mut self) -> Pin<Box<dyn Future<Output = GResult<()>> + Send + '_>> {
        Box::pin(async move {
            self.listener = None;
            self.connection_count = 0;
            Ok(())
        })
    }
}

/// 异步 TCP 连接
///
/// 基于 `tokio::net::TcpStream` 的异步 TCP 连接实现。
/// 支持读写超时配置。
pub struct AsyncTcpConnection {
    /// TCP 异步数据流
    stream: Option<tokio::net::TcpStream>,
    /// 读取超时时间
    read_timeout: Option<Duration>,
    /// 写入超时时间
    write_timeout: Option<Duration>,
}

impl AsyncTcpConnection {
    /// 从 tokio TcpStream 创建异步 TCP 连接
    ///
    /// # 参数
    /// - `stream`: tokio 异步 TCP 数据流
    /// - `read_timeout`: 读取超时时间
    /// - `write_timeout`: 写入超时时间
    pub fn new(stream: tokio::net::TcpStream, read_timeout: Option<Duration>, write_timeout: Option<Duration>) -> Self {
        Self { stream: Some(stream), read_timeout, write_timeout }
    }
}

impl AsyncConnection for AsyncTcpConnection {
    fn send<'a>(&'a mut self, data: &'a [u8]) -> Pin<Box<dyn Future<Output = GResult<()>> + Send + 'a>> {
        Box::pin(async move {
            let stream = self
                .stream
                .as_mut()
                .ok_or_else(|| GError { kind: GErrorKind::Network, message: "异步 TCP 连接已关闭".to_string() })?;
            let write_result = if let Some(timeout) = self.write_timeout {
                tokio::time::timeout(timeout, stream.write_all(data))
                    .await
                    .map_err(|_| GError { kind: GErrorKind::Network, message: "异步 TCP 写入超时".to_string() })?
            }
            else {
                stream.write_all(data).await
            };
            write_result
                .map_err(|e| GError { kind: GErrorKind::Network, message: format!("异步 TCP 发送数据失败: {}", e) })?;
            Ok(())
        })
    }

    fn recv(&mut self) -> Pin<Box<dyn Future<Output = GResult<Vec<u8>>> + Send + '_>> {
        Box::pin(async move {
            let stream = self
                .stream
                .as_mut()
                .ok_or_else(|| GError { kind: GErrorKind::Network, message: "异步 TCP 连接已关闭".to_string() })?;
            let mut buf = vec![0u8; 4096];
            let read_result = if let Some(timeout) = self.read_timeout {
                tokio::time::timeout(timeout, stream.read(&mut buf))
                    .await
                    .map_err(|_| GError { kind: GErrorKind::Network, message: "异步 TCP 读取超时".to_string() })?
            }
            else {
                stream.read(&mut buf).await
            };
            let n = read_result
                .map_err(|e| GError { kind: GErrorKind::Network, message: format!("异步 TCP 接收数据失败: {}", e) })?;
            buf.truncate(n);
            Ok(buf)
        })
    }

    fn close(&mut self) -> Pin<Box<dyn Future<Output = GResult<()>> + Send + '_>> {
        Box::pin(async move {
            self.stream = None;
            Ok(())
        })
    }

    fn peer_addr(&self) -> Option<String> {
        self.stream.as_ref().and_then(|s| s.peer_addr().ok()).map(|a| a.to_string())
    }
}
