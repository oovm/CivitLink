//! WAE WebSocket - 实时通信抽象层
//!
//! 提供统一的 WebSocket 通信能力，支持服务端和客户端模式。
//!
//! 深度融合 tokio 运行时，所有 API 都是异步优先设计。
//! 微服务架构友好，支持房间管理、广播、自动重连、心跳检测等特性。

#![warn(missing_docs)]

use async_trait::async_trait;
use futures_util::{SinkExt, StreamExt};
use serde::{Serialize, de::DeserializeOwned};
use std::{collections::HashMap, net::SocketAddr, sync::Arc, time::Duration};
use tokio::sync::{RwLock, broadcast, mpsc};
use tokio_tungstenite::tungstenite::protocol::Message as WsMessage;
use wae_types::{WaeError, WaeErrorKind, WaeResult};

/// 连接 ID 类型
pub type ConnectionId = String;

/// 房间 ID 类型
pub type RoomId = String;

/// WebSocket 消息类型
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Message {
    /// 文本消息
    Text(String),
    /// 二进制消息
    Binary(Vec<u8>),
    /// Ping 消息
    Ping,
    /// Pong 消息
    Pong,
    /// 关闭消息
    Close,
}

impl Message {
    /// 创建文本消息
    pub fn text(content: impl Into<String>) -> Self {
        Message::Text(content.into())
    }

    /// 创建二进制消息
    pub fn binary(data: impl Into<Vec<u8>>) -> Self {
        Message::Binary(data.into())
    }

    /// 检查是否为文本消息
    pub fn is_text(&self) -> bool {
        matches!(self, Message::Text(_))
    }

    /// 检查是否为二进制消息
    pub fn is_binary(&self) -> bool {
        matches!(self, Message::Binary(_))
    }

    /// 获取文本内容
    pub fn as_text(&self) -> Option<&str> {
        match self {
            Message::Text(s) => Some(s),
            _ => None,
        }
    }

    /// 获取二进制内容
    pub fn as_binary(&self) -> Option<&[u8]> {
        match self {
            Message::Binary(data) => Some(data),
            _ => None,
        }
    }
}

impl From<WsMessage> for Message {
    fn from(msg: WsMessage) -> Self {
        match msg {
            WsMessage::Text(s) => Message::Text(s.to_string()),
            WsMessage::Binary(data) => Message::Binary(data.to_vec()),
            WsMessage::Ping(_) => Message::Ping,
            WsMessage::Pong(_) => Message::Pong,
            WsMessage::Close(_) => Message::Close,
            _ => Message::Close,
        }
    }
}

impl From<Message> for WsMessage {
    fn from(msg: Message) -> Self {
        match msg {
            Message::Text(s) => WsMessage::Text(s.into()),
            Message::Binary(data) => WsMessage::Binary(data.into()),
            Message::Ping => WsMessage::Ping(Vec::new().into()),
            Message::Pong => WsMessage::Pong(Vec::new().into()),
            Message::Close => WsMessage::Close(None),
        }
    }
}

/// 连接信息
#[derive(Debug, Clone)]
pub struct Connection {
    /// 连接 ID
    pub id: ConnectionId,
    /// 客户端地址
    pub addr: SocketAddr,
    /// 连接时间
    pub connected_at: std::time::Instant,
    /// 用户自定义数据
    pub metadata: HashMap<String, String>,
    /// 所属房间列表
    pub rooms: Vec<RoomId>,
}

impl Connection {
    /// 创建新连接
    pub fn new(id: ConnectionId, addr: SocketAddr) -> Self {
        Self { id, addr, connected_at: std::time::Instant::now(), metadata: HashMap::new(), rooms: Vec::new() }
    }

    /// 设置元数据
    pub fn with_metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }

    /// 获取连接持续时间
    pub fn duration(&self) -> Duration {
        self.connected_at.elapsed()
    }
}

/// 连接管理器
pub struct ConnectionManager {
    connections: Arc<RwLock<HashMap<ConnectionId, Connection>>>,
    max_connections: u32,
}

