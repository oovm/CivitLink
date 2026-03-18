# wae-resilience

弹性容错模块，提供微服务架构中的弹性容错能力，包括熔断器、限流器、重试、超时和舱壁隔离。

## 概述

本模块深度融合 tokio 运行时，所有 API 都是异步优先设计。在微服务架构中，外部服务可能不可用或响应缓慢，弹性容错机制能够保护系统免受级联故障影响，确保系统在高负载或部分故障时仍能提供稳定服务。

## 快速开始

最小可运行的弹性容错示例：

```rust
use wae::resilience::*;
use std::time::Duration;

async fn call_external_api() -> Result<String, String> {
    Ok("response".to_string())
}

async fn protected_call() -> ResilienceResult<String> {
    let pipeline = ResiliencePipelineBuilder::new("external-api")
        .circuit_breaker(CircuitBreakerConfig::default())
        .rate_limiter(TokenBucketConfig::new(100, 10))
        .retry(RetryConfig::default())
        .timeout(TimeoutConfig::default())
        .build();

    pipeline.execute(|| call_external_api()).await
}
```

## 熔断器

熔断器用于防止故障蔓延。当下游服务故障率达到阈值时，熔断器自动开启，拒绝后续请求，避免资源浪费。

### 基本使用

```rust
use wae::resilience::{CircuitBreaker, CircuitBreakerConfig, CircuitState};
use std::time::Duration;

let config = CircuitBreakerConfig::default()
    .failure_threshold(5)
    .success_threshold(3)
    .timeout(Duration::from_secs(30))
    .time_window(Duration::from_secs(60));

let circuit_breaker = CircuitBreaker::new("my-service", config);

let result = circuit_breaker.execute(|| async {
    call_downstream_service().await
}).await;

match circuit_breaker.state() {
    CircuitState::Closed => println!("正常运行"),
    CircuitState::Open => println!("熔断保护中"),
    CircuitState::HalfOpen => println!("试探性恢复"),
}
```

### 手动控制

需要更细粒度控制时，可以手动记录成功/失败：

```rust
let cb = CircuitBreaker::with_name("database");

loop {
    match cb.state() {
        CircuitState::Closed => {
            if cb.is_call_allowed() {
                match db_query().await {
                    Ok(_) => cb.record_success(),
                    Err(_) => cb.record_failure(),
                }
            }
        }
        CircuitState::Open => {
            println!("熔断器开启，等待恢复...");
            tokio::time::sleep(Duration::from_secs(5)).await;
        }
        CircuitState::HalfOpen => {
            println!("试探性恢复中...");
        }
    }
}
```

### 状态转换说明

| 状态 | 行为 | 转换条件 |
|------|------|----------|
| Closed | 正常运行，允许请求通过 | 连续失败次数达到 `failure_threshold` 时转为 Open |
| Open | 熔断保护，拒绝所有请求 | 等待 `timeout` 时间后转为 HalfOpen |
| HalfOpen | 试探性恢复，允许部分请求通过 | 成功次数达到 `success_threshold` 转为 Closed，失败则转回 Open |

## 限流器

限流器控制请求速率，防止系统过载。提供两种实现：令牌桶和滑动窗口。

### 令牌桶（TokenBucket）

适合允许突发流量的场景：

```rust
use wae::resilience::{TokenBucket, TokenBucketConfig, RateLimiter};

let config = TokenBucketConfig::new(100, 10);
let limiter = TokenBucket::new(config);

if limiter.try_acquire().is_ok() {
    println!("获取许可成功，剩余: {}", limiter.available_permits());
}

limiter.acquire().await?;
```

**参数说明**：
- `capacity`: 桶容量，最大令牌数，决定突发流量上限
- `refill_rate`: 每秒补充的令牌数，决定平均请求速率

### 滑动窗口（SlidingWindow）

适合需要精确控制请求速率的场景：

```rust
use wae::resilience::{SlidingWindow, SlidingWindowConfig, RateLimiter};
use std::time::Duration;

let config = SlidingWindowConfig::new(Duration::from_secs(1), 100);
let limiter = SlidingWindow::new(config);

match limiter.try_acquire() {
    Ok(()) => println!("请求通过"),
    Err(_) => println!("请求被限流"),
}
```

**参数说明**：
- `window_size`: 时间窗口大小
- `max_requests`: 窗口内最大请求数

### 选择建议

