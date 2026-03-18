# wae-https

HTTPS 服务核心模块，提供统一的 HTTP/HTTPS 服务抽象，支持中间件、路由、请求处理等核心功能。

## 概述

基于 axum 框架构建，深度融合 tokio 运行时，支持异步请求处理、中间件链式处理、统一错误处理、请求追踪与日志、TLS/HTTPS 支持、HTTP/2 和 HTTP/1.1 双协议支持、模板渲染等功能。

## 快速开始

### 最小 HTTP 服务

```rust
use axum::{Router, routing::get};
use wae_https::HttpsServerBuilder;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let app = Router::new()
        .route("/", get(|| async { "Hello, World!" }));

    let server = HttpsServerBuilder::new()
        .addr("0.0.0.0:3000".parse()?)
        .service_name("my-service")
        .router(app)
        .build();

    server.serve().await?;
    Ok(())
}
```

### 带 HTTPS 的服务

```rust
use axum::{Router, routing::get};
use wae_https::HttpsServerBuilder;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let app = Router::new()
        .route("/", get(|| async { "Hello, HTTPS!" }));

    let server = HttpsServerBuilder::new()
        .addr("0.0.0.0:443".parse()?)
        .service_name("secure-service")
        .router(app)
        .tls("/etc/ssl/cert.pem", "/etc/ssl/key.pem")
        .build();

    server.serve().await?;
    Ok(())
}
```

## 核心用法

### 路由

基础路由定义：

```rust
use axum::{Router, routing::{get, post, put, delete}, Json};
use wae_https::extract::Path;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
struct User {
    id: u64,
    name: String,
}

async fn list_users() -> Json<Vec<User>> {
    Json(vec![
        User { id: 1, name: "Alice".to_string() },
        User { id: 2, name: "Bob".to_string() },
    ])
}

async fn get_user(Path(id): Path<u64>) -> Json<User> {
    Json(User { id, name: "Alice".to_string() })
}

async fn create_user(Json(user): Json<User>) -> Json<User> {
    Json(user)
}

let app = Router::new()
    .route("/users", get(list_users).post(create_user))
    .route("/users/:id", get(get_user).put(update_user).delete(delete_user));
```

路由合并与嵌套：

```rust
use wae_https::router::RouterBuilder;

let api_router = Router::new()
    .route("/users", get(list_users))
    .route("/posts", get(list_posts));

let admin_router = Router::new()
    .route("/settings", get(get_settings));

let app = Router::new()
    .nest("/api", api_router)
    .nest("/admin", admin_router);
```

健康检查路由：

```rust
use wae_https::router::health_check_router;

let app = Router::new()
    .merge(health_check_router());
```

### 统一响应格式

使用 `ApiResponse` 返回统一格式的 JSON 响应：

```rust
use wae_https::ApiResponse;
use serde::Serialize;

#[derive(Serialize)]
struct UserData {
    id: u64,
    name: String,
}

async fn get_user() -> ApiResponse<UserData> {
    ApiResponse::success(UserData {
        id: 1,
        name: "Alice".to_string(),
    })
}

async fn create_user() -> ApiResponse<UserData> {
    ApiResponse::success_with_trace(
        UserData { id: 1, name: "Alice".to_string() },
        "trace-123",
    )
}

async fn error_example() -> ApiResponse<UserData> {
    ApiResponse::error("USER_NOT_FOUND", "用户不存在")
}

async fn error_with_trace() -> ApiResponse<UserData> {
    ApiResponse::error_with_trace("INVALID_PARAMS", "参数错误", "trace-123")
}
```

成功响应示例：

```json
{
    "success": true,
    "data": { "id": 1, "name": "Alice" },
    "error": null,
    "trace_id": null
}
```

错误响应示例：

```json
{
    "success": false,
    "data": null,
    "error": {
        "code": "USER_NOT_FOUND",
        "message": "用户不存在"
    },
    "trace_id": "trace-123"
}
```

### 错误处理

