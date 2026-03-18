# wae-request

请求模块 - 提供 HTTP 客户端功能。

## 主要功能

- **HTTP 客户端**: GET、POST、PUT、DELETE 等请求
- **请求拦截器**: 请求/响应拦截处理
- **重试机制**: 自动重试失败请求
- **超时控制**: 请求超时配置

## 技术栈

- **HTTP 客户端**: hyper, hyper-tls
- **异步运行时**: Tokio
- **序列化**: serde

## 使用示例

```rust
use wae_request::{HttpClient, HttpClientConfig};

#[tokio::main]
async fn main() {
    let client = HttpClient::new(HttpClientConfig {
        base_url: "https://api.example.com".to_string(),
        timeout: 30,
    });
    
    let response = client
        .get("/users")
        .header("Authorization", "Bearer token")
        .send()
        .await?;
    
    let users: Vec<User> = response.json().await?;
    
    let created = client
        .post("/users")
        .json(&NewUser { name: "张三".to_string() })
        .send()
        .await?;
}
```

## 拦截器

```rust
client.add_request_interceptor(|req| {
    req.header("X-Request-Id", uuid::Uuid::new_v4().to_string());
    req
});

client.add_response_interceptor(|res| async move {
    println!("Response status: {}", res.status());
    res
});
```