impl ConnectionManager {
    /// 创建新的连接管理器
    pub fn new(max_connections: u32) -> Self {
        Self { connections: Arc::new(RwLock::new(HashMap::new())), max_connections }
    }

    /// 添加连接
    pub async fn add(&self, connection: Connection) -> WaeResult<()> {
        let mut connections = self.connections.write().await;
        if connections.len() >= self.max_connections as usize {
            return Err(WaeError::new(WaeErrorKind::ResourceConflict {
                resource: "Connection".to_string(),
                reason: format!("Maximum connections ({}) exceeded", self.max_connections),
            }));
        }
        connections.insert(connection.id.clone(), connection);
        Ok(())
    }

    /// 移除连接
    pub async fn remove(&self, id: &str) -> Option<Connection> {
        let mut connections = self.connections.write().await;
        connections.remove(id)
    }

    /// 获取连接
    pub async fn get(&self, id: &str) -> Option<Connection> {
        let connections = self.connections.read().await;
        connections.get(id).cloned()
    }

    /// 检查连接是否存在
    pub async fn exists(&self, id: &str) -> bool {
        let connections = self.connections.read().await;
        connections.contains_key(id)
    }

    /// 获取连接数量
    pub async fn count(&self) -> usize {
        let connections = self.connections.read().await;
        connections.len()
    }

    /// 获取所有连接 ID
    pub async fn all_ids(&self) -> Vec<ConnectionId> {
        let connections = self.connections.read().await;
        connections.keys().cloned().collect()
    }

    /// 更新连接的房间列表
    pub async fn join_room(&self, id: &str, room: &str) -> WaeResult<()> {
        let mut connections = self.connections.write().await;
        if let Some(conn) = connections.get_mut(id) {
            if !conn.rooms.contains(&room.to_string()) {
                conn.rooms.push(room.to_string());
            }
            return Ok(());
        }
        Err(WaeError::not_found("Connection", id))
    }

    /// 离开房间
    pub async fn leave_room(&self, id: &str, room: &str) -> WaeResult<()> {
        let mut connections = self.connections.write().await;
        if let Some(conn) = connections.get_mut(id) {
            conn.rooms.retain(|r| r != room);
            return Ok(());
        }
        Err(WaeError::not_found("Connection", id))
    }
}

/// 房间管理器
pub struct RoomManager {
    rooms: Arc<RwLock<HashMap<RoomId, Vec<ConnectionId>>>>,
}

impl RoomManager {
    /// 创建新的房间管理器
    pub fn new() -> Self {
        Self { rooms: Arc::new(RwLock::new(HashMap::new())) }
    }

    /// 创建房间
    pub async fn create_room(&self, room_id: &str) {
        let mut rooms = self.rooms.write().await;
        rooms.entry(room_id.to_string()).or_insert_with(Vec::new);
    }

    /// 删除房间
    pub async fn delete_room(&self, room_id: &str) -> Option<Vec<ConnectionId>> {
        let mut rooms = self.rooms.write().await;
        rooms.remove(room_id)
    }

    /// 加入房间
    pub async fn join(&self, room_id: &str, connection_id: &str) {
        let mut rooms = self.rooms.write().await;
        let room = rooms.entry(room_id.to_string()).or_insert_with(Vec::new);
        if !room.contains(&connection_id.to_string()) {
            room.push(connection_id.to_string());
        }
    }

    /// 离开房间
    pub async fn leave(&self, room_id: &str, connection_id: &str) {
        let mut rooms = self.rooms.write().await;
        if let Some(room) = rooms.get_mut(room_id) {
            room.retain(|id| id != connection_id);
            if room.is_empty() {
                rooms.remove(room_id);
            }
        }
    }

    /// 获取房间内的所有连接
    pub async fn get_members(&self, room_id: &str) -> Vec<ConnectionId> {
        let rooms = self.rooms.read().await;
        rooms.get(room_id).cloned().unwrap_or_default()
    }

    /// 检查房间是否存在
    pub async fn room_exists(&self, room_id: &str) -> bool {
        let rooms = self.rooms.read().await;
        rooms.contains_key(room_id)
    }

