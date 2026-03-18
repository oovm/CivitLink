# wae-http-client

## 概述

HTTP 客户端模块提供前后端一致的请求处理能力：

- **后端 (wae-request)**: 基于 tokio + tokio-rustls 的纯 Rust HTTP/1.1 客户端，支持自动重试、超时控制、HTTPS
- **前端 (wae-client)**: 基于 Fetch API 构建的 HTTP 客户端，支持拦截器、重试机制、请求取消

---

## 快速开始

### 前端请求

```typescript
import { createHttpClient } from "@wae/client";

const client = createHttpClient({ baseUrl: "https://api.example.com" });

const response = await client.get<UserInfo>("/user/123");
if (response.success) {
    console.log("用户:", response.data);
}
```

### 后端请求

```rust
use wae_request::HttpClient;

let client = HttpClient::with_defaults();
let response = client.get("https://api.example.com/user/123").await?;

if response.is_success() {
    let user: UserInfo = response.json()?;
    println!("用户: {:?}", user);
}
```

---

## 前端用法

### 创建客户端

```typescript
import { createHttpClient } from "@wae/client";

const client = createHttpClient({
    baseUrl: "https://api.example.com/v1",
    timeout: 30000,
    maxRetries: 3,
    retryDelay: 1000,
    defaultHeaders: {
        "Content-Type": "application/json",
        "X-API-Version": "1.0",
    },
});
```

### 基础请求

```typescript
// GET 请求
const users = await client.get<User[]>("/users");

// 带查询参数
const users = await client.get<User[]>("/users", {
    page: "1",
    limit: "10",
});

// POST 请求
const created = await client.post<User>("/users", {
    name: "张三",
    email: "zhangsan@example.com",
});

// PUT 请求
const updated = await client.put<User>("/users/123", { name: "李四" });

// PATCH 请求
const patched = await client.patch<User>("/users/123", { name: "李四" });

// DELETE 请求
await client.delete<void>("/users/123");
```

### 请求拦截器

请求拦截器用于在请求发送前修改请求配置，常用于添加认证令牌：

```typescript
client.addRequestInterceptor((url, options) => {
    const token = localStorage.getItem("token");
    return {
        url,
        options: {
            ...options,
            headers: {
                ...options.headers,
                Authorization: `Bearer ${token}`,
            },
        },
    };
});
```

添加请求日志：

```typescript
client.addRequestInterceptor((url, options) => {
    console.log(`[请求] ${options.method} ${url}`);
    return { url, options };
});
```

### 响应拦截器

响应拦截器用于统一处理响应数据：

```typescript
client.addResponseInterceptor((response, request) => {
    console.log(`[响应] ${request.url} - ${response.success ? "成功" : "失败"}`);
    
    if (!response.success && response.error) {
        console.error(`错误码: ${response.error.code}, 消息: ${response.error.message}`);
    }
    
    return response;
});
```

数据转换示例：

```typescript
client.addResponseInterceptor((response) => {
    if (response.success && response.data) {
        response.data.createdAt = new Date(response.data.createdAt);
    }
    return response;
});
```

### 错误拦截器

错误拦截器用于统一处理请求错误：

```typescript
client.addErrorInterceptor((error, request) => {
    console.error(`[错误] ${request.url}:`, error.message);
    
    switch (error.type) {
        case "Authentication":
            localStorage.removeItem("token");
            window.location.href = "/login";
            break;
        case "RateLimit":
            console.warn("请求频率过高，请稍后重试");
            break;
        case "Network":
            console.error("网络连接失败，请检查网络");
            break;
    }
});
```

### 取消请求

```typescript
const client = createHttpClient();

const requestPromise = client.get("/large-data");

setTimeout(() => {
    client.abort();
    console.log("请求已取消");
}, 5000);
```

### 动态更新配置

