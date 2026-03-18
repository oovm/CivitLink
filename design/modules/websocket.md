# wae-websocket

WebSocket 通信模块，提供统一的 WebSocket 通信能力。本文档以实际用法为核心，帮助开发者快速上手实时通信功能。

## 概述

WebSocket 模块提供全双工实时通信能力，支持服务端和客户端模式：

- **后端 (wae-websocket)**: WebSocket 服务端，支持房间管理、广播、心跳检测
- **前端 (wae-websocket-frontend)**: WebSocket 客户端，支持自动重连、消息队列、房间管理

```
┌─────────────────────────────────────────────────────────────────┐
│                         前端 (Frontend)                          │
│  WebSocketClient: connect / send / joinRoom / 事件订阅           │
└─────────────────────────────────────────────────────────────────┘
                              │ WebSocket
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│                         后端 (Backend)                           │
│  WebSocketServer: 连接管理 / 房间管理 / 消息广播 / 心跳检测       │
└─────────────────────────────────────────────────────────────────┘
```

---

## 快速开始

### 最小服务端

```rust
use wae_websocket::{WebSocketServer, ServerConfig, ClientHandler, Connection, Message, WebSocketResult};

struct MyHandler;

impl ClientHandler for MyHandler {
    async fn on_connect(&self, conn: &Connection) -> WebSocketResult<()> {
        println!("客户端连接: {}", conn.id);
        Ok(())
    }

    async fn on_message(&self, conn: &Connection, msg: Message) -> WebSocketResult<()> {
        println!("收到消息: {:?}", msg);
        Ok(())
    }

    async fn on_disconnect(&self, conn: &Connection) {
        println!("客户端断开: {}", conn.id);
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let server = WebSocketServer::new(ServerConfig::new().port(8080));
    server.start(MyHandler).await?;
    Ok(())
}
```

### 最小客户端

```typescript
import { createWebSocketClient } from "@wae/websocket";

const client = createWebSocketClient({ url: "ws://localhost:8080/ws" });

client.on("message", (data) => {
    if (data.message.type === "Text") {
        console.log("收到消息:", data.message.text);
    }
});

await client.connect();
client.sendText("Hello, Server!");
```

---

## 服务端用法

### 启动服务

```rust
use wae_websocket::{WebSocketServer, ServerConfig};

let config = ServerConfig::new()
    .host("0.0.0.0")
    .port(8080)
    .max_connections(1000)
    .heartbeat_interval(Duration::from_secs(30))
    .connection_timeout(Duration::from_secs(60));

let server = WebSocketServer::new(config);
server.start(MyHandler).await?;
```

### 连接管理

服务端通过 `ClientHandler` trait 处理连接生命周期：

```rust
struct MyHandler {
    server: WebSocketServer,
}

impl ClientHandler for MyHandler {
    async fn on_connect(&self, conn: &Connection) -> WebSocketResult<()> {
        println!("新连接: {} 来自 {}", conn.id, conn.addr);
        println!("连接元数据: {:?}", conn.metadata);
        Ok(())
    }

    async fn on_disconnect(&self, conn: &Connection) {
        println!("连接断开: {}", conn.id);
    }
}
```

获取所有连接信息：

```rust
let conn_manager = server.connection_manager();

let count = conn_manager.count().await;
let all_ids = conn_manager.all_ids().await;

if let Some(conn) = conn_manager.get("conn-id").await {
    println!("连接地址: {}", conn.addr);
}
```

### 消息处理

在 `on_message` 中处理收到的消息：

```rust
impl ClientHandler for MyHandler {
    async fn on_message(&self, conn: &Connection, msg: Message) -> WebSocketResult<()> {
        match &msg {
            Message::Text(text) => {
                println!("文本消息: {}", text);
            }
            Message::Binary(data) => {
                println!("二进制消息: {} 字节", data.len());
            }
            _ => {}
        }
        Ok(())
    }
}
```

### 发送消息

使用 `Sender` 发送消息：

```rust
let sender = server.sender();

sender.send_to("conn-id", Message::text("单播消息")).await?;

server.broadcast(Message::text("广播给所有人")).await?;

server.broadcast_to_room("lobby", Message::text("房间广播")).await?;
```

### 房间管理

使用 `RoomManager` 管理房间：

```rust
let room_manager = server.room_manager();

room_manager.create_room("lobby").await;

room_manager.join("lobby", "conn-1").await;
room_manager.join("lobby", "conn-2").await;

let members = room_manager.get_members("lobby").await;
println!("房间成员: {:?}", members);

let count = room_manager.member_count("lobby").await;

room_manager.leave("lobby", "conn-1").await;

room_manager.delete_room("lobby").await;
```

### 关闭服务

```rust
server.shutdown();
```

---

## 客户端用法

### 创建连接

```typescript
import { createWebSocketClient } from "@wae/websocket";

const client = createWebSocketClient({
    url: "ws://localhost:8080/ws",
    reconnectInterval: 3000,
    heartbeatInterval: 25000,
    connectionTimeout: 10000,
    maxReconnectAttempts: 5,
});

await client.connect();

console.log("连接状态:", client.getState());
console.log("连接 ID:", client.getConnectionId());
```

