# wae-cache

缓存服务抽象层，提供统一的缓存能力抽象，支持多种缓存后端。

## 概述

wae-cache 深度融合 tokio 运行时，所有 API 都是异步优先设计。通过 `CacheBackend` trait 抽象缓存操作，`CacheService` 提供类型安全的泛型封装，自动处理序列化/反序列化。支持分布式缓存、过期策略、批量操作等特性，适用于微服务架构。

## 快速开始

```rust
use wae_cache::{memory_cache, CacheConfig};
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cache = memory_cache(CacheConfig::default());
    
    cache.set("greeting", &"Hello, World!", Some(Duration::from_secs(60))).await?;
    
    let value: Option<String> = cache.get("greeting").await?;
    println!("{:?}", value);
    
    Ok(())
}
```

## 核心用法

### 创建缓存服务

使用 `memory_cache` 便捷函数创建内存缓存：

```rust
use wae_cache::{memory_cache, CacheConfig};
use std::time::Duration;

let config = CacheConfig {
    key_prefix: "myapp:".to_string(),
    default_ttl: Some(Duration::from_secs(600)),
    ..Default::default()
};

let cache = memory_cache(config);
```

`key_prefix` 会自动添加到所有键前，避免多应用共用缓存时的键冲突。`default_ttl` 设置默认过期时间，未指定 TTL 的缓存项将使用此值。

### 基本操作

```rust
use std::time::Duration;

cache.set("user:1", &user, Some(Duration::from_secs(300))).await?;

let user: Option<User> = cache.get("user:1").await?;

let exists: bool = cache.exists("user:1").await?;

let deleted: bool = cache.delete("user:1").await?;
```

`set` 支持任意可序列化类型，`get` 返回 `Option<T>`，键不存在时返回 `None`。

### 批量操作

批量操作可显著提升性能，减少网络往返：

```rust
let items = vec![
    ("user:1", &user1),
    ("user:2", &user2),
    ("user:3", &user3),
];
cache.mset(&items, Some(Duration::from_secs(300))).await?;

let values: Vec<Option<User>> = cache.mget(&["user:1", "user:2", "user:999"]).await?;

let deleted_count: u64 = cache.mdelete(&["user:1", "user:2"]).await?;
```

`mget` 返回的向量长度与输入键数量相同，键不存在对应位置为 `None`。

### 计数器

原子计数操作，适用于限流、统计等场景：

```rust
cache.set("api:calls", &0i64, Some(Duration::from_secs(3600))).await?;

let count = cache.incr("api:calls", 1).await?;
println!("当前调用次数: {}", count);

let count = cache.incr("api:calls", 10).await?;

let count = cache.decr("api:calls", 5).await?;
```

计数器值溢出时会回绕。对不存在的键执行 `incr`/`decr` 会从 0 开始计算。

### 过期管理

动态管理缓存项的过期时间：

```rust
cache.set("session:abc", &session_data, None).await?;

cache.expire("session:abc", Duration::from_secs(1800)).await?;

let remaining: Option<Duration> = cache.ttl("session:abc").await?;
match remaining {
    Some(d) => println!("剩余时间: {:?}", d),
    None => println!("键不存在或无过期时间"),
}
```

## 缓存策略

### 缓存穿透防护

缓存穿透指查询不存在的数据，请求穿透缓存直达数据库。解决方案是缓存空值：

```rust
async fn get_user(cache: &CacheService, id: u64) -> Result<Option<User>, CacheError> {
    let key = format!("user:{}", id);
    
    if let Some(user) = cache.get::<User>(&key).await? {
        return Ok(Some(user));
    }
    
    let user = db::find_user(id).await?;
    
    if let Some(u) = &user {
        cache.set(&key, u, Some(Duration::from_secs(300))).await?;
    } else {
        cache.set(&key, &"", Some(Duration::from_secs(60))).await?;
    }
    
    Ok(user)
}
```

### 缓存雪崩防护

缓存雪崩指大量缓存同时过期，导致数据库压力骤增。解决方案是添加随机过期时间：