```typescript
const client = createHttpClient({ baseUrl: "https://api.example.com" });

// 切换环境
client.updateConfig({
    baseUrl: "https://staging-api.example.com",
});

// 查看当前配置
const config = client.getConfig();
console.log(config.baseUrl);
```

### 清除拦截器

```typescript
client.clearRequestInterceptors();
client.clearResponseInterceptors();
client.clearErrorInterceptors();
```

---

## 后端用法

### 创建客户端

```rust
use wae_request::{HttpClient, HttpClientConfig};
use std::time::Duration;
use std::collections::HashMap;

let config = HttpClientConfig {
    timeout: Duration::from_secs(30),
    connect_timeout: Duration::from_secs(10),
    user_agent: "my-app/1.0".to_string(),
    max_retries: 3,
    retry_delay: Duration::from_millis(1000),
    default_headers: HashMap::from([
        ("X-API-Version".to_string(), "1.0".to_string()),
    ]),
};

let client = HttpClient::new(config);

// 或使用默认配置
let client = HttpClient::with_defaults();
```

### 基础请求

```rust
// GET 请求
let response = client.get("https://api.example.com/users").await?;

// 带自定义请求头
let mut headers = HashMap::new();
headers.insert("X-Custom-Header".to_string(), "value".to_string());
let response = client.get_with_headers("https://api.example.com/users", headers).await?;

// POST JSON
#[derive(Serialize)]
struct CreateUser {
    name: String,
    email: String,
}

let user = CreateUser {
    name: "张三".to_string(),
    email: "zhangsan@example.com".to_string(),
};
let response = client.post_json("https://api.example.com/users", &user).await?;

// 通用请求方法
let response = client.request(
    "PUT",
    "https://api.example.com/users/123",
    Some(body_bytes),
    headers,
).await?;
```

### 使用请求构建器

请求构建器提供链式 API 构建复杂请求：

```rust
use wae_request::{HttpClient, RequestBuilder};

let client = HttpClient::with_defaults();

let response = RequestBuilder::new(client, "POST", "https://api.example.com/data")
    .bearer_auth("your-token")
    .header("X-Custom-Header", "value")
    .json(&serde_json::json!({"key": "value"}))
    .send()
    .await?;

println!("Status: {}", response.status);
```

直接解析 JSON 响应：

```rust
#[derive(Deserialize)]
struct UserData {
    id: u64,
    name: String,
}

let data: UserData = RequestBuilder::new(client, "GET", "https://api.example.com/user/123")
    .bearer_auth("your-token")
    .send_json()
    .await?;
```

### 便捷函数

```rust
use wae_request::{get, post};

// 快速 GET 请求
let response = get("https://api.example.com/data")
    .bearer_auth("token")
    .header("X-Request-Id", "123")
    .send()
    .await?;

// 快速 POST 请求并解析 JSON
let data: MyData = post("https://api.example.com/create")
    .json(&serde_json::json!({"name": "test"}))
    .send_json()
    .await?;
```

### 响应处理

```rust
let response = client.get("https://api.example.com/data").await?;

// 检查状态码
if response.is_success() {
    // 解析 JSON
    let data: MyData = response.json()?;
    println!("{:?}", data);
}

// 获取文本内容
let text = response.text();

// 访问响应头
if let Some(content_type) = response.headers.get("content-type") {
    println!("Content-Type: {}", content_type);
}
```

### 错误处理

```rust
use wae_request::{HttpClient, HttpError};

let client = HttpClient::with_defaults();

match client.get("https://api.example.com/data").await {
    Ok(response) => {
        println!("Status: {}", response.status);
    }
    Err(HttpError::Timeout) => {
        eprintln!("请求超时");
    }
    Err(HttpError::ConnectionFailed(msg)) => {
        eprintln!("连接失败: {}", msg);
    }
    Err(HttpError::TlsError(msg)) => {
        eprintln!("TLS 错误: {}", msg);
    }
    Err(HttpError::StatusError { status, body }) => {
        eprintln!("HTTP 错误 {}: {}", status, body);
    }
    Err(e) => {
        eprintln!("请求失败: {}", e);
    }
}
```