    /// 获取房间数量
    pub async fn room_count(&self) -> usize {
        let rooms = self.rooms.read().await;
        rooms.len()
    }

    /// 获取房间成员数量
    pub async fn member_count(&self, room_id: &str) -> usize {
        let rooms = self.rooms.read().await;
        rooms.get(room_id).map(|r| r.len()).unwrap_or(0)
    }

    /// 广播消息到房间
    pub async fn broadcast(&self, room_id: &str, sender: &Sender, message: &Message) -> WaeResult<Vec<ConnectionId>> {
        let members = self.get_members(room_id).await;
        let mut sent_to = Vec::new();
        for conn_id in &members {
            if sender.send_to(conn_id, message.clone()).await.is_ok() {
                sent_to.push(conn_id.clone());
            }
        }
        Ok(sent_to)
    }
}

impl Default for RoomManager {
    fn default() -> Self {
        Self::new()
    }
}

/// 消息发送器
#[derive(Clone)]
pub struct Sender {
    senders: Arc<RwLock<HashMap<ConnectionId, mpsc::UnboundedSender<Message>>>>,
}

impl Sender {
    /// 创建新的发送器
    pub fn new() -> Self {
        Self { senders: Arc::new(RwLock::new(HashMap::new())) }
    }

    /// 注册连接的发送通道
    pub async fn register(&self, connection_id: ConnectionId, sender: mpsc::UnboundedSender<Message>) {
        let mut senders = self.senders.write().await;
        senders.insert(connection_id, sender);
    }

    /// 注销连接的发送通道
    pub async fn unregister(&self, connection_id: &str) {
        let mut senders = self.senders.write().await;
        senders.remove(connection_id);
    }

    /// 发送消息到指定连接
    pub async fn send_to(&self, connection_id: &str, message: Message) -> WaeResult<()> {
        let senders = self.senders.read().await;
        if let Some(sender) = senders.get(connection_id) {
            sender
                .send(message)
                .map_err(|e| WaeError::new(WaeErrorKind::InternalError { reason: format!("Send failed: {}", e) }))?;
            return Ok(());
        }
        Err(WaeError::not_found("Connection", connection_id))
    }

    /// 广播消息到所有连接
    pub async fn broadcast(&self, message: Message) -> WaeResult<usize> {
        let senders = self.senders.read().await;
        let mut count = 0;
        for sender in senders.values() {
            if sender.send(message.clone()).is_ok() {
                count += 1;
            }
        }
        Ok(count)
    }

    /// 获取连接数量
    pub async fn count(&self) -> usize {
        let senders = self.senders.read().await;
        senders.len()
    }
}

impl Default for Sender {
    fn default() -> Self {
        Self::new()
    }
}

/// 服务端配置
#[derive(Debug, Clone)]
pub struct ServerConfig {
    /// 监听地址
    pub host: String,
    /// 监听端口
    pub port: u16,
    /// 最大连接数
    pub max_connections: u32,
    /// 心跳间隔
    pub heartbeat_interval: Duration,
    /// 连接超时
    pub connection_timeout: Duration,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            host: "0.0.0.0".to_string(),
            port: 8080,
            max_connections: 1000,
            heartbeat_interval: Duration::from_secs(30),
            connection_timeout: Duration::from_secs(60),
        }
    }
}

impl ServerConfig {
    /// 创建新的服务端配置
    pub fn new() -> Self {
        Self::default()
    }

    /// 设置监听地址
    pub fn host(mut self, host: impl Into<String>) -> Self {
        self.host = host.into();
        self
    }

    /// 设置监听端口
    pub fn port(mut self, port: u16) -> Self {
        self.port = port;
        self
    }

    /// 设置最大连接数
    pub fn max_connections(mut self, max: u32) -> Self {
        self.max_connections = max;
        self
    }

    /// 设置心跳间隔
    pub fn heartbeat_interval(mut self, interval: Duration) -> Self {
        self.heartbeat_interval = interval;
        self
    }
}

/// 客户端处理器 trait
#[async_trait]
pub trait ClientHandler: Send + Sync {
    /// 连接建立时调用
    async fn on_connect(&self, connection: &Connection) -> WaeResult<()>;