使用 `HttpError` 返回标准 HTTP 错误：

```rust
use wae_https::error::HttpError;

async fn get_user(id: u64) -> Result<Json<User>, HttpError> {
    let user = find_user(id).ok_or_else(|| HttpError::not_found("用户不存在"))?;
    Ok(Json(user))
}

async fn update_user(id: u64, data: UpdateUser) -> Result<Json<User>, HttpError> {
    if !is_authorized() {
        return Err(HttpError::forbidden("无权修改该用户"));
    }
    if is_invalid(&data) {
        return Err(HttpError::bad_request("参数格式错误"));
    }
    Ok(Json(update(id, data)?))
}
```

常用错误构造方法：

```rust
HttpError::bad_request("参数格式错误")
HttpError::unauthorized("未登录")
HttpError::forbidden("无权访问")
HttpError::not_found("资源不存在")
HttpError::conflict("资源冲突")
HttpError::payload_too_large("请求体过大")
HttpError::internal("服务器内部错误")
HttpError::service_unavailable("服务暂不可用")
HttpError::gateway_timeout("网关超时")
```

使用 `ErrorExt` trait 简化错误转换：

```rust
use wae_https::error::ErrorExt;

async fn handler() -> Result<Json<Data>, HttpError> {
    let data = database_query()
        .await
        .not_found()?;  // 自动转换为 404 错误
    
    let result = external_api_call()
        .await
        .internal_error()?;  // 自动转换为 500 错误
    
    Ok(Json(data))
}
```

### 请求提取

从请求中提取参数：

```rust
use axum::Json;
use wae_https::extract::{Path, Query, State, Header};
use serde::Deserialize;

#[derive(Deserialize)]
struct UserQuery {
    name: Option<String>,
    page: Option<u32>,
}

#[derive(Deserialize)]
struct CreateUser {
    name: String,
    email: String,
}

async fn search_users(
    Query(query): Query<UserQuery>,
) -> Json<Vec<User>> {
    Json(vec![])
}

async fn get_user(Path(id): Path<u64>) -> Json<User> {
    Json(User { id, name: "Alice".to_string() })
}

async fn create_user(Json(body): Json<CreateUser>) -> Json<User> {
    Json(User { id: 1, name: body.name })
}

async fn handler_with_header(
    Header(auth): Header<String>,
) -> String {
    format!("Authorization: {}", auth)
}

async fn handler_with_state(
    State(db): State<Database>,
) -> Json<Vec<User>> {
    Json(db.get_users())
}
```

## 中间件

### CORS 跨域

开发环境使用宽松模式：

```rust
use wae_https::middleware::cors_permissive;
use axum::Router;

let app = Router::new()
    .route("/api", get(handler))
    .layer(cors_permissive());
```

生产环境使用严格模式：

```rust
use wae_https::middleware::cors_strict;

let app = Router::new()
    .route("/api", get(handler))
    .layer(cors_strict());
```

自定义 CORS 配置：

```rust
use wae_https::middleware::CorsConfig;

let layer = CorsConfig::new()
    .allow_origins(["https://example.com", "https://api.example.com"])
    .allow_methods(["GET", "POST", "PUT", "DELETE"])
    .allow_headers(["Content-Type", "Authorization"])
    .allow_credentials(true)
    .max_age(3600)
    .into_layer();

let app = Router::new()
    .route("/api", get(handler))
    .layer(layer);
```

### 压缩

启用响应压缩：

```rust
use wae_https::middleware::Compression;

let compression = Compression::new();
let app = Router::new()
    .route("/api", get(handler))
    .layer(compression.build());
```

自定义压缩算法：

```rust
use wae_https::middleware::CompressionBuilder;

let layer = CompressionBuilder::new()
    .gzip()
    .brotli()
    .no_deflate()
    .build()
    .build();

let app = Router::new()
    .route("/api", get(handler))
    .layer(layer);
```

### 请求追踪

添加请求 ID：

