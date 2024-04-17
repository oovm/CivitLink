//! WebSocket 网络驱动模块
//! 提供 WebSocket 协议的驱动和连接存根实现，以及基于 tokio 的异步 WebSocket 实现

use std::{future::Future, pin::Pin, time::Duration};

use futures::{SinkExt, StreamExt};
use gg_core::{GError, GErrorKind, GResult};
use tokio_tungstenite::tungstenite::protocol::Message;

use crate::{
    Connection, NetDriver,
    async_driver::{AsyncConnection, AsyncNetDriver},
};

/// WebSocket 流类型
///
/// 封装 tokio-tungstenite 的 WebSocket 流，支持 TLS 和非 TLS 连接。
pub type WsStream = tokio_tungstenite::WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>;

/// WebSocket 配置
///
/// 用于配置 WebSocket 连接的心跳、消息大小限制等参数。
pub struct WebSocketConfig {
    /// 心跳 ping 间隔
    pub ping_interval: Option<Duration>,
    /// pong 响应超时时间
    pub pong_timeout: Option<Duration>,
    /// 最大消息大小
    pub max_message_size: Option<usize>,
    /// 最大帧大小
    pub max_frame_size: Option<usize>,
}

impl Default for WebSocketConfig {
    fn default() -> Self {
        Self { ping_interval: None, pong_timeout: None, max_message_size: None, max_frame_size: None }
    }
}

/// WebSocket 重连配置
///
/// 用于配置 WebSocket 客户端的指数退避重连策略。
pub struct WebSocketReconnectConfig {
    /// 最大重连次数
    pub max_retries: u32,
    /// 初始重连延迟
    pub initial_delay: Duration,
    /// 最大重连延迟
    pub max_delay: Duration,
    /// 指数退避乘数
    pub multiplier: f64,
}

impl Default for WebSocketReconnectConfig {
    fn default() -> Self {
        Self { max_retries: 5, initial_delay: Duration::from_secs(1), max_delay: Duration::from_secs(30), multiplier: 2.0 }
    }
}

/// WebSocket 消息类型
///
/// 表示 WebSocket 协议中不同类型的消息帧。
pub enum WebSocketMessage {
    /// 文本消息
    Text(String),
    /// 二进制消息
    Binary(Vec<u8>),
    /// Ping 帧
    Ping(Vec<u8>),
    /// Pong 帧
    Pong(Vec<u8>),
    /// 关闭帧
    Close,
}

/// WebSocket 网络驱动
///
/// WebSocket 协议需要异步运行时支持，当前为存根实现。
/// 在同步 VM 环境中调用将返回错误，需要集成 tokio 运行时后才能使用。
pub struct WebSocketDriver {
    /// 监听地址
    listen_addr: Option<String>,
}

impl WebSocketDriver {
    /// 创建新的 WebSocket 驱动
    pub fn new() -> Self {
        Self { listen_addr: None }
    }
}

impl NetDriver for WebSocketDriver {
    fn listen(&mut self, addr: &str) -> GResult<()> {
        self.listen_addr = Some(addr.to_string());
        Err(GError {
            kind: GErrorKind::Runtime,
            message: "WebSocket 驱动需要 tokio 异步运行时支持，当前同步环境不可用".to_string(),
        })
    }

    fn accept(&mut self) -> GResult<Box<dyn Connection>> {
        Err(GError {
            kind: GErrorKind::Runtime,
            message: "WebSocket 驱动需要 tokio 异步运行时支持，当前同步环境不可用".to_string(),
        })
    }

    fn shutdown(&mut self) -> GResult<()> {
        self.listen_addr = None;
        Ok(())
    }
}

/// WebSocket 连接
///
/// WebSocket 协议需要异步运行时支持，当前为存根实现。
/// 在同步 VM 环境中调用将返回错误。
pub struct WebSocketConnection {
    /// 对端地址
    peer: Option<String>,
}