```rust
use rand::Rng;

fn random_ttl(base: Duration, jitter: Duration) -> Duration {
    let jitter_secs = rand::thread_rng().gen_range(0..jitter.as_secs());
    base + Duration::from_secs(jitter_secs)
}

let ttl = random_ttl(Duration::from_secs(300), Duration::from_secs(60));
cache.set("hot_data", &data, Some(ttl)).await?;
```

### 缓存击穿防护

缓存击穿指热点数据过期瞬间大量请求穿透。解决方案是使用互斥锁或提前刷新：

```rust
async fn get_hot_data(cache: &CacheService) -> Result<Data, CacheError> {
    let key = "hot_data";
    
    if let Some(data) = cache.get::<Data>(key).await? {
        if let Some(ttl) = cache.ttl(key).await? {
            if ttl < Duration::from_secs(30) {
                tokio::spawn(async move {
                    let fresh = fetch_data().await.unwrap();
                    cache.set(key, &fresh, Some(Duration::from_secs(300))).await.unwrap();
                });
            }
        }
        return Ok(data);
    }
    
    let data = fetch_data().await?;
    cache.set(key, &data, Some(Duration::from_secs(300))).await?;
    Ok(data)
}
```

## 最佳实践

### 键命名规范

使用冒号分隔的命名空间，保持一致性：

```rust
cache.set("user:profile:123", &profile, None).await?;
cache.set("user:settings:123", &settings, None).await?;
cache.set("article:content:456", &content, None).await?;
cache.set("article:comments:456", &comments, None).await?;
```

### 合理设置 TTL

根据数据特性设置过期时间：

```rust
let session_ttl = Duration::from_secs(1800);
cache.set("session:token", &session, Some(session_ttl)).await?;

let config_ttl = Duration::from_secs(3600);
cache.set("config:theme", &config, Some(config_ttl)).await?;

let cache_ttl = Duration::from_secs(300);
cache.set("query:result:hash", &result, Some(cache_ttl)).await?;
```

### 使用 key_prefix 隔离环境

```rust
let config = CacheConfig {
    key_prefix: format!("{}:{}:", env!("APP_ENV"), "v1"),
    ..Default::default()
};
```

生成的键格式为 `production:v1:user:1`，便于环境隔离和版本管理。

### 批量操作优先

避免循环中执行单次操作：

```rust
// 不推荐
for user in &users {
    cache.set(&format!("user:{}", user.id), user, None).await?;
}

// 推荐
let items: Vec<_> = users.iter()
    .map(|u| (format!("user:{}", u.id).as_str(), u as &dyn Serialize))
    .collect();
cache.mset(&items, None).await?;
```

## 常见问题

### 如何存储复杂类型？

任何实现 `Serialize` 和 `DeserializeOwned` 的类型都可以直接存储：

```rust
#[derive(serde::Serialize, serde::Deserialize)]
struct User {
    id: u64,
    name: String,
    roles: Vec<String>,
}

cache.set("user:1", &User { id: 1, name: "Alice".into(), roles: vec!["admin".into()] }, None).await?;
let user: Option<User> = cache.get("user:1").await?;
```

### 如何清空缓存？

```rust
cache.clear().await?;
```

注意：生产环境慎用，会影响所有缓存数据。

### 如何判断键是否存在？

```rust
if cache.exists("user:1").await? {
    // 键存在
}
```

或通过 `get` 返回的 `Option` 判断：

```rust
match cache.get::<User>("user:1").await? {
    Some(user) => {},
    None => {},
}
```

### 如何处理序列化错误？

`CacheError::SerializationFailed` 和 `CacheError::DeserializationFailed` 包含详细错误信息：

```rust
match cache.get::<User>("user:1").await {
    Ok(Some(user)) => Ok(user),
    Ok(None) => Err("用户不存在".into()),
    Err(CacheError::DeserializationFailed(msg)) => {
        eprintln!("反序列化失败: {}", msg);
        Err("数据格式错误".into())
    }
    Err(e) => Err(e.into()),
}
```

### 内存缓存适合什么场景？

`MemoryCacheBackend` 适用于：
- 单元测试和集成测试
- 单机应用
- 临时数据缓存
- 开发环境

生产环境推荐使用 Redis 等分布式缓存后端。