### HTTPS 请求

后端自动支持 HTTPS，使用系统根证书验证：

```rust
let response = client.get("https://api.example.com/secure-data").await?;
```

---

## 前后端协作

### 完整请求流程

```
前端 HttpClient                    后端 API Server              外部服务
       │                                  │                         │
       │  1. 发起请求                       │                         │
       │  (拦截器处理)                      │                         │
       │ ──────────────────────────────▶   │                         │
       │                                  │  2. 验证认证              │
       │                                  │  3. 处理业务逻辑          │
       │                                  │                         │
       │                                  │  4. 调用外部服务          │
       │                                  │ ───────────────────────▶ │
       │                                  │                         │
       │                                  │  5. 返回结果              │
       │                                  │ ◀─────────────────────── │
       │                                  │                         │
       │  6. 返回 ApiResponse              │                         │
       │ ◀──────────────────────────────  │                         │
       │                                  │                         │
       │  7. 响应拦截器处理                 │                         │
       │  8. 错误拦截器处理 (如有错误)       │                         │
```

### 实际协作示例

前端发起登录请求：

```typescript
const client = createHttpClient({ baseUrl: "/api" });

client.addRequestInterceptor((url, options) => {
    const token = localStorage.getItem("token");
    if (token) {
        options.headers = {
            ...options.headers,
            Authorization: `Bearer ${token}`,
        };
    }
    return { url, options };
});

client.addErrorInterceptor((error) => {
    if (error.type === "Authentication") {
        localStorage.removeItem("token");
        window.location.href = "/login";
    }
});

const response = await client.post<LoginResult>("/auth/login", {
    username: "user",
    password: "pass",
});

if (response.success) {
    localStorage.setItem("token", response.data.token);
}
```

后端处理登录并调用外部服务：

```rust
use wae_request::HttpClient;

async fn handle_login(credentials: LoginRequest) -> Result<LoginResponse, Error> {
    let client = HttpClient::with_defaults();
    
    // 调用外部认证服务
    let auth_response = client
        .post_json("https://auth-provider.com/verify", &credentials)
        .await?;
    
    if !auth_response.is_success() {
        return Err(Error::Unauthorized);
    }
    
    let auth_data: AuthData = auth_response.json()?;
    
    // 创建本地会话
    let token = create_session(&auth_data.user_id);
    
    Ok(LoginResponse { token })
}
```

---

## 最佳实践

### 前端实践

**1. 单例客户端**

```typescript
// api/client.ts
import { createHttpClient, HttpClient } from "@wae/client";

let client: HttpClient | null = null;

export function getApiClient(): HttpClient {
    if (!client) {
        client = createHttpClient({
            baseUrl: import.meta.env.VITE_API_URL,
            timeout: 30000,
        });
        
        setupInterceptors(client);
    }
    return client;
}

function setupInterceptors(client: HttpClient) {
    client.addRequestInterceptor(authInterceptor);
    client.addResponseInterceptor(loggingInterceptor);
    client.addErrorInterceptor(errorHandler);
}
```

**2. 类型安全的 API 封装**

```typescript
// api/users.ts
import { getApiClient } from "./client";

interface User {
    id: string;
    name: string;
    email: string;
}

interface CreateUserRequest {
    name: string;
    email: string;
}

export const usersApi = {
    list: () => getApiClient().get<User[]>("/users"),
    
    get: (id: string) => getApiClient().get<User>(`/users/${id}`),
    
    create: (data: CreateUserRequest) => 
        getApiClient().post<User>("/users", data),
    
    update: (id: string, data: Partial<User>) => 
        getApiClient().patch<User>(`/users/${id}`, data),
    
    delete: (id: string) => 
        getApiClient().delete<void>(`/users/${id}`),
};
```

**3. 请求取消与清理**