    /// 收到消息时调用
    async fn on_message(&self, connection: &Connection, message: Message) -> WaeResult<()>;

    /// 连接关闭时调用
    async fn on_disconnect(&self, connection: &Connection);
}

/// 默认客户端处理器
pub struct DefaultClientHandler;

#[async_trait]
impl ClientHandler for DefaultClientHandler {
    async fn on_connect(&self, _connection: &Connection) -> WaeResult<()> {
        Ok(())
    }

    async fn on_message(&self, _connection: &Connection, _message: Message) -> WaeResult<()> {
        Ok(())
    }

    async fn on_disconnect(&self, _connection: &Connection) {}
}

/// WebSocket 服务端
pub struct WebSocketServer {
    config: ServerConfig,
    connection_manager: Arc<ConnectionManager>,
    room_manager: Arc<RoomManager>,
    sender: Sender,
    shutdown_tx: broadcast::Sender<()>,
}

impl WebSocketServer {
    /// 创建新的 WebSocket 服务端
    pub fn new(config: ServerConfig) -> Self {
        let (shutdown_tx, _) = broadcast::channel(1);
        Self {
            config,
            connection_manager: Arc::new(ConnectionManager::new(1000)),
            room_manager: Arc::new(RoomManager::new()),
            sender: Sender::new(),
            shutdown_tx,
        }
    }

    /// 获取连接管理器
    pub fn connection_manager(&self) -> &Arc<ConnectionManager> {
        &self.connection_manager
    }

    /// 获取房间管理器
    pub fn room_manager(&self) -> &Arc<RoomManager> {
        &self.room_manager
    }

    /// 获取消息发送器
    pub fn sender(&self) -> &Sender {
        &self.sender
    }

    /// 获取配置
    pub fn config(&self) -> &ServerConfig {
        &self.config
    }

    /// 启动服务端
    pub async fn start<H: ClientHandler + 'static>(&self, handler: H) -> WaeResult<()> {
        let addr = format!("{}:{}", self.config.host, self.config.port);
        let listener = tokio::net::TcpListener::bind(&addr)
            .await
            .map_err(|_e| WaeError::new(WaeErrorKind::ConnectionFailed { target: addr.clone() }))?;

        tracing::info!("WebSocket server listening on {}", addr);

        let mut shutdown_rx = self.shutdown_tx.subscribe();
        let handler = Arc::new(handler);

        loop {
            tokio::select! {
                accept_result = listener.accept() => {
                    match accept_result {
                        Ok((stream, addr)) => {
                            let connection_manager = self.connection_manager.clone();
                            let room_manager = self.room_manager.clone();
                            let sender = self.sender.clone();
                            let handler = handler.clone();
                            let config = self.config.clone();

                            tokio::spawn(async move {
                                if let Err(e) = Self::handle_connection(
                                    stream,
                                    addr,
                                    connection_manager,
                                    room_manager,
                                    sender,
                                    handler,
                                    config,
                                ).await {
                                    tracing::error!("Connection error: {}", e);
                                }
                            });
                        }
                        Err(e) => {
                            tracing::error!("Accept error: {}", e);
                        }
                    }
                }
                _ = shutdown_rx.recv() => {
                    tracing::info!("WebSocket server shutting down");
                    break;
                }
            }
        }

