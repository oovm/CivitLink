# wae-session

Session 管理模块，提供完整的 HTTP Session 管理功能。

## 概述

wae-session 深度融合 tokio 运行时，所有 API 都是异步优先设计。支持多种存储后端（内存、Redis 等），提供灵活的配置选项，并与 axum 框架无缝集成。通过 `SessionLayer` 中间件和 `SessionExtractor` 提取器，可以轻松在请求处理函数中访问和操作 Session 数据。

## 快速开始

最小可运行的 Session 示例：

```rust
use wae_session::{SessionLayer, MemorySessionStore, SessionConfig, SessionExtractor};
use axum::{Router, routing::get};

#[tokio::main]
async fn main() {
    let store = MemorySessionStore::new();
    let layer = SessionLayer::new(store, SessionConfig::default());

    let app = Router::new()
        .route("/", get(handler))
        .layer(layer);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn handler(session: SessionExtractor) -> String {
    let count: u32 = session.get_typed("count").await.unwrap_or(0);
    session.set("count", count + 1).await;
    format!("访问次数: {}", count + 1)
}
```

## 核心用法

### 配置 Session

`SessionConfig` 提供链式配置方法：

```rust
use wae_session::{SessionConfig, SameSite};
use std::time::Duration;

let config = SessionConfig::new()
    .with_cookie_name("my_session")
    .with_cookie_domain("example.com")
    .with_cookie_path("/")
    .with_secure(true)
    .with_http_only(true)
    .with_same_site(SameSite::Strict)
    .with_ttl(Duration::from_secs(7200))
    .with_id_length(64)
    .signed("your-secret-key");
```

常用配置说明：

| 配置项 | 说明 | 默认值 |
|--------|------|--------|
| `cookie_name` | Cookie 名称 | `"session"` |
| `ttl` | Session 有效期 | 3600 秒 |
| `cookie_secure` | 仅 HTTPS 传输 | `false` |
| `cookie_http_only` | 防止 JS 访问 | `true` |
| `cookie_same_site` | CSRF 防护策略 | `Lax` |

### 使用 SessionExtractor

在 axum 处理函数中通过 `SessionExtractor` 访问 Session：

```rust
use wae_session::SessionExtractor;
use axum::response::IntoResponse;

async fn handler(session: SessionExtractor) -> impl IntoResponse {
    let user_id: Option<String> = session.get_typed("user_id").await;
    format!("User ID: {:?}", user_id)
}
```

### 存储和读取数据

Session 支持存储任意可序列化类型：

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
struct UserProfile {
    id: String,
    name: String,
    email: String,
}

async fn save_profile(session: SessionExtractor) -> String {
    let profile = UserProfile {
        id: "123".to_string(),
        name: "Alice".to_string(),
        email: "alice@example.com".to_string(),
    };
    session.set("profile", profile).await;
    "Profile saved".to_string()
}

async fn get_profile(session: SessionExtractor) -> String {
    match session.get_typed::<UserProfile>("profile").await {
        Some(profile) => format!("Profile: {} ({})", profile.name, profile.email),
        None => "No profile found".to_string(),
    }
}

async fn update_email(session: SessionExtractor) -> String {
    if let Some(mut profile) = session.get_typed::<UserProfile>("profile").await {
        profile.email = "new@example.com".to_string();
        session.set("profile", profile).await;
        "Email updated".to_string()
    } else {
        "No profile found".to_string()
    }
}

async fn delete_profile(session: SessionExtractor) -> String {
    session.remove("profile").await;
    "Profile deleted".to_string()
}
```

## 登录态管理

完整的登录登出示例：

```rust
use wae_session::{SessionLayer, MemorySessionStore, SessionConfig, SessionExtractor};
use axum::{Router, routing::{get, post}, extract::Form, response::{IntoResponse, Redirect}};
use serde::Deserialize;

#[derive(Deserialize)]
struct LoginForm {
    username: String,
    password: String,
}