```typescript
import { useEffect, useRef } from "react";

function useAbortableRequest() {
    const clientRef = useRef<HttpClient>();
    
    useEffect(() => {
        return () => {
            clientRef.current?.abort();
        };
    }, []);
    
    return clientRef.current;
}
```

### 后端实践

**1. 共享客户端实例**

```rust
use std::sync::Arc;
use wae_request::HttpClient;

#[derive(Clone)]
pub struct AppState {
    pub http_client: Arc<HttpClient>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            http_client: Arc::new(HttpClient::with_defaults()),
        }
    }
}
```

**2. 配置管理**

```rust
use wae_request::HttpClientConfig;
use std::time::Duration;

pub fn create_http_client() -> HttpClient {
    let config = HttpClientConfig {
        timeout: Duration::from_secs(30),
        connect_timeout: Duration::from_secs(10),
        max_retries: 3,
        retry_delay: Duration::from_millis(500),
        ..Default::default()
    };
    
    HttpClient::new(config)
}
```

**3. 错误转换**

```rust
impl From<HttpError> for ApiError {
    fn from(err: HttpError) -> Self {
        match err {
            HttpError::Timeout => ApiError::Timeout,
            HttpError::ConnectionFailed(_) => ApiError::ServiceUnavailable,
            HttpError::StatusError { status, body } => {
                ApiError::ExternalServiceError { status, message: body }
            }
            _ => ApiError::InternalError(err.to_string()),
        }
    }
}
```

---

## 常见问题

### Q: 前端如何处理跨域请求？

浏览器会自动处理 CORS。确保后端配置了正确的 CORS 头：

```
Access-Control-Allow-Origin: *
Access-Control-Allow-Methods: GET, POST, PUT, DELETE, OPTIONS
Access-Control-Allow-Headers: Content-Type, Authorization
```

### Q: 如何设置请求超时？

前端：

```typescript
const client = createHttpClient({
    timeout: 10000, // 10 秒
});
```

后端：

```rust
let config = HttpClientConfig {
    timeout: Duration::from_secs(10),
    ..Default::default()
};
```

### Q: 重试机制如何工作？

前后端都会自动重试以下情况：

- 状态码：408、429、500、502、503、504
- 错误类型：超时、连接失败、DNS 失败

配置重试次数和间隔：

```typescript
// 前端
const client = createHttpClient({
    maxRetries: 3,
    retryDelay: 1000,
});
```

```rust
// 后端
let config = HttpClientConfig {
    max_retries: 3,
    retry_delay: Duration::from_millis(1000),
    ..Default::default()
};
```

### Q: 如何处理文件上传？

前端：

```typescript
const formData = new FormData();
formData.append("file", fileBlob);

const response = await fetch("/api/upload", {
    method: "POST",
    body: formData,
});
```

后端：

```rust
use std::fs;

let file_content = fs::read("path/to/file")?;
let response = client
    .request("POST", "https://api.example.com/upload", Some(file_content), HashMap::new())
    .await?;
```

### Q: 如何调试请求？

前端添加日志拦截器：

```typescript
client.addRequestInterceptor((url, options) => {
    console.log(`[请求] ${options.method} ${url}`, options.body);
    return { url, options };
});

client.addResponseInterceptor((response, request) => {
    console.log(`[响应] ${request.url}`, response);
    return response;
});
```

后端使用日志：

```rust
let response = client.get("https://api.example.com/data").await?;
println!("响应状态: {}", response.status);
println!("响应头: {:?}", response.headers);
println!("响应体: {}", response.text());
```

### Q: 前端如何处理认证过期？

```typescript
client.addErrorInterceptor((error) => {
    if (error.type === "Authentication") {
        localStorage.removeItem("token");
        window.location.href = "/login";
    }
});
```

### Q: 后端如何自定义 User-Agent？

```rust
let config = HttpClientConfig {
    user_agent: "my-app/1.0 (contact@example.com)".to_string(),
    ..Default::default()
};
```
