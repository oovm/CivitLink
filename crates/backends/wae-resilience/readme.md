# wae-resilience

弹性模块 - 提供系统弹性模式支持。

## 主要功能

- **熔断器**: 防止级联故障
- **限流器**: 控制请求速率
- **重试策略**: 智能重试机制
- **舱壁隔离**: 资源隔离保护

## 技术栈

- **弹性模式**: 多种弹性模式实现
- **异步运行时**: Tokio

## 使用示例

```rust
use wae_resilience::{CircuitBreaker, RateLimiter, RetryPolicy};

#[tokio::main]
async fn main() {
    let breaker = CircuitBreaker::new(CircuitBreakerConfig {
        failure_threshold: 5,
        success_threshold: 3,
        timeout: Duration::from_secs(30),
    });
    
    let limiter = RateLimiter::new(RateLimiterConfig {
        max_requests: 100,
        window: Duration::from_secs(1),
    });
    
    let result = breaker.call(|| async {
        limiter.acquire().await?;
        external_service_call().await
    }).await;
}
```

## 熔断器状态

| 状态 | 说明 |
|------|------|
| Closed | 正常状态，请求正常通过 |
| Open | 熔断状态，请求直接失败 |
| HalfOpen | 半开状态，允许部分请求通过测试 |

## 重试策略

```rust
let retry = RetryPolicy::exponential_backoff(3, Duration::from_millis(100));
let result = retry.execute(|| async {
    unreliable_service().await
}).await;
```
