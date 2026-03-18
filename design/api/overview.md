# API 概览

WAE 提供统一的 API 设计，所有模块都遵循相同的设计原则。同时提供 Rust 后端和 TypeScript 前端的完整 API。

## 核心 Trait

### 异步优先

所有可能涉及 I/O 的操作都是异步的：

```rust
pub trait AuthService: Send + Sync {
    async fn login(&self, credentials: &Credentials) -> WaeResult<AuthToken>;
    async fn logout(&self, token: &str) -> WaeResult<()>;
    async fn refresh_token(&self, refresh_token: &str) -> WaeResult<AuthToken>;
    async fn validate_token(&self, token: &str) -> WaeResult<TokenValidation>;
}

pub trait StorageProvider: Send + Sync {
    fn get_presigned_put_url(&self, key: &str, config: &StorageConfig) -> WaeResult<Url>;
    fn sign_url(&self, path: &str, config: &StorageConfig) -> WaeResult<Url>;
}
```

### 错误处理

统一使用 `WaeResult<T>` 作为返回类型：

```rust
pub type WaeResult<T> = Result<T, WaeError>;

/// 错误分类
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ErrorCategory {
    /// 验证错误 (400)
    Validation,
    /// 认证错误 (401)
    Auth,
    /// 权限错误 (403)
    Permission,
    /// 资源未找到 (404)
    NotFound,
    /// 请求冲突 (409)
    Conflict,
    /// 请求过多 (429)
    RateLimited,
    /// 网络/服务错误 (502/503)
    Network,
    /// 存储错误 (500)
    Storage,
    /// 数据库错误 (500)
    Database,
    /// 缓存错误 (500)
    Cache,
    /// 配置错误 (500)
    Config,
    /// 超时错误 (408/504)
    Timeout,
    /// 内部错误 (500)
    Internal,
}
```

## 后端模块 (wae-server)

All-in-one 入口包，导出所有后端模块：

```rust
pub use wae_ai as ai;
pub use wae_config as config;
#[cfg(any(feature = "database-turso", feature = "database-postgres", feature = "database-mysql"))]
pub use wae_database as database;
pub use wae_distributed as distributed;
pub use wae_effect as effect;
pub use wae_email as email;
pub use wae_event as event;
pub use wae_https as https;
#[cfg(feature = "observability")]
pub use wae_observability as observability;
pub use wae_resilience as resilience;
pub use wae_scheduler as scheduler;
pub use wae_service as service;
pub use wae_session as session;
pub use wae_storage as storage;
pub use wae_testing as testing;
#[cfg(feature = "tools")]
pub use wae_tools as tools;
pub use wae_types as types;
pub use wae_websocket as websocket;

pub use types::{WaeError, WaeResult};
```

## 前端模块 (@wae/*)

TypeScript/JavaScript 前端库：

| 包名 | 说明 |
|-----|------|
| `@wae/core` | TypeScript 类型定义 |
| `@wae/client` | HTTP 客户端 |
| `@wae/auth` | 认证客户端 |
| `@wae/websocket` | WebSocket 客户端 |
| `@wae/storage` | 存储服务客户端 |

### 使用示例

```rust
// 使用 all-in-one
use wae_server::{ai, storage, email, https, WaeError, WaeResult};

// 或单独使用模块
use wae_ai::{HunyuanProvider, ChatCapability};
use wae_storage::{StorageService, StorageConfig};
use wae_https::{HttpsServerBuilder, ApiResponse};
use wae_resilience::{CircuitBreaker, CircuitBreakerConfig};
use wae_scheduler::{CronScheduler, CronSchedulerConfig};
use wae_authentication::{AuthService, AuthConfig, Credentials};
use wae_websocket::{WebSocketServer, ServerConfig};
```

## 模块概览

### 核心模块

| 模块 | 说明 |
|-----|------|
| `wae-types` | 核心类型定义 (WaeError, WaeResult, ErrorCategory) |
| `wae-config` | 配置管理 (ConfigLoader, load_config) |
| `wae-https` | HTTP/HTTPS 服务 (HttpsServerBuilder, ApiResponse) |
| `wae-database` | 数据库 ORM (条件编译) |
| `wae-crypto` | 加密工具 (hash, hmac, password, totp) |

### 服务模块

