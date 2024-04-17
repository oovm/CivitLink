# GG 引擎网络模块

提供网络协议抽象层，支持 TCP、WebSocket 和 HTTP 驱动。

## 模块结构

- **同步驱动**：`NetDriver` / `Connection` trait，适用于同步 VM 环境
- **异步驱动**：`AsyncNetDriver` / `AsyncConnection` trait，基于 tokio 异步运行时
- **TCP**：同步 `TcpDriver` / `TcpConnection`，异步 `AsyncTcpDriver` / `AsyncTcpConnection`
- **WebSocket**：异步 `AsyncWebSocketDriver` / `AsyncWebSocketConnection`，支持心跳和重连
- **HTTP**：`HttpDriver` 路由分发，`AsyncHttpDriver` 异步 HTTP 服务器，中间件支持
- **连接池**：`ConnectionPool` 同步连接池，`AsyncConnectionPool` 异步连接池
- **TLS**：`TlsAcceptor` / `TlsConnector`，基于 tokio-rustls（需启用 `tls` feature）