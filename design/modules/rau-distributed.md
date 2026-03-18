# wae-distributed

分布式能力抽象层，提供统一的分布式系统能力抽象，包括分布式锁、功能开关、分布式 ID 生成等。

## 概述

深度融合 tokio 运行时，所有 API 都是异步优先设计。微服务架构友好，支持高可用、高并发场景。

## 快速开始

```rust
use wae_distributed::{InMemoryLockManager, DistributedLock, FeatureFlagManager, FlagDefinition, Strategy, SnowflakeGenerator, IdGenerator};
use std::sync::Arc;
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 分布式锁
    let lock_manager = Arc::new(InMemoryLockManager::new());
    let lock = lock_manager.create_lock("resource-1");
    
    if lock.try_lock().await? {
        println!("Lock acquired");
        // 执行需要互斥的操作
        lock.unlock().await?;
    }
    
    // 功能开关
    let flag_manager = FeatureFlagManager::new();
    flag_manager.register(
        FlagDefinition::new("new-feature")
            .with_strategy(Strategy::On)
            .with_enabled(true)
    );
    
    if flag_manager.is_enabled("new-feature").await {
        println!("Feature is enabled");
    }
    
    // ID 生成
    let id_gen = SnowflakeGenerator::new(1, 1)?;
    let id = id_gen.generate().await;
    println!("Generated ID: {}", id);
    
    Ok(())
}
```

## 分布式锁

### 基本用法

```rust
use wae_distributed::{InMemoryLockManager, DistributedLock};
use std::sync::Arc;

let manager = Arc::new(InMemoryLockManager::new());
let lock = manager.create_lock("resource-1");

// 尝试获取锁，立即返回
if lock.try_lock().await? {
    // 获取成功
    lock.unlock().await?;
}

// 阻塞等待获取锁
lock.lock().await?;
// 执行操作
lock.unlock().await?;

// 带超时的获取
if lock.lock_with_timeout(Duration::from_secs(5)).await.is_ok() {
    // 在 5 秒内获取成功
    lock.unlock().await?;
}
```

### 锁选项

```rust
use wae_distributed::LockOptions;

let options = LockOptions::new()
    .with_ttl(Duration::from_secs(30))  // 锁的生存时间
    .with_wait_timeout(Duration::from_secs(5)); // 等待超时
```

### 生产环境建议

生产环境应使用 Redis 等分布式存储实现分布式锁，`InMemoryLock` 仅适用于单机测试。

## 功能开关

### 基本用法

```rust
use wae_service::{FeatureFlagManager, FlagDefinition, Strategy};

let manager = FeatureFlagManager::new();

// 注册开关
let flag = FlagDefinition::new("new-feature")
    .with_description("Enable new feature")
    .with_strategy(Strategy::On)
    .with_enabled(true);

manager.register(flag);

// 检查状态
if manager.is_enabled("new-feature").await {
    println!("Feature is enabled");
}
```

### 策略类型

| 策略 | 说明 | 适用场景 |
|------|------|----------|
| `On` | 始终开启 | 全量发布 |
| `Off` | 始终关闭 | 紧急回滚 |
| `Percentage(n)` | 按百分比灰度 | 渐进发布 |
| `UserList(users)` | 用户白名单 | 内测、灰度测试 |

### 百分比灰度

```rust
let flag = FlagDefinition::new("beta-feature")
    .with_strategy(Strategy::Percentage(30)) // 30% 用户可见
    .with_enabled(true);

manager.register(flag);

if manager.is_enabled_for_user("beta-feature", "user-123").await {
    println!("User has access to beta feature");
}
```

### 用户白名单

```rust
let flag = FlagID::new("admin-feature")
    .with_strategy(Strategy::UserList(vec![
        "admin-1".to_string(),
        "admin-2".to_string(),
    ]))
    .with_enabled(true);

manager.register(flag);
```

### 动态更新

```rust
// 更新开关状态
manager.update("new-feature", false);

// 移除开关
manager.unregister("old-feature");
```

## ID 生成器

### 雪花算法

```rust
use wae_distributed::{SnowflakeGenerator, IdGenerator};

// worker_id: 机器 ID，datacenter_id: 数据中心 ID
let generator = SnowflakeGenerator::new(1, 1)?;

let id = generator.generate().await;
println!("ID: {}", id);

let ids = generator.generate_batch(10).await;
println!("Generated {} IDs", ids.len());
```

**特点**：
- 时间有序
- 趋势递增
- 分布式唯一
- 性能高

### UUID

```rust
use wae_distributed::{UuidGenerator, IdGenerator, UuidVersion};

// V4: 随机 UUID
let gen_v4 = UuidGenerator::v4();
let id = gen_v4.generate().await;
println!("V4 UUID: {}", id);

// V7: 时间有序 UUID
let gen_v7 = UuidGenerator::v7();
let id = gen_v7.generate().await;
println!("V7 UUID: {}", id);
```

**选择建议**：
- 需要时间排序：选择 V7
- 不依赖时间：选择 V4

## 最佳实践

### 分布式锁模式

```rust
async fn with_lock<T>(
    lock: &InMemoryLock,
    f: impl Future<Output = T>,
) -> Result<T, LockError> {
    lock.lock().await?;
    let result = f.await;
    lock.unlock().await?;
    Ok(result)
}

// 使用
let result = with_lock(&lock, async {
    // 临界区操作
    42
}).await?;
```

### 功能开关封装

```rust
struct FeatureService {
    manager: FeatureFlagManager,
}

impl FeatureService {
    async fn is_feature_enabled(&self, feature: &str, user_id: &str) -> bool {
        // 先检查用户特定权限
        if self.manager.is_enabled_for_user(feature, user_id).await {
            return true;
        }
        // 再检查全局开关
        self.manager.is_enabled(feature).await
    }
}
```

### ID 生成服务

```rust
use std::sync::Arc;

struct IdService {
    snowflake: Arc<SnowflakeGenerator>,
    uuid_v4: UuidGenerator,
    uuid_v7: UuidGenerator,
}

impl IdService {
    fn new(worker_id: u64, datacenter_id: u64) -> Result<Self, String> {
        Ok(Self {
            snowflake: Arc::new(SnowflakeGenerator::new(worker_id, datacenter_id)?),
            uuid_v4: UuidGenerator::v4(),
            uuid_v7: UuidGenerator::v7(),
        })
    }
    
    async fn generate_order_id(&self) -> String {
        self.snowflake.generate().await
    }
    
    async fn generate_session_id(&self) -> String {
        self.uuid_v7.generate().await
    }
}
```

## 常见问题

### 如何处理锁超时？

```rust
match lock.lock_with_timeout(Duration::from_secs(5)).await {
    Ok(_) => {
        // 获取成功
        lock.unlock().await?;
    }
    Err(LockError::WaitTimeout) => {
        println("Could not acquire lock within timeout");
    }
    Err(e) => return Err(e.into()),
}
```

### 如何实现分布式锁的自动续期？

```rust
async fn with_auto_renew<T>(
    lock: &InMemoryLock,
    ttl: Duration,
    f: impl Future<Output = T>,
) -> Result<T, LockError> {
    lock.lock().await?;
    
    let (tx, mut rx) = tokio::sync::onshot::channel();
    let lock_clone = lock.clone();
    
    // 续期任务
    let handle = tokio::spawn(async move {
        let mut interval = tokio::time::interval(ttl / 2);
        loop {
            tokio::select! {
                _ = interval.tick() => {
                    // 续期逻辑
                }
                _ = &mut rx => break,
            }
        }
    });
    
    let result = f.await;
    let _ = tx.send(());
    handle.abort();
    lock.unlock().await?;
    Ok(result)
}
```

### 如何实现多变量测试？

```rust
let variants = vec![
    ("button-blue", FlagDefinition::new("button-color")
        .with_strategy(Strategy::Percentage(33))
        .with_enabled(true)),
    ("button-red", FlagDefinition::new("button-color")
        .with_strategy(Strategy::Percentage(33))
        .with_enabled(true)),
];

for (variant, flag) in variants {
    manager.register(flag);
}

// 获取变体
let variant = manager.get_variant("button-color").await;
```

### 如何保证 ID 唯一性？

- 雪花算法：确保 worker_id 和 datacenter_id 组合唯一
- UUID：算法本身保证唯一性
- 数据库自增：使用数据库事务保证

### 如何在测试中使用内存锁？

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_with_lock() {
        let manager = Arc::new(InMemoryLockManager::new());
        let lock = manager.create_lock("test-lock");
        
        assert!(lock.try_lock().await?);
        assert!(!lock.try_lock().await?); // 已被锁定
        lock.unlock().await?;
        assert!(lock.try_lock().await?);
        
        Ok(())
    }
}
```
