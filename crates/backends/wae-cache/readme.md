# wae-cache

缓存模块 - 提供统一的缓存抽象层。

## 主要功能

- **多后端支持**: 支持 Redis、内存缓存等多种后端
- **异步 API**: 全异步接口设计
- **TTL 支持**: 支持过期时间设置
- **序列化**: 自动序列化/反序列化

## 技术栈

- **Redis**: redis (可选)
- **异步运行时**: Tokio
- **序列化**: serde

## 使用示例

```rust
use wae_cache::{CacheClient, CacheConfig};

#[tokio::main]
async fn main() {
    let client = CacheClient::new(CacheConfig {
        url: "redis://127.0.0.1:6379".to_string(),
    }).await?;
    
    client.set("user:001", &user, Some(3600)).await?;
    let user: Option<User> = client.get("user:001").await?;
    
    client.delete("user:001").await?;
}
```

## 缓存策略

- **Cache-Aside**: 应用层管理缓存
- **Write-Through**: 写入时同步更新缓存
- **Write-Behind**: 异步更新缓存