| 场景 | 推荐实现 |
|------|----------|
| API 网关、允许短时突发 | TokenBucket |
| 精确限流、防止瞬时过载 | SlidingWindow |
| 简单限流需求 | 两者皆可，TokenBucket 更灵活 |

## 重试机制

重试机制在临时故障时自动重试，提高请求成功率。

### 基本重试

```rust
use wae::resilience::{retry_async, RetryConfig, RetryContext};

let config = RetryConfig::default();

let result = retry_async(&config, |ctx: RetryContext| async move {
    println!("尝试第 {} 次", ctx.attempt);
    call_external_service().await
}).await?;
```

### 重试策略

```rust
use wae::resilience::{RetryConfig, RetryPolicy};
use std::time::Duration;

let exponential = RetryConfig::new(3, RetryPolicy::exponential(
    Duration::from_millis(100),
    Duration::from_secs(30),
    2.0,
));

let fixed = RetryConfig::default()
    .max_retries(5)
    .policy(RetryPolicy::fixed(Duration::from_secs(1)));

let linear = RetryConfig::new(3, RetryPolicy::linear(
    Duration::from_millis(100),
    Duration::from_millis(200),
    Duration::from_secs(10),
));
```

| 策略 | 适用场景 | 间隔计算 |
|------|----------|----------|
| ExponentialBackoff | 网络抖动、服务重启 | 每次间隔翻倍 |
| FixedInterval | 简单重试 | 固定间隔 |
| LinearBackoff | 渐进恢复 | 每次间隔线性增加 |

### 条件重试

只对特定错误重试：

```rust
use wae::resilience::{retry_async_if, RetryConfig};

let result = retry_async_if(
    &RetryConfig::default(),
    |ctx| async move {
        call_external_service().await
    },
    |error: &String| error.contains("temporary") || error.contains("timeout"),
).await?;
```

### 重试上下文

`RetryContext` 提供当前重试状态：

```rust
retry_async(&config, |ctx| async move {
    println!("第 {} 次尝试，最多 {} 次", ctx.attempt, ctx.max_retries);
    if let Some(last_error) = &ctx.last_error {
        println!("上次错误: {}", last_error);
    }
    call_external_service().await
}).await?;
```

## 超时控制

超时控制防止请求长时间阻塞。

### 基本超时

```rust
use wae::resilience::with_timeout;
use std::time::Duration;

let result = with_timeout(Duration::from_secs(5), async {
    call_slow_service().await
}).await?;
```

### 保留原始错误类型

需要区分超时错误和业务错误时：

```rust
use wae::resilience::with_timeout_raw;
use std::time::Duration;

let result = with_timeout_raw(Duration::from_secs(5), async {
    call_service().await
}).await;

match result {
    Ok(Ok(response)) => println!("成功: {}", response),
    Ok(Err(business_error)) => println!("业务错误: {}", business_error),
    Err(ResilienceError::Timeout(duration)) => println!("超时: {:?}", duration),
    Err(e) => println!("其他错误: {}", e),
}
```

### 配置超时

```rust
use wae::resilience::TimeoutConfig;
use std::time::Duration;

let config = TimeoutConfig::new(
    Duration::from_secs(60),
    Duration::from_secs(10),
);
```

## 舱壁隔离

舱壁隔离限制并发调用数，防止资源耗尽。

### 基本使用

```rust
use wae::resilience::{Bulkhead, BulkheadConfig};
use std::time::Duration;

let config = BulkheadConfig::new(10, Duration::from_secs(5));
let bulkhead = Bulkhead::new(config);

let result = bulkhead.execute(|| async {
    call_external_service().await
}).await?;

println!("当前并发数: {}", bulkhead.concurrent_count());
println!("可用许可: {}", bulkhead.available_permits());
```

### 手动获取许可

需要跨多个操作共享许可时：

```rust
let bulkhead = Bulkhead::with_defaults();

let permit = bulkhead.acquire().await?;
{
    let _permit = permit;
    call_external_service().await?;
}
```

### 配置说明

| 参数 | 说明 | 默认值 |
|------|------|--------|
| `max_concurrent_calls` | 最大并发调用数 | 10 |
| `max_wait_duration` | 获取许可的最大等待时间 | 5s |

## 组合管道

组合多种弹性策略，构建完整的容错方案。

### 构建管道