        Ok(())
    }

    async fn handle_connection<H: ClientHandler>(
        stream: tokio::net::TcpStream,
        addr: SocketAddr,
        connection_manager: Arc<ConnectionManager>,
        room_manager: Arc<RoomManager>,
        sender: Sender,
        handler: Arc<H>,
        config: ServerConfig,
    ) -> WaeResult<()> {
        let ws_stream = tokio_tungstenite::accept_async(stream)
            .await
            .map_err(|_e| WaeError::new(WaeErrorKind::ConnectionFailed { target: addr.to_string() }))?;

        let connection_id = uuid::Uuid::new_v4().to_string();
        let connection = Connection::new(connection_id.clone(), addr);

        if connection_manager.add(connection.clone()).await.is_err() {
            return Err(WaeError::new(WaeErrorKind::ResourceConflict {
                resource: "Connection".to_string(),
                reason: format!("Maximum connections ({}) exceeded", config.max_connections),
            }));
        }

        handler.on_connect(&connection).await?;
        tracing::info!("Client connected: {} from {}", connection_id, addr);

        let (ws_sender, mut ws_receiver) = ws_stream.split();
        let (tx, mut rx) = mpsc::unbounded_channel::<Message>();

        sender.register(connection_id.clone(), tx).await;

        let send_task = async move {
            let mut ws_sender = ws_sender;
            while let Some(msg) = rx.recv().await {
                if ws_sender.send(msg.into()).await.is_err() {
                    break;
                }
            }
            let _ = ws_sender.close().await;
        };

        let connection_manager_clone = connection_manager.clone();
        let room_manager_clone = room_manager.clone();
        let sender_clone = sender.clone();
        let connection_id_clone = connection_id.clone();
        let connection_clone = connection.clone();
        let handler_clone = handler.clone();
        let recv_task = async move {
            while let Some(msg_result) = ws_receiver.next().await {
                match msg_result {
                    Ok(ws_msg) => {
                        let msg: Message = ws_msg.into();
                        if matches!(msg, Message::Close) {
                            break;
                        }
                        if handler_clone.on_message(&connection_clone, msg).await.is_err() {
                            break;
                        }
                    }
                    Err(_) => break,
                }
            }
        };

        tokio::select! {
            _ = send_task => {},
            _ = recv_task => {},
        }

        for room_id in &connection.rooms {
            room_manager_clone.leave(room_id, &connection_id_clone).await;
        }

        connection_manager_clone.remove(&connection_id_clone).await;
        sender_clone.unregister(&connection_id_clone).await;
        handler.on_disconnect(&connection).await;

        tracing::info!("Client disconnected: {}", connection_id);

        Ok(())
    }

    /// 停止服务端
    pub fn shutdown(&self) {
        let _ = self.shutdown_tx.send(());
    }

    /// 广播消息到所有连接
    pub async fn broadcast(&self, message: Message) -> WaeResult<usize> {
        self.sender.broadcast(message).await
    }

    /// 广播消息到房间
    pub async fn broadcast_to_room(&self, room_id: &str, message: Message) -> WaeResult<Vec<ConnectionId>> {
        self.room_manager.broadcast(room_id, &self.sender, &message).await
    }
}

/// 客户端配置
#[derive(Debug, Clone)]
pub struct ClientConfig {
    /// 服务端 URL
    pub url: String,
    /// 重连间隔
    pub reconnect_interval: Duration,
    /// 心跳间隔
    pub heartbeat_interval: Duration,
    /// 连接超时
    pub connection_timeout: Duration,
    /// 最大重连次数 (0 表示无限重连)
    pub max_reconnect_attempts: u32,
}

impl Default for ClientConfig {
    fn default() -> Self {
        Self {
            url: "ws://127.0.0.1:8080".to_string(),
            reconnect_interval: Duration::from_secs(5),
            heartbeat_interval: Duration::from_secs(30),
            connection_timeout: Duration::from_secs(10),
            max_reconnect_attempts: 0,
        }
    }
}

impl ClientConfig {
    /// 创建新的客户端配置
    pub fn new(url: impl Into<String>) -> Self {
        Self { url: url.into(), ..Self::default() }
    }

    /// 设置重连间隔
    pub fn reconnect_interval(mut self, interval: Duration) -> Self {
        self.reconnect_interval = interval;
        self
    }

    /// 设置心跳间隔
    pub fn heartbeat_interval(mut self, interval: Duration) -> Self {
        self.heartbeat_interval = interval;
        self
    }

    /// 设置最大重连次数
    pub fn max_reconnect_attempts(mut self, attempts: u32) -> Self {
        self.max_reconnect_attempts = attempts;
        self
    }
}

/// WebSocket 客户端
pub struct WebSocketClient {
    config: ClientConfig,
    sender: mpsc::UnboundedSender<Message>,
    receiver: mpsc::UnboundedReceiver<Message>,
}