```rust
use wae_https::middleware::RequestIdLayer;
use axum::{Router, middleware};

let app = Router::new()
    .route("/api", get(handler))
    .layer(middleware::from_fn(RequestIdLayer::middleware));
```

请求 ID 会从 `x-request-id` 请求头读取，如果不存在则生成新的 UUID。

添加追踪日志：

```rust
use wae_https::middleware::TracingLayer;
use axum::{Router, middleware};

let app = Router::new()
    .route("/api", get(handler))
    .layer(middleware::from_fn(TracingLayer::middleware));
```

### 组合多个中间件

```rust
use axum::{Router, middleware};
use wae_https::middleware::{cors_permissive, Compression, RequestIdLayer, TracingLayer};

let app = Router::new()
    .route("/api/users", get(list_users))
    .layer(cors_permissive())
    .layer(Compression::new().build())
    .layer(middleware::from_fn(TracingLayer::middleware))
    .layer(middleware::from_fn(RequestIdLayer::middleware));
```

## HTTPS/TLS

### 基础 TLS 配置

```rust
use wae_https::HttpsServerBuilder;

let server = HttpsServerBuilder::new()
    .addr("0.0.0.0:443".parse()?)
    .router(app)
    .tls("/etc/ssl/cert.pem", "/etc/ssl/key.pem")
    .build();
```

### 支持 HTTP/2 的 TLS

```rust
use wae_https::tls::create_tls_acceptor_with_http2;

let acceptor = create_tls_acceptor_with_http2(
    "/etc/ssl/cert.pem",
    "/etc/ssl/key.pem",
    true,
)?;
```

### 客户端证书验证

```rust
use wae_https::tls::create_tls_acceptor_with_client_auth;

let acceptor = create_tls_acceptor_with_client_auth(
    "/etc/ssl/cert.pem",
    "/etc/ssl/key.pem",
    "/etc/ssl/ca.pem",
    true,
)?;
```

### 使用 TlsConfigBuilder

```rust
use wae_https::tls::TlsConfigBuilder;

let acceptor = TlsConfigBuilder::new()
    .cert_path("/etc/ssl/cert.pem")
    .key_path("/etc/ssl/key.pem")
    .ca_path("/etc/ssl/ca.pem")
    .enable_http2(true)
    .build()?;
```

### HTTP/2 配置

```rust
use wae_https::{HttpsServerBuilder, HttpVersion, Http2Config};
use std::time::Duration;

let http2_config = Http2Config::new()
    .with_max_concurrent_streams(512)
    .with_stream_idle_timeout(Duration::from_secs(120))
    .with_enable_push(false);

let server = HttpsServerBuilder::new()
    .addr("0.0.0.0:443".parse()?)
    .router(app)
    .http_version(HttpVersion::Both)
    .http2_config(http2_config)
    .tls("/etc/ssl/cert.pem", "/etc/ssl/key.pem")
    .build();
```

## 模板渲染

基于 Askama 模板引擎：

```rust
use wae_https::template::{TemplateConfig, TemplateRenderer, HtmlTemplate};
use askama::Template;

#[derive(Template)]
#[template(path = "user.html")]
struct UserTemplate<'a> {
    name: &'a str,
}

async fn render_user() -> HtmlTemplate<UserTemplate<'static>> {
    HtmlTemplate::new(UserTemplate { name: "Alice" })
}
```

配置模板目录：

```rust
use wae_https::template::TemplateConfig;

let config = TemplateConfig::new()
    .with_template_dir("views")
    .with_extension("html")
    .with_cache(true);
```

## 最佳实践

### 推荐的服务结构

```rust
use axum::{Router, routing::get, Json};
use wae_https::{HttpsServerBuilder, ApiResponse, middleware::{cors_permissive, Compression}};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
struct User {
    id: u64,
    name: String,
}

async fn list_users() -> ApiResponse<Vec<User>> {
    ApiResponse::success(vec![
        User { id: 1, name: "Alice".to_string() },
    ])
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let app = Router::new()
        .route("/api/users", get(list_users))
        .layer(cors_permissive())
        .layer(Compression::new().build());

    let server = HttpsServerBuilder::new()
        .addr("0.0.0.0:3000".parse()?)
        .service_name("user-service")
        .router(app)
        .build();

    server.serve().await?;
    Ok(())
}
```