impl WebSocketConnection {
    /// 创建新的 WebSocket 连接存根
    pub fn new(peer: Option<String>) -> Self {
        Self { peer }
    }
}

impl Connection for WebSocketConnection {
    fn send(&mut self, _data: &[u8]) -> GResult<()> {
        Err(GError {
            kind: GErrorKind::Runtime,
            message: "WebSocket 连接需要 tokio 异步运行时支持，当前同步环境不可用".to_string(),
        })
    }

    fn recv(&mut self) -> GResult<Vec<u8>> {
        Err(GError {
            kind: GErrorKind::Runtime,
            message: "WebSocket 连接需要 tokio 异步运行时支持，当前同步环境不可用".to_string(),
        })
    }

    fn close(&mut self) -> GResult<()> {
        self.peer = None;
        Ok(())
    }

    fn peer_addr(&self) -> Option<String> {
        self.peer.clone()
    }
}

/// 异步 WebSocket 网络驱动
///
/// 基于 tokio 和 tokio-tungstenite 的异步 WebSocket 服务器驱动。
/// 实现 `AsyncNetDriver` trait，支持异步监听、接受连接和关闭。
pub struct AsyncWebSocketDriver {
    /// TCP 监听器
    listener: Option<tokio::net::TcpListener>,
    /// WebSocket 配置
    config: WebSocketConfig,
}

impl AsyncWebSocketDriver {
    /// 创建新的异步 WebSocket 驱动，使用默认配置
    pub fn new() -> Self {
        Self { listener: None, config: WebSocketConfig::default() }
    }

    /// 使用指定配置创建异步 WebSocket 驱动
    pub fn with_config(config: WebSocketConfig) -> Self {
        Self { listener: None, config }
    }
}

impl AsyncNetDriver for AsyncWebSocketDriver {
    fn listen<'a>(&'a mut self, addr: &'a str) -> Pin<Box<dyn Future<Output = GResult<()>> + Send + 'a>> {
        Box::pin(async move {
            let listener = tokio::net::TcpListener::bind(addr)
                .await
                .map_err(|e| GError { kind: GErrorKind::Network, message: format!("WebSocket 监听失败: {}", e) })?;
            self.listener = Some(listener);
            Ok(())
        })
    }

    fn accept(&mut self) -> Pin<Box<dyn Future<Output = GResult<Box<dyn AsyncConnection>>> + Send + '_>> {
        let ping_interval = self.config.ping_interval;
        let pong_timeout = self.config.pong_timeout;
        let max_message_size = self.config.max_message_size;
        let max_frame_size = self.config.max_frame_size;
        Box::pin(async move {
            let listener = self
                .listener
                .as_mut()
                .ok_or_else(|| GError { kind: GErrorKind::Network, message: "WebSocket 驱动未监听".to_string() })?;
            let (stream, addr) =
                listener.accept().await.map_err(|e| GError {
                    kind: GErrorKind::Network,
                    message: format!("WebSocket 接受 TCP 连接失败: {}", e),
                })?;
            let tls_stream = tokio_tungstenite::MaybeTlsStream::Plain(stream);
            let ws_stream = tokio_tungstenite::accept_async(tls_stream)
                .await
                .map_err(|e| GError { kind: GErrorKind::Network, message: format!("WebSocket 升级失败: {}", e) })?;
            let conn = AsyncWebSocketConnection::new(
                ws_stream,
                addr.to_string(),
                WebSocketConfig { ping_interval, pong_timeout, max_message_size, max_frame_size },
            );
            Ok(Box::new(conn) as Box<dyn AsyncConnection>)
        })
    }

    fn shutdown(&mut self) -> Pin<Box<dyn Future<Output = GResult<()>> + Send + '_>> {
        Box::pin(async move {
            self.listener = None;
            Ok(())
        })
    }
}

