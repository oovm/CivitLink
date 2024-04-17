# GG Runtime Net

GG Runtime Net provides networking functionality for the GG engine runtime.

## Features
- TCP/IP support
- WebSocket integration
- HTTP client
- Network protocol handling
- Asynchronous networking

## Dependencies
- gg-core
- tokio
- tokio-tungstenite

## Usage

```rust
use gg_runtime_net::*;

// Create network client
let client = NetworkClient::new();

// Connect to server
let connection = client.connect("ws://localhost:8080").await;

// Send message
connection.send("Hello, server!");

// Receive message
let message = connection.receive().await;
```