### 发送消息

```typescript
client.sendText("文本消息");

client.sendBinary(new Uint8Array([1, 2, 3, 4]));

client.sendJson({ type: "chat", content: "Hello", timestamp: Date.now() });

import { textMessage, binaryMessage } from "@wae/types";
client.send(textMessage("使用消息对象"));
```

### 接收消息

```typescript
client.on("message", (data) => {
    const { message, connection } = data;
    
    switch (message.type) {
        case "Text":
            console.log("文本:", message.text);
            break;
        case "Binary":
            console.log("二进制:", message.binary);
            break;
    }
});
```

### 房间管理

```typescript
await client.joinRoom("lobby");
await client.joinRoom("game-room");

console.log("当前房间:", client.getRooms());

await client.leaveRoom("lobby");
```

### 事件订阅

客户端支持以下事件：

```typescript
client.on("connect", (data) => {
    console.log("连接成功:", data.connection.id);
});

client.on("disconnect", (data) => {
    console.log("断开连接:", data.reason);
});

client.on("message", (data) => {
    console.log("收到消息:", data.message);
});

client.on("error", (data) => {
    console.error("连接错误:", data.error);
});

client.on("reconnect", (data) => {
    console.log(`重连中: ${data.attempt}/${data.maxAttempts}`);
});

client.on("reconnect_failed", (data) => {
    console.error("重连失败:", data.totalAttempts);
});
```

取消订阅：

```typescript
const handler = (data) => console.log(data);
client.on("message", handler);
client.off("message", handler);

client.off("message");
```

### 断开连接

```typescript
await client.disconnect();
```

---

## 完整示例：聊天室

### 服务端实现

```rust
use wae_websocket::{WebSocketServer, ServerConfig, ClientHandler, Connection, Message, WebSocketResult};
use std::sync::Arc;

struct ChatHandler {
    server: Arc<WebSocketServer>,
}

impl ClientHandler for ChatHandler {
    async fn on_connect(&self, conn: &Connection) -> WebSocketResult<()> {
        self.server.room_manager().create_room("chat").await;
        self.server.room_manager().join("chat", &conn.id).await;
        
        let welcome = format!("用户 {} 加入了聊天室", conn.id);
        self.server.broadcast_to_room("chat", Message::text(welcome)).await?;
        
        Ok(())
    }

    async fn on_message(&self, conn: &Connection, msg: Message) -> WebSocketResult<()> {
        if let Message::Text(text) = msg {
            let broadcast_msg = format!("{}: {}", conn.id, text);
            self.server.broadcast_to_room("chat", Message::text(broadcast_msg)).await?;
        }
        Ok(())
    }

    async fn on_disconnect(&self, conn: &Connection) {
        let _ = self.server.room_manager().leave("chat", &conn.id).await;
        
        let goodbye = format!("用户 {} 离开了聊天室", conn.id);
        let _ = self.server.broadcast_to_room("chat", Message::text(goodbye)).await;
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = ServerConfig::new().port(9000);
    let server = Arc::new(WebSocketServer::new(config));
    let handler = ChatHandler { server: server.clone() };
    
    println!("聊天室服务启动在端口 9000");
    server.start(handler).await?;
    Ok(())
}
```

### 前端实现

```typescript
import { createWebSocketClient } from "@wae/websocket";

const client = createWebSocketClient({
    url: "ws://localhost:9000/ws",
    maxReconnectAttempts: 3,
});

const messages: string[] = [];

client.on("connect", () => {
    console.log("已连接到聊天室");
});

client.on("message", (data) => {
    if (data.message.type === "Text") {
        messages.push(data.message.text!);
        renderMessages();
    }
});

client.on("disconnect", (data) => {
    console.log("断开连接:", data.reason);
});

client.on("reconnect", (data) => {
    console.log(`正在重连 (${data.attempt}/${data.maxAttempts})`);
});

await client.connect();

function sendMessage(text: string) {
    if (text.trim()) {
        client.sendText(text.trim());
    }
}

function renderMessages() {
    const container = document.getElementById("messages");
    container.innerHTML = messages.map(m => `<div>${m}</div>`).join("");
}

document.getElementById("send-btn").onclick = () => {
    const input = document.getElementById("msg-input") as HTMLInputElement;
    sendMessage(input.value);
    input.value = "";
};
```

---

## 心跳与重连

### 服务端心跳检测

服务端自动进行心跳检测，超时未响应的连接将被关闭：

```rust
let config = ServerConfig::new()
    .heartbeat_interval(Duration::from_secs(30))
    .connection_timeout(Duration::from_secs(60));
```

- `heartbeat_interval`: 发送心跳的间隔
- `connection_timeout`: 连接超时时间，超时后自动断开

### 客户端自动重连

前端内置自动重连机制：