/// 异步 WebSocket 连接
///
/// 基于 tokio-tungstenite 的异步 WebSocket 连接。
/// 实现 `AsyncConnection` trait，支持异步收发数据和连接管理。
pub struct AsyncWebSocketConnection {
    /// WebSocket 流
    ws_stream: Option<WsStream>,
    /// 对端地址
    peer: Option<String>,
    /// WebSocket 配置
    config: WebSocketConfig,
    /// 上次收到 pong 的时间
    last_pong: Option<tokio::time::Instant>,
}

impl AsyncWebSocketConnection {
    /// 创建新的异步 WebSocket 连接
    pub fn new(ws_stream: WsStream, peer: String, config: WebSocketConfig) -> Self {
        Self { ws_stream: Some(ws_stream), peer: Some(peer), config, last_pong: Some(tokio::time::Instant::now()) }
    }

    /// 发送类型化的 WebSocket 消息
    pub async fn send_message(&mut self, msg: &WebSocketMessage) -> GResult<()> {
        let stream = self
            .ws_stream
            .as_mut()
            .ok_or_else(|| GError { kind: GErrorKind::Network, message: "WebSocket 连接已关闭".to_string() })?;
        let message = match msg {
            WebSocketMessage::Text(text) => Message::Text(text.clone().into()),
            WebSocketMessage::Binary(data) => Message::Binary(data.clone().into()),
            WebSocketMessage::Ping(data) => Message::Ping(data.clone().into()),
            WebSocketMessage::Pong(data) => Message::Pong(data.clone().into()),
            WebSocketMessage::Close => Message::Close(None),
        };
        stream
            .send(message)
            .await
            .map_err(|e| GError { kind: GErrorKind::Network, message: format!("WebSocket 发送消息失败: {}", e) })
    }

    /// 接收类型化的 WebSocket 消息
    pub async fn recv_message(&mut self) -> GResult<WebSocketMessage> {
        let stream = self
            .ws_stream
            .as_mut()
            .ok_or_else(|| GError { kind: GErrorKind::Network, message: "WebSocket 连接已关闭".to_string() })?;
        let msg = stream
            .next()
            .await
            .ok_or_else(|| GError { kind: GErrorKind::Network, message: "WebSocket 连接已关闭".to_string() })?;
        let msg =
            msg.map_err(|e| GError { kind: GErrorKind::Network, message: format!("WebSocket 接收消息失败: {}", e) })?;
        match msg {
            Message::Text(text) => Ok(WebSocketMessage::Text(text.to_string())),
            Message::Binary(data) => Ok(WebSocketMessage::Binary(data.into())),
            Message::Ping(data) => Ok(WebSocketMessage::Ping(data.into())),
            Message::Pong(data) => {
                self.last_pong = Some(tokio::time::Instant::now());
                Ok(WebSocketMessage::Pong(data.into()))
            }
            Message::Close(_) => Ok(WebSocketMessage::Close),
            Message::Frame(_) => Ok(WebSocketMessage::Binary(vec![])),
        }
    }

    /// 发送 ping 帧
    pub async fn ping(&mut self, data: &[u8]) -> GResult<()> {
        self.send_message(&WebSocketMessage::Ping(data.to_vec())).await
    }

    /// 检查 pong 是否超时
    pub fn check_pong_timeout(&self) -> bool {
        match (self.last_pong, self.config.pong_timeout) {
            (Some(last), Some(timeout)) => last.elapsed() > timeout,
            _ => false,
        }
    }
}

impl AsyncConnection for AsyncWebSocketConnection {
    fn send<'a>(&'a mut self, data: &'a [u8]) -> Pin<Box<dyn Future<Output = GResult<()>> + Send + 'a>> {
        Box::pin(async move { self.send_message(&WebSocketMessage::Binary(data.to_vec())).await })
    }