### 分层架构

```rust
mod handlers {
    use axum::Json;
    use wae_https::{ApiResponse, error::HttpError};
    use crate::services::UserService;

    pub async fn list_users(
        State(service): State<UserService>,
    ) -> Result<ApiResponse<Vec<User>>, HttpError> {
        let users = service.get_all().await?;
        Ok(ApiResponse::success(users))
    }
}

mod services {
    use wae_https::error::HttpError;

    pub struct UserService {
        db: Database,
    }

    impl UserService {
        pub async fn get_all(&self) -> Result<Vec<User>, HttpError> {
            self.db.query("SELECT * FROM users").await
                .internal_error()
        }
    }
}
```

### 错误处理模式

```rust
use wae_https::error::{HttpError, ErrorExt};

async fn handler() -> Result<ApiResponse<Data>, HttpError> {
    let config = load_config()
        .await
        .internal_error()?;

    let user = database::find_user(id)
        .await
        .not_found()?;

    if !user.is_active {
        return Err(HttpError::forbidden("用户已被禁用"));
    }

    let result = process(user)
        .await
        .map_http_error(|e| HttpError::bad_request(&e))?;

    Ok(ApiResponse::success(result))
}
```

## 常见问题

### 如何同时监听 HTTP 和 HTTPS？

```rust
use wae_https::HttpsServerBuilder;
use tokio::try_join;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let http_server = HttpsServerBuilder::new()
        .addr("0.0.0.0:80".parse()?)
        .router(app.clone())
        .build();

    let https_server = HttpsServerBuilder::new()
        .addr("0.0.0.0:443".parse()?)
        .router(app)
        .tls("/etc/ssl/cert.pem", "/etc/ssl/key.pem")
        .build();

    try_join!(
        http_server.serve(),
        https_server.serve(),
    )?;

    Ok(())
}
```

### 如何添加全局状态？

```rust
use axum::extract::State;

#[derive(Clone)]
struct AppState {
    db: Database,
    config: Config,
}

async fn handler(State(state): State<AppState>) -> Json<Data> {
    Json(state.db.query())
}

let state = AppState { db, config };

let app = Router::new()
    .route("/api", get(handler))
    .with_state(state);
```

### 如何处理大文件上传？

```rust
use axum::body::Body;
use wae_https::error::HttpError;

async fn upload(body: Body) -> Result<ApiResponse<String>, HttpError> {
    let bytes = axum::body::to_bytes(body, 10 * 1024 * 1024)
        .await
        .map_err(|_| HttpError::payload_too_large("文件大小超过 10MB"))?;
    
    save_file(bytes).await?;
    Ok(ApiResponse::success("上传成功".to_string()))
}
```

### 如何自定义错误响应？

```rust
use wae_https::error::{HttpError, ErrorResponse};

async fn handler() -> Result<Json<Data>, HttpError> {
    let result = operation().await.map_err(|e| {
        HttpError::bad_request("操作失败").with_details(serde_json::json!({
            "reason": e.to_string()
        }))
    })?;
    
    Ok(Json(result))
}
```

### 如何实现请求超时？

```rust
use axum::middleware;
use tokio::time::{timeout, Duration};

async fn timeout_middleware(
    req: axum::extract::Request,
    next: axum::middleware::Next,
) -> Result<axum::response::Response, HttpError> {
    match timeout(Duration::from_secs(30), next.run(req)).await {
        Ok(response) => Ok(response),
        Err(_) => Err(HttpError::request_timeout("请求超时")),
    }
}

let app = Router::new()
    .route("/api", get(handler))
    .layer(middleware::from_fn(timeout_middleware));
```
