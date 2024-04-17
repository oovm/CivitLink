#![warn(missing_docs)]

#![doc = include_str!("readme.md")]

use gg_core::GResult;

/// 网络驱动 trait
///
/// 定义网络监听、接受连接和关闭的统一接口。
/// 不同的网络协议（TCP、WebSocket 等）实现此 trait 以提供协议特定的驱动。
pub trait NetDriver {
    /// 在指定地址上开始监听
    ///
    /// # 参数
    /// - `addr`: 监听地址，格式为 "host:port"
    fn listen(&mut self, addr: &str) -> GResult<()>;

    /// 接受一个新的连接
    ///
    /// 阻塞等待直到有新的客户端连接到达，返回封装后的连接对象。
    fn accept(&mut self) -> GResult<Box<dyn Connection>>;

    /// 关闭网络驱动，释放资源
    fn shutdown(&mut self) -> GResult<()>;
}

/// 网络连接 trait
///
/// 定义数据收发和连接管理的统一接口。
/// 所有协议的连接对象都需要实现此 trait。
pub trait Connection {
    /// 发送数据到对端
    ///
    /// # 参数
    /// - `data`: 要发送的字节数据
    fn send(&mut self, data: &[u8]) -> GResult<()>;

    /// 从对端接收数据
    ///
    /// 阻塞等待直到接收到数据，返回接收到的字节数据。
    fn recv(&mut self) -> GResult<Vec<u8>>;

    /// 关闭连接，释放资源
    fn close(&mut self) -> GResult<()>;

    /// 获取对端地址
    ///
    /// 返回对端的地址字符串，如果无法获取则返回 None。
    fn peer_addr(&self) -> Option<String>;

    /// 检查连接是否存活
    ///
    /// 默认实现返回 true，具体连接类型可覆盖此方法提供更精确的存活检测。
    fn is_alive(&self) -> bool {
        true
    }
}

/// TCP 网络驱动模块
pub mod tcp;

/// WebSocket 网络驱动模块
pub mod websocket;

/// HTTP 网络驱动模块
pub mod http;

/// 连接池模块
pub mod connection_pool;

/// 异步网络驱动模块
pub mod async_driver;

/// 异步 HTTP 服务器模块
pub mod async_http;

#[cfg(feature = "tls")]
/// TLS/SSL 安全连接模块
pub mod tls;

pub use async_driver::{AsyncConnection, AsyncNetDriver, Middleware, NextMiddleware};
pub use async_http::{AsyncHttpDriver, AuthMiddleware, CorsMiddleware, LoggingMiddleware, RouteTrie};
pub use connection_pool::{AsyncConnectionPool, ConnectionPool, ConnectionPoolConfig, PooledConnection};
pub use http::{HttpDriver, HttpMethod, HttpRequest, HttpResponse, RouteHandler};
pub use tcp::{AsyncTcpConnection, AsyncTcpDriver, TcpConnection, TcpDriver, TcpListenerConfig};
pub use websocket::{
    AsyncWebSocketConnection, AsyncWebSocketDriver, WebSocketClient, WebSocketConfig, WebSocketConnection, WebSocketDriver,
    WebSocketMessage, WebSocketReconnectConfig,
};
