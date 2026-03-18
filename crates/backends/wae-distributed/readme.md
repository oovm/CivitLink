# wae-distributed

分布式模块 - 提供分布式系统支持。

## 主要功能

- **分布式锁**: 基于多种后端的分布式锁实现
- **服务注册**: 服务注册与发现
- **集群管理**: 节点状态监控
- **一致性协议**: 分布式一致性支持

## 技术栈

- **分布式锁**: 分布式锁实现
- **异步运行时**: Tokio

## 使用示例

```rust
use wae_distributed::{DistributedLock, LockConfig};

#[tokio::main]
async fn main() {
    let lock = DistributedLock::new(LockConfig {
        key: "resource:001",
        ttl: 30,
    });
    
    if lock.acquire().await? {
        let _guard = lock.guard();
    }
}
```

## 使用场景

- 分布式任务调度
- 资源竞争控制
- 幂等性保证