```rust
use wae::resilience::*;

let pipeline = ResiliencePipelineBuilder::new("api-service")
    .circuit_breaker(CircuitBreakerConfig::default()
        .failure_threshold(10)
        .timeout(Duration::from_secs(60)))
    .rate_limiter(TokenBucketConfig::new(200, 20))
    .retry(RetryConfig::new(3, RetryPolicy::exponential(
        Duration::from_millis(200),
        Duration::from_secs(60),
        2.0,
    )))
    .timeout(TimeoutConfig::new(
        Duration::from_secs(60),
        Duration::from_secs(10),
    ))
    .bulkhead(BulkheadConfig::new(20, Duration::from_secs(10)))
    .build();

let result = pipeline.execute(|| async {
    call_external_service().await
}).await?;
```

### 执行顺序

管道按以下顺序执行：

1. **Bulkhead** - 检查并发限制
2. **RateLimiter** - 检查限流
3. **CircuitBreaker** - 检查熔断状态
4. **Timeout** - 设置超时
5. **Retry** - 执行并处理重试

### 选择性组合

可以只启用需要的策略：

```rust
let pipeline = ResiliencePipelineBuilder::new("simple-service")
    .circuit_breaker(CircuitBreakerConfig::default())
    .retry(RetryConfig::default())
    .build();
```

## 最佳实践

### 配置建议

**熔断器**：
- `failure_threshold`: 根据服务重要性设置，关键服务设置较低值（3-5）
- `timeout`: 设置为服务平均响应时间的 2-3 倍
- `success_threshold`: 设置为 2-3，快速恢复

**限流器**：
- 根据下游服务容量设置
- 预留 20% 余量应对突发

**重试**：
- 幂等操作才重试
- 设置合理的最大重试次数（通常 3 次）
- 使用指数退避避免雪崩

**超时**：
- 设置为 P99 响应时间的 2 倍
- 全局超时应大于操作超时

**舱壁隔离**：
- 根据资源类型设置不同隔离池
- 数据库连接池大小应与舱壁大小匹配

### 错误处理

```rust
match pipeline.execute(|| call_service()).await {
    Ok(result) => handle_success(result),
    Err(ResilienceError::CircuitBreakerOpen) => {
        handle_fallback()
    }
    Err(ResilienceError::RateLimitExceeded) => {
        handle_rate_limited()
    }
    Err(ResilienceError::MaxRetriesExceeded(e)) => {
        handle_retry_exhausted(&e)
    }
    Err(ResilienceError::Timeout(duration)) => {
        handle_timeout(duration)
    }
    Err(ResilienceError::BulkheadFull(available)) => {
        handle_bulkhead_full(available)
    }
    Err(ResilienceError::BulkheadWaitTimeout) => {
        handle_bulkhead_timeout()
    }
    Err(ResilienceError::Internal(e)) => {
        handle_internal_error(&e)
    }
}
```

### 监控指标

建议监控以下指标：

- 熔断器状态变化次数
- 限流拒绝次数
- 重试次数和成功率
- 超时次数
- 舱壁等待时间和拒绝次数

## 常见问题

### 熔断器一直开启怎么办？

检查下游服务是否真的故障。如果服务已恢复但熔断器未关闭，可能是 `success_threshold` 设置过高，或 `timeout` 时间过长。

### 重试导致请求量翻倍？

确保使用指数退避策略，并设置合理的最大重试次数。对于高并发场景，考虑在重试前加入随机抖动。

### 限流器选择困难？

- 需要允许短时突发：选择 TokenBucket
- 需要严格限制每秒请求数：选择 SlidingWindow
- 不确定：从 TokenBucket 开始，它更灵活

### 舱壁隔离导致请求排队？

检查 `max_concurrent_calls` 是否设置过小，或 `max_wait_duration` 是否过短。可以通过监控 `available_permits` 了解资源使用情况。

### 多个弹性策略如何组合？

推荐组合：
- 外部 API 调用：熔断器 + 重试 + 超时
- 数据库访问：舱壁隔离 + 超时
- 高并发场景：限流器 + 舱壁隔离 + 熔断器

### 如何实现降级？

```rust
match pipeline.execute(|| call_service()).await {
    Ok(result) => result,
    Err(ResilienceError::CircuitBreakerOpen) |
    Err(ResilienceError::MaxRetriesExceeded(_)) => {
        get_cached_result().await
    }
    Err(e) => return Err(e),
}
```