```typescript
const client = createWebSocketClient({
    url: "ws://localhost:8080/ws",
    reconnectInterval: 3000,
    maxReconnectAttempts: 0,
});

client.on("reconnect", (data) => {
    console.log(`第 ${data.attempt} 次重连`);
});

client.on("reconnect_failed", (data) => {
    console.error(`重连失败，共尝试 ${data.totalAttempts} 次`);
});
```

- `reconnectInterval`: 重连间隔（毫秒）
- `maxReconnectAttempts`: 最大重连次数，`0` 表示无限重连

### 客户端心跳保活

```typescript
const client = createWebSocketClient({
    url: "ws://localhost:8080/ws",
    heartbeatInterval: 25000,
});
```

客户端会自动发送心跳包保持连接活跃。

---

## 最佳实践

### 服务端

**1. 使用 Arc 共享 Server**

```rust
use std::sync::Arc;

let server = Arc::new(WebSocketServer::new(config));
let handler = MyHandler { server: server.clone() };
```

**2. 在 on_disconnect 中清理资源**

```rust
async fn on_disconnect(&self, conn: &Connection) {
    let _ = self.server.room_manager().leave_all(&conn.id).await;
}
```

**3. 错误处理**

```rust
async fn on_message(&self, conn: &Connection, msg: Message) -> WebSocketResult<()> {
    match process_message(msg) {
        Ok(_) => Ok(()),
        Err(e) => {
            eprintln!("处理消息错误: {}", e);
            Ok(())
        }
    }
}
```

**4. 限制消息大小**

在处理二进制消息时检查大小：

```rust
if let Message::Binary(data) = &msg {
    if data.len() > 1024 * 1024 {
        return Ok(());
    }
}
```

### 客户端

**1. 合理设置重连参数**

```typescript
const client = createWebSocketClient({
    url: "ws://localhost:8080/ws",
    reconnectInterval: 3000,
    maxReconnectAttempts: 10,
});
```

**2. 处理所有事件**

```typescript
client.on("connect", handleConnect);
client.on("disconnect", handleDisconnect);
client.on("message", handleMessage);
client.on("error", handleError);
client.on("reconnect", handleReconnect);
client.on("reconnect_failed", handleReconnectFailed);
```

**3. 断开时清理**

```typescript
window.addEventListener("beforeunload", () => {
    client.disconnect();
});
```

**4. 消息队列**

断线期间的消息可以在重连后重发：

```typescript
const pendingMessages: string[] = [];

client.on("connect", () => {
    while (pendingMessages.length > 0) {
        const msg = pendingMessages.shift()!;
        client.sendText(msg);
    }
});

function safeSend(text: string) {
    if (client.getState() === "connected") {
        client.sendText(text);
    } else {
        pendingMessages.push(text);
    }
}
```

---

## 常见问题

### Q: 如何获取客户端 IP 地址？

```rust
async fn on_connect(&self, conn: &Connection) -> WebSocketResult<()> {
    println!("客户端 IP: {}", conn.addr);
    Ok(())
}
```

### Q: 如何实现私聊？

```rust
async fn on_message(&self, conn: &Connection, msg: Message) -> WebSocketResult<()> {
    if let Message::Text(text) = msg {
        let parts: Vec<&str> = text.splitn(2, ' ').collect();
        if parts[0] == "/pm" {
            let target = parts.get(1).unwrap();
            self.server.sender().send_to(target, Message::text("私聊消息")).await?;
        }
    }
    Ok(())
}
```

### Q: 如何限制单个用户连接数？

```rust
async fn on_connect(&self, conn: &Connection) -> WebSocketResult<()> {
    let user_id = conn.metadata.get("user_id").unwrap();
    let count = self.server.connection_manager().count().await;
    if count > MAX_PER_USER {
        return Err(WebSocketError::MaxConnectionsExceeded(MAX_PER_USER));
    }
    Ok(())
}
```

### Q: 客户端如何判断连接状态？

```typescript
if (client.getState() === "connected") {
    client.sendText("消息");
}
```

### Q: 如何处理 JSON 消息？

服务端：

```rust
if let Message::Text(text) = msg {
    let data: MyMessage = serde_json::from_str(&text)?;
}
```

客户端：

```typescript
client.sendJson({ type: "action", payload: { id: 123 } });

client.on("message", (data) => {
    if (data.message.type === "Text") {
        const json = JSON.parse(data.message.text!);
    }
});
```

### Q: 如何优雅关闭服务？

```rust
tokio::select! {
    _ = shutdown_signal() => {
        println!("正在关闭服务...");
        server.shutdown();
    }
    result = server.start(handler) => {
        result?;
    }
}
```

### Q: 连接超时时间如何选择？

- 心跳间隔建议 25-30 秒
- 连接超时建议为心跳间隔的 2 倍
- 重连间隔建议 3-5 秒，可使用指数退避

```rust
let config = ServerConfig::new()
    .heartbeat_interval(Duration::from_secs(30))
    .connection_timeout(Duration::from_secs(60));
```

```typescript
const client = createWebSocketClient({
    url: "ws://localhost:8080/ws",
    heartbeatInterval: 30000,
    reconnectInterval: 3000,
});
```
