# WAE HTTPS - HTTPS 服务核心模块

提供统一的 HTTP/HTTPS 服务抽象，支持中间件、路由、请求处理等核心功能。

深度融合 tokio 运行时，支持：
- 异步请求处理
- 中间件链式处理
- 统一错误处理
- 请求追踪与日志
- TLS/HTTPS 支持
- HTTP/2 支持（包括 h2c 和 TLS + HTTP/2）

---

HTTPS 模块 - 提供安全的 HTTP 客户端功能。

## 主要功能

- **HTTPS 支持**: 安全的 HTTP 请求
- **证书验证**: TLS/SSL 证书验证
- **代理支持**: HTTP/HTTPS 代理配置
- **连接池**: 复用连接提升性能

## 技术栈

- **HTTP 客户端**: hyper, hyper-tls
- **TLS**: rustls
- **异步运行时**: Tokio

## 使用示例

```rust,no_run
use wae_https::{HttpsServerBuilder, HttpsServer};
use axum::Router;

#[tokio::main]
async fn main() {
    let server = HttpsServerBuilder::new()
        .service_name("my-service")
        .router(Router::new())
        .build();
    
    server.serve().await.unwrap();
}
```

## HTTP/2 配置

```rust,no_run
use wae_https::{HttpsServerBuilder, Http2Config};
use axum::Router;

let server = HttpsServerBuilder::new()
    .http2_config(Http2Config::new().with_max_concurrent_streams(128))
    .router(Router::new())
    .build();
```
