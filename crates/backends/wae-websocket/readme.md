# wae-websocket

WebSocket 模块 - 提供实时通信抽象层。

## 主要功能

- **服务端/客户端**: 统一的 WebSocket API
- **房间管理**: 支持房间广播
- **连接管理**: 连接状态跟踪
- **自动重连**: 客户端断线重连
- **心跳检测**: 保持连接活跃

## 技术栈

- **WebSocket**: tokio-tungstenite
- **异步运行时**: Tokio
- **序列化**: serde

## 服务端示例

```rust
use wae_websocket::{WebSocketServer, ServerConfig, ClientHandler, Connection, Message};

struct MyHandler;


impl ClientHandler for MyHandler {
    async fn on_connect(&self, conn: &Connection) -> Result<()> {
        println!("客户端连接: {}", conn.id);
        Ok(())
    }
    
    async fn on_message(&self, conn: &Connection, msg: Message) -> Result<()> {
        println!("收到消息: {:?}", msg);
        Ok(())
    }
    
    async fn on_disconnect(&self, conn: &Connection) {
        println!("客户端断开: {}", conn.id);
    }
}

#[tokio::main]
async fn main() {
    let server = WebSocketServer::new(ServerConfig {
        port: 8080,
        max_connections: 1000,
        ..Default::default()
    });
    
    server.start(MyHandler).await?;
}
```

## 客户端示例

```rust
use wae_websocket::{WebSocketClient, ClientConfig, Message};

#[tokio::main]
async fn main() {
    let mut client = WebSocketClient::new(ClientConfig::new("ws://localhost:8080"));
    
    client.send_text("Hello, Server!").await?;
    
    while let Some(msg) = client.receive().await {
        println!("收到: {:?}", msg);
    }
}
```

## 房间广播

```rust
let server = WebSocketServer::new(config);

server.room_manager().join("room-1", "conn-001").await;
server.broadcast_to_room("room-1", Message::text("大家好")).await?;
```