    fn recv(&mut self) -> Pin<Box<dyn Future<Output = GResult<Vec<u8>>> + Send + '_>> {
        Box::pin(async move {
            loop {
                let msg = self.recv_message().await?;
                match msg {
                    WebSocketMessage::Binary(data) => return Ok(data),
                    WebSocketMessage::Text(text) => return Ok(text.into_bytes()),
                    WebSocketMessage::Pong(_) => {
                        self.last_pong = Some(tokio::time::Instant::now());
                        continue;
                    }
                    WebSocketMessage::Ping(data) => {
                        self.send_message(&WebSocketMessage::Pong(data)).await?;
                        continue;
                    }
                    WebSocketMessage::Close => {
                        return Err(GError { kind: GErrorKind::Network, message: "WebSocket 连接已关闭".to_string() });
                    }
                }
            }
        })
    }

    fn close(&mut self) -> Pin<Box<dyn Future<Output = GResult<()>> + Send + '_>> {
        Box::pin(async move {
            if let Some(stream) = self.ws_stream.as_mut() {
                let _ = stream.send(Message::Close(None)).await;
            }
            self.ws_stream = None;
            self.peer = None;
            Ok(())
        })
    }

    fn peer_addr(&self) -> Option<String> {
        self.peer.clone()
    }
}

/// WebSocket 客户端
///
/// 支持自动重连的 WebSocket 客户端，使用指数退避策略进行重连。
pub struct WebSocketClient {
    /// 服务器 URL
    url: String,
    /// WebSocket 连接
    connection: Option<AsyncWebSocketConnection>,
    /// 重连配置
    reconnect_config: WebSocketReconnectConfig,
    /// 当前重试次数
    retry_count: u32,
}

impl WebSocketClient {
    /// 创建新的 WebSocket 客户端，使用默认重连配置
    pub fn new(url: &str) -> Self {
        Self { url: url.to_string(), connection: None, reconnect_config: WebSocketReconnectConfig::default(), retry_count: 0 }
    }

    /// 使用指定重连配置创建 WebSocket 客户端
    pub fn with_reconnect_config(url: &str, config: WebSocketReconnectConfig) -> Self {
        Self { url: url.to_string(), connection: None, reconnect_config: config, retry_count: 0 }
    }

    /// 连接到 WebSocket 服务器
    pub async fn connect(&mut self) -> GResult<()> {
        let (ws_stream, _) = tokio_tungstenite::connect_async(&self.url)
            .await
            .map_err(|e| GError { kind: GErrorKind::Network, message: format!("WebSocket 连接失败: {}", e) })?;
        self.connection = Some(AsyncWebSocketConnection::new(ws_stream, self.url.clone(), WebSocketConfig::default()));
        self.retry_count = 0;
        Ok(())
    }

    /// 使用指数退避策略重连
    pub async fn reconnect(&mut self) -> GResult<()> {
        if self.retry_count >= self.reconnect_config.max_retries {
            return Err(GError {
                kind: GErrorKind::Network,
                message: format!("WebSocket 重连失败: 已达到最大重试次数 {}", self.reconnect_config.max_retries),
            });
        }
        let delay_secs =
            self.reconnect_config.initial_delay.as_secs_f64() * self.reconnect_config.multiplier.powi(self.retry_count as i32);
        let delay = Duration::from_secs_f64(delay_secs.min(self.reconnect_config.max_delay.as_secs_f64()));
        tokio::time::sleep(delay).await;
        self.retry_count += 1;
        self.connect().await
    }

    /// 获取连接的不可变引用
    pub fn connection(&self) -> Option<&AsyncWebSocketConnection> {
        self.connection.as_ref()
    }

    /// 获取连接的可变引用
    pub fn connection_mut(&mut self) -> Option<&mut AsyncWebSocketConnection> {
        self.connection.as_mut()
    }

    /// 检查是否已连接
    pub fn is_connected(&self) -> bool {
        self.connection.is_some()
    }

    /// 重置重试计数
    pub fn reset_retry_count(&mut self) {
        self.retry_count = 0;
    }
}