#[tokio::main]
async fn main() {
    let store = MemorySessionStore::new();
    let config = SessionConfig::default()
        .with_ttl(std::time::Duration::from_secs(86400))
        .with_secure(true);

    let app = Router::new()
        .route("/", get(index))
        .route("/login", get(login_page).post(login))
        .route("/logout", post(logout))
        .route("/dashboard", get(dashboard))
        .layer(SessionLayer::new(store, config));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn index(session: SessionExtractor) -> impl IntoResponse {
    if session.get_typed::<String>("user_id").await.is_some() {
        Redirect::to("/dashboard")
    } else {
        Redirect::to("/login")
    }
}

async fn login_page(session: SessionExtractor) -> impl IntoResponse {
    if session.get_typed::<String>("user_id").await.is_some() {
        Redirect::to("/dashboard")
    } else {
        "Login form..."
    }
}

async fn login(session: SessionExtractor, Form(form): Form<LoginForm>) -> impl IntoResponse {
    if validate_user(&form.username, &form.password) {
        session.set("user_id", &form.username).await;
        session.set("login_time", chrono::Utc::now().to_rfc3339()).await;
        Redirect::to("/dashboard")
    } else {
        "Invalid credentials"
    }
}

async fn logout(session: SessionExtractor) -> impl IntoResponse {
    session.clear().await;
    Redirect::to("/login")
}

async fn dashboard(session: SessionExtractor) -> impl IntoResponse {
    match session.get_typed::<String>("user_id").await {
        Some(user_id) => {
            let login_time = session.get_typed::<String>("login_time").await
                .unwrap_or_else(|| "unknown".to_string());
            format!("Welcome, {}! Login time: {}", user_id, login_time)
        }
        None => Redirect::to("/login"),
    }
}

fn validate_user(username: &str, password: &str) -> bool {
    username == "admin" && password == "secret"
}
```

### 登录态检查中间件

创建可复用的登录态检查：

```rust
use axum::{
    extract::Request,
    middleware::{self, Next},
    response::{IntoResponse, Redirect},
};

async fn require_auth(
    session: SessionExtractor,
    request: Request,
    next: Next,
) -> impl IntoResponse {
    if session.get_typed::<String>("user_id").await.is_some() {
        next.run(request).await
    } else {
        Redirect::to("/login")
    }
}

let protected_routes = Router::new()
    .route("/dashboard", get(dashboard))
    .route("/settings", get(settings))
    .route_layer(middleware::from_fn(require_auth));
```

## 自定义存储

实现 `SessionStore` trait 创建自定义存储后端：

```rust
use wae_session::SessionStore;
use std::time::Duration;
use async_trait::async_trait;

#[derive(Clone)]
struct RedisSessionStore {
    client: redis::Client,
}

#[async_trait]
impl SessionStore for RedisSessionStore {
    async fn get(&self, session_id: &str) -> Option<String> {
        let mut conn = self.client.get_async_connection().await.ok()?;
        redis::cmd("GET")
            .arg(format!("session:{}", session_id))
            .query_async(&mut conn)
            .await
            .ok()
    }

    async fn set(&self, session_id: &str, data: &str, ttl: Duration) {
        if let Ok(mut conn) = self.client.get_async_connection().await {
            let _ = redis::cmd("SETEX")
                .arg(format!("session:{}", session_id))
                .arg(ttl.as_secs())
                .arg(data)
                .query_async::<_, ()>(&mut conn)
                .await;
        }
    }

    async fn remove(&self, session_id: &str) {
        if let Ok(mut conn) = self.client.get_async_connection().await {
            let _ = redis::cmd("DEL")
                .arg(format!("session:{}", session_id))
                .query_async::<_, ()>(&mut conn)
                .await;
        }
    }

    async fn exists(&self, session_id: &str) -> bool {
        if let Ok(mut conn) = self.client.get_async_connection().await {
            redis::cmd("EXISTS")
                .arg(format!("session:{}", session_id))
                .query_async::<_, i32>(&mut conn)
                .await
                .unwrap_or(0) > 0
        } else {
            false
        }
    }

    async fn refresh(&self, session_id: &str, ttl: Duration) {
        if let Ok(mut conn) = self.client.get_async_connection().await {
            let _ = redis::cmd("EXPIRE")
                .arg(format!("session:{}", session_id))
                .arg(ttl.as_secs())
                .query_async::<_, ()>(&mut conn)
                .await;
        }
    }
}
```

使用自定义存储：

```rust
let redis_store = RedisSessionStore {
    client: redis::Client::open("redis://127.0.0.1/").unwrap(),
};
let layer = SessionLayer::new(redis_store, SessionConfig::default());
```

## 最佳实践

### 1. 生产环境配置

```rust
let config = SessionConfig::new()
    .with_cookie_name("__session")
    .with_secure(true)
    .with_http_only(true)
    .with_same_site(SameSite::Strict)
    .with_ttl(Duration::from_secs(3600))
    .signed(&env_secret_key);
```

### 2. 敏感数据不要存 Session

Session 数据存储在客户端可访问的存储中，不要存储密码、密钥等敏感信息：

```rust
// 错误：不要存储密码
session.set("password", password).await;

// 正确：只存储必要的标识
session.set("user_id", user_id).await;
```

### 3. 合理设置 TTL

根据业务场景设置合理的过期时间：

```rust
// 短期会话：30 分钟
let short_session = SessionConfig::default()
    .with_ttl(Duration::from_secs(1800));

// 长期会话：7 天（记住登录）
let long_session = SessionConfig::default()
    .with_ttl(Duration::from_secs(86400 * 7));
```

### 4. 使用类型化存取

优先使用 `get_typed` 和 `set` 进行类型安全的数据操作：

```rust
// 推荐：类型安全
let user_id: Option<String> = session.get_typed("user_id").await;

// 不推荐：手动 JSON 解析
let raw = session.get("user_id").await;
let user_id: Option<String> = raw.and_then(|v| serde_json::from_value(v).ok());
```

### 5. 及时清理无用数据

```rust
async fn logout(session: SessionExtractor) {
    session.clear().await;
}

async fn remove_temp_data(session: SessionExtractor) {
    session.remove("temp_verification_code").await;
}
```

## 常见问题

### Q: Session 数据丢失？

检查以下几点：
1. 存储后端是否正常运行（Redis 连接等）
2. TTL 是否设置过短
3. Cookie 的 `secure` 属性在非 HTTPS 环境下会导致 Cookie 无法发送

```rust
// 开发环境关闭 secure
let config = SessionConfig::default()
    .with_secure(false);
```

### Q: 如何实现"记住我"功能？

使用较长的 TTL 并在用户活动时刷新：

```rust
async fn refresh_session(session: SessionExtractor, store: impl SessionStore) {
    if let Some(user_id) = session.get_typed::<String>("user_id").await {
        store.refresh(session.id(), Duration::from_secs(86400 * 30)).await;
    }
}
```

### Q: 如何处理并发修改？

Session 内部使用 `RwLock` 保证线程安全，但同一请求内的多次修改会覆盖：

```rust
async fn handler(session: SessionExtractor) {
    session.set("key", "value1").await;
    session.set("key", "value2").await;
    assert_eq!(session.get_typed::<String>("key").await, Some("value2".to_string()));
}
```

### Q: 内存存储重启后数据丢失？

`MemorySessionStore` 仅适用于开发或单实例部署。生产环境推荐使用 Redis 等持久化存储。

### Q: 如何查看当前 Session 的所有数据？

```rust
async fn debug_session(session: SessionExtractor) -> String {
    let keys = session.keys().await;
    let mut result = String::new();
    for key in keys {
        let value = session.get(&key).await;
        result.push_str(&format!("{}: {:?}\n", key, value));
    }
    result
}
```