impl WebSocketClient {
    /// 创建新的 WebSocket 客户端
    pub fn new(config: ClientConfig) -> Self {
        let (outgoing_tx, mut outgoing_rx) = mpsc::unbounded_channel::<Message>();
        let (incoming_tx, incoming_rx) = mpsc::unbounded_channel::<Message>();

        let config_clone = config.clone();

        tokio::spawn(async move {
            let mut attempt = 0u32;
            loop {
                match tokio_tungstenite::connect_async(&config_clone.url).await {
                    Ok((ws_stream, _)) => {
                        tracing::info!("WebSocket client connected to {}", config_clone.url);
                        attempt = 0;

                        let (mut ws_sender, mut ws_receiver) = ws_stream.split();

                        let send_task = async {
                            while let Some(msg) = outgoing_rx.recv().await {
                                if ws_sender.send(msg.into()).await.is_err() {
                                    break;
                                }
                            }
                        };

                        let recv_task = async {
                            while let Some(msg_result) = ws_receiver.next().await {
                                match msg_result {
                                    Ok(ws_msg) => {
                                        let msg: Message = ws_msg.into();
                                        if matches!(msg, Message::Close) {
                                            break;
                                        }
                                        if incoming_tx.send(msg).is_err() {
                                            break;
                                        }
                                    }
                                    Err(_) => break,
                                }
                            }
                        };

                        tokio::select! {
                            _ = send_task => {},
                            _ = recv_task => {},
                        }

                        tracing::warn!("WebSocket client disconnected, attempting to reconnect...");
                    }
                    Err(e) => {
                        tracing::error!("WebSocket connection failed: {}", e);
                    }
                }

                attempt += 1;
                if config_clone.max_reconnect_attempts > 0 && attempt >= config_clone.max_reconnect_attempts {
                    tracing::error!("Max reconnect attempts reached, giving up");
                    break;
                }

                tokio::time::sleep(config_clone.reconnect_interval).await;
            }
        });

        Self { config, sender: outgoing_tx, receiver: incoming_rx }
    }

    /// 发送消息
    pub async fn send(&self, message: Message) -> WaeResult<()> {
        self.sender
            .send(message)
            .map_err(|e| WaeError::new(WaeErrorKind::InternalError { reason: format!("Send failed: {}", e) }))
    }

    /// 发送文本消息
    pub async fn send_text(&self, text: impl Into<String>) -> WaeResult<()> {
        self.send(Message::text(text)).await
    }

    /// 发送二进制消息
    pub async fn send_binary(&self, data: impl Into<Vec<u8>>) -> WaeResult<()> {
        self.send(Message::binary(data)).await
    }

    /// 发送 JSON 消息
    pub async fn send_json<T: Serialize + ?Sized>(&self, value: &T) -> WaeResult<()> {
        let json = serde_json::to_string(value).map_err(|_e| WaeError::serialization_failed("JSON"))?;
        self.send_text(json).await
    }

    /// 接收消息
    pub async fn receive(&mut self) -> Option<Message> {
        self.receiver.recv().await
    }

    /// 接收并解析 JSON 消息
    pub async fn receive_json<T: DeserializeOwned>(&mut self) -> WaeResult<Option<T>> {
        match self.receive().await {
            Some(msg) => {
                let text = msg.as_text().ok_or_else(|| WaeError::deserialization_failed("Expected text message"))?;
                let value: T = serde_json::from_str(text).map_err(|_e| WaeError::deserialization_failed("JSON"))?;
                Ok(Some(value))
            }
            None => Ok(None),
        }
    }

    /// 获取配置
    pub fn config(&self) -> &ClientConfig {
        &self.config
    }

    /// 关闭连接
    pub async fn close(&self) -> WaeResult<()> {
        self.send(Message::Close).await
    }
}

/// 便捷函数：创建 WebSocket 服务端
pub fn websocket_server(config: ServerConfig) -> WebSocketServer {
    WebSocketServer::new(config)
}

/// 便捷函数：创建 WebSocket 客户端
pub fn websocket_client(config: ClientConfig) -> WebSocketClient {
    WebSocketClient::new(config)
}