| 模块 | 说明 |
|-----|------|
| `wae-service` | 服务发现与注册 (ServiceRegistry, ServiceDiscovery, LoadBalancer) |
| `wae-websocket` | WebSocket 服务 (WebSocketServer, WebSocketClient) |
| `wae-storage` | 对象存储服务 (StorageService, StorageProvider) |
| `wae-email` | 邮件服务 (SmtpEmailProvider, SendmailEmailProvider) |
| `wae-ai` | AI 服务抽象 (ChatCapability, TextToImageCapability) |
| `wae-search` | 搜索服务 (Elasticsearch, OpenSearch) |

### 基础设施模块

| 模块 | 说明 |
|-----|------|
| `wae-resilience` | 弹性容错 (CircuitBreaker, RateLimiter, Retry, Timeout, Bulkhead) |
| `wae-scheduler` | 任务调度器 (CronScheduler, IntervalScheduler, DelayedQueue) |
| `wae-event` | 事件驱动 |
| `wae-queue` | 消息队列 |
| `wae-distributed` | 分布式支持 |
| `wae-observability` | 可观测性 (logging, metrics, tracing, health) |
| `wae-monitoring` | 监控服务 |

### 认证模块

| 模块 | 说明 |
|-----|------|
| `wae-authentication` | 认证服务 (JWT, OAuth2, SAML, TOTP) |
| `wae-session` | Session 管理 (Session, SessionStore, SessionLayer) |

### 开发工具模块

| 模块 | 说明 |
|-----|------|
| `wae-testing` | 测试支持 |
| `wae-tools` | 开发工具 (迁移、自动迁移) |
| `wae-macros` | 过程宏 |
| `wae-schema` | Schema 定义 |
| `wae-request` | HTTP 客户端 |
| `wae-effect` | 副作用管理 |
| `wae-cache` | 缓存服务 |

## 命名约定

| 类型 | 命名 | 示例 |
|-----|------|------|
| Trait | XxxCapability / XxxProvider / XxxService | ChatCapability, StorageProvider, AuthService |
| Struct | XxxConfig / XxxParams | AiConfig, TextToImageParams, StorageConfig |
| Enum | XxxType / XxxError / XxxState | StorageProviderType, WaeErrorKind, CircuitState |
| Result | XxxResult | AiResult, StorageResult, WaeResult |
| 错误 | XxxError | WaeError, WebSocketError, ConfigError |
| Builder | XxxBuilder | HttpsServerBuilder, ResiliencePipelineBuilder |

## HTTP API

### 统一响应结构

```rust
pub struct ApiResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub error: Option<ApiErrorBody>,
    pub trace_id: Option<String>,
}

pub struct ApiErrorBody {
    pub code: String,
    pub message: String,
}
```

### 创建响应

```rust
// 成功响应
let response = ApiResponse::success(user_data);
let response = ApiResponse::success_with_trace(user_data, "trace-123");

// 错误响应
let response = ApiResponse::error("INVALID_PARAMS", "参数无效");
let response = ApiResponse::error_with_trace("NOT_FOUND", "用户不存在", "trace-123");
```

### JSON 响应示例

```json
{
    "success": true,
    "data": {
        "id": "user-001",
        "username": "alice"
    },
    "error": null,
    "trace_id": "trace-123"
}
```

```json
{
    "success": false,
    "data": null,
    "error": {
        "code": "NOT_FOUND",
        "message": "用户不存在"
    },
    "trace_id": "trace-123"
}
```

## 前端 API

### HTTP 客户端

```typescript
import { createHttpClient, type ApiResponse } from "@wae/client";

const client = createHttpClient({
    baseUrl: "http://localhost:3000",
    timeout: 30000,
    maxRetries: 3,
    retryDelay: 1000,
});

// GET 请求
const response: ApiResponse<UserData> = await client.get("/users/1");

// POST 请求
const response: ApiResponse<UserData> = await client.post("/users", {
    username: "alice",
    email: "alice@example.com",
});

// 检查响应
if (response.success && response.data) {
    console.log(response.data);
} else {
    console.error(response.error?.message);
}
```

### 认证客户端

```typescript
import { createAuthClient } from "@wae/auth";

const auth = createAuthClient({
    baseUrl: "http://localhost:3000",
    tokenKey: "access_token",
    refreshTokenKey: "refresh_token",
});

// 登录
const token = await auth.login({
    identifier: "user@example.com",
    password: "password",
});

// 获取状态
const state = auth.getState();
console.log(state.isAuthenticated, state.user);

// 订阅状态变化
const unsubscribe = auth.subscribe((state) => {
    console.log("Auth state changed:", state);
});

// 登出
await auth.logout();
```

### WebSocket 客户端

```typescript
import { createWebSocketClient } from "@wae/websocket";

const ws = createWebSocketClient({
    url: "ws://localhost:8080",
    reconnectInterval: 5000,
    heartbeatInterval: 30000,
    maxReconnectAttempts: 0,
});

// 连接
await ws.connect();

// 发送消息
ws.sendText("Hello");
ws.sendJson({ type: "chat", message: "Hello" });
ws.sendBinary(new Uint8Array([1, 2, 3]));

// 订阅事件
ws.on("message", (data) => {
    console.log("Received:", data.message);
});

ws.on("disconnect", (data) => {
    console.log("Disconnected:", data.reason);
});

// 房间管理
ws.joinRoom("room-1");
ws.leaveRoom("room-1");
```

### 存储客户端

```typescript
import { createStorageClient } from "@wae/storage";

const storage = createStorageClient({
    baseUrl: "http://localhost:3000",
});

// 上传文件
const result = await storage.uploadFile(file, "uploads/test.jpg", (progress) => {
    console.log(`Progress: ${progress.percentage}%`);
});

console.log("Access URL:", result.accessUrl);

// 下载文件
const blob = await storage.downloadFile("uploads/test.jpg");
const text = await storage.downloadAsText("uploads/test.txt");
const json = await storage.downloadAsJson("uploads/data.json");

// 获取签名 URL
const signedUrl = await storage.getSignedUrl("uploads/test.jpg", 3600);
```

## 最佳实践

### 配置管理

```rust
use wae_config::ConfigLoader;
use serde::Deserialize;

#[derive(Deserialize)]
struct AppConfig {
    listen_addr: String,
    ai: AiConfig,
    storage: StorageConfig,
}

impl AppConfig {
    pub async fn load() -> Result<Self, WaeError> {
        ConfigLoader::new()
            .with_toml("config.toml")
            .with_env("APP_")
            .extract()
    }
}
```

### 错误处理

```rust
use wae_server::WaeError;

async fn handle_request() -> Result<(), WaeError> {
    let result = provider.chat(&params, &config).await?;
    Ok(())
}
```

### 弹性容错

```rust
use wae_resilience::{CircuitBreaker, CircuitBreakerConfig, RetryConfig, retry_async};

let circuit_breaker = CircuitBreaker::new(CircuitBreakerConfig::default());

let result = retry_async(
    || async {
        circuit_breaker.execute(|| async {
            some_external_service().await
        }).await
    },
    RetryConfig::default(),
).await?;
```

### 任务调度

```rust
use wae_scheduler::{CronScheduler, CronSchedulerConfig};

let scheduler = CronScheduler::new(CronSchedulerConfig::default());
scheduler.schedule("0 0 * * * *", my_task).await?;
scheduler.start().await?;
```

### WebSocket 服务端

```rust
use wae_websocket::{WebSocketServer, ServerConfig, ClientHandler, Connection, Message};

struct MyHandler;

impl ClientHandler for MyHandler {
    async fn on_connect(&self, conn: &Connection) -> WaeResult<()> {
        println!("Client connected: {}", conn.id);
        Ok(())
    }

    async fn on_message(&self, conn: &Connection, msg: Message) -> WaeResult<()> {
        println!("Received: {:?}", msg);
        Ok(())
    }

    async fn on_disconnect(&self, conn: &Connection) {
        println!("Client disconnected: {}", conn.id);
    }
}

let server = WebSocketServer::new(ServerConfig::default());
server.start(MyHandler).await?;
```

### 可观测性

```rust
use wae_observability::{logging, metrics, tracing};

// 初始化日志
logging::init();

// 初始化指标
metrics::init();

// 初始化追踪
tracing::init();
```

### 数据库操作

```rust
use wae_database::{ConnectionPool, QueryBuilder};

// 执行查询
let users = pool.query("SELECT * FROM users WHERE active = ?", &[true]).await?;

// 使用 ORM
let user = User::find_by_id(pool, 1).await?;
let users = User::find_all(pool).await?;
```