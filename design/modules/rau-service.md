# wae-service

服务发现与注册模块，提供统一的服务发现与注册抽象，支持多种注册中心。

## 概述

深度融合 tokio 运行时，支持服务注册与注销、服务发现与健康检查、负载均衡策略、服务元数据管理。

## 快速开始

```rust
use wae_service::{ServiceInstance, ServiceRegistry, ServiceDiscovery, ServiceStatus, LoadBalancer, LoadBalanceStrategy};
use std::sync::Arc;
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let instance = ServiceInstance {
        id: "user-service-1".to_string(),
        name: "user-service".to_string(),
        addr: "127.0.0.1:3000".parse()?,
        metadata: [("version".to_string(), "1.0.0".to_string())].into_iter().collect(),
        tags: vec!["api".to_string()],
        registered_at: chrono::Utc::now(),
        last_heartbeat: chrono::Utc::now(),
        weight: 1,
        status: ServiceStatus::Healthy,
    };
    
    registry.register(&instance).await?;
    println!("Service registered: {}", instance.name);
    
    let instances = discovery.discover("user-service").await?;
    let lb = LoadBalancer::new(LoadBalanceStrategy::RoundRobin);
    if let Some(selected) = lb.select(&instances) {
        println!("Selected: {}", selected.addr);
    }
    
    Ok(())
}
```

## 核心用法

### 服务注册

```rust
use wae_service::{ServiceInstance, ServiceRegistry, ServiceStatus};
use chrono::Utc;

let instance = ServiceInstance {
    id: "user-service-1".to_string(),
    name: "user-service".to_string(),
    addr: "127.0.0.1:3000".parse()?,
    metadata: HashMap::new(),
    tags: vec!["api".to_string()],
    registered_at: Utc::now(),
    last_heartbeat: Utc::now(),
    weight: 1,
    status: ServiceStatus::Healthy,
};

registry.register(&instance).await?;
```

### 服务发现

```rust
use wae_service::{ServiceDiscovery, LoadBalancer, LoadBalanceStrategy};

let instances = discovery.discover("user-service").await?;
println!("Found {} instances", instances.len());

let lb = LoadBalancer::new(LoadBalanceStrategy::RoundRobin);
if let Some(instance) = lb.select(&instances) {
    println!("Selected: {}", instance.addr);
}
```

### 订阅服务变更

```rust
let mut subscription = discovery.subscribe("user-service").await?;

loop {
    let instances = subscription.wait_for_change().await?;
    println!("Updated: {} instances", instances.len());
}
```

### 心跳与健康检查

```rust
// 发送心跳
registry.heartbeat("user-service-1").await?;

// 更新状态
registry.update_status("user-service-1", ServiceStatus::Maintenance).await?;

// 注销服务
registry.deregister("负载均衡策略

| 策略 | 适用场景 | 特点 |
|------|----------|------|
| `RoundRobin` | 无状态服务 | 轮询，均匀分布 |
| `Random` | 简单场景 | 随机选择 |
| `WeightedRoundRobin` | 异构实例 | 按权重分配 |
| `LeastConnections` | 长连接场景 | 选择连接最少的实例 |

```rust
use wae_service::{LoadBalancer, LoadBalanceStrategy};

// 轮询
let lb = LoadBalancer::new(LoadBalanceStrategy::RoundRobin);

// 加权轮询 - 适用于不同配置的服务器
let lb = LoadBalancer::new(Locator::new(LoadBalanceStrategy::WeightedRoundRobin);

// 最少连接 - 适用于长连接场景
let lb = LoadBalancer::new(LoadBalanceStrategy::LeastConnections);
```

## 服务注册配置

```rust
use wae_service::{ServiceRegistration, HealthCheckConfig};
use std::time::Duration;

let registration = ServiceRegistration {
    name: "order-service".to_string(),
    addr: "0.0.0.0:4000".parse()?,
    metadata: [("version".to_string(), "1.0.0".to_string())]
        .into_iter()
        .collect(),
    tags: vec!["api".to_string(), "order".to_string()],
    weight: 2,
    heartbeat_interval: Duration::from_secs(10),
    health_check: Some(HealthCheckConfig {
        path: "/health".to_string(),
        interval: Duration::from_secs(30),
        timeout: Duration::from_secs(5),
        failure_threshold: 3,
    }),
};
```

## 最佳实践

### 服务启动流程

```rust
async fn start_service() -> Result<(), Box<dyn std::error::Error>> {
    let instance = create_instance();
    
    // 1. 注册服务
    registry.register(&instance).await?;
    
    // 2. 启动心跳任务
    let handle = tokio::spawn(async move {
        loop {
            tokio::time::sleep(Duration::from_secs(10)).await;
            if let Err(e) = registry.heartbeat(&instance.id).await {
                eprintln!("Heartbeat failed: {}", e);
            }
        }
    });
    
    // 3. 运行服务
    run_server().await;
    
    // 4. 停止心跳
    handle.abort();
    
    // 5. 注销服务
    registry.deregister(&instance.id).await?;
    
    Ok(())
}
```

### 服务调用

```rust
async fn call_service(
    discovery: &dyn ServiceDiscovery,
    lb: &LoadBalancer,
) -> Result<HttpResponse, Box<dyn std::error::Error>> {
    // 1. 发现服务
    let instances = discovery.discover("target-service").await?;
    if instances.is_empty() {
        return Err("No instances available".into());
    }
    
    // 2. 负载均衡
    let instance = lb.select(&instances)
        .ok_or("No healthy instance")?;
    
    // 3. 调用服务
    let response = http_client
        .get(&format!("http://{}/api", instance.addr))
        .await?;
    
    Ok(response)
}
```

## 常见问题

### 如何实现服务健康检查？

通过 `HealthCheckConfig` 配置 HTTP 健康检查端点，服务发现组件会定期检查该端点，失败次数超过阈值后自动标记为 Unhealthy。

### 如何实现灰度发布？

使用 metadata 标记版本，客户端根据版本过滤实例：

```rust
let v2_instances: Vec<_> = instances
    .into_iter()
    .filter(|i| i.metadata.get("version") == Some(&"2.0.0".to_string()))
    .collect();
```

### 如何实现服务熔断？

结合 `wae-resilience` 模块的 CircuitBreaker：

```rust
use wae_resilience::{CircuitBreaker, CircuitBreakerConfig};

let cb = CircuitBreaker::new("user-service", CircuitBreakerConfig::default());

let result = cb.execute(|| async {
    call_service(&discovery, &lb).await
}).await;
```

### 如何处理服务不可用？

结合重试机制：

```rust
use wae_resilience::{retry_async, RetryConfig, RetryPolicy};
use std::time::Duration;

let config = RetryConfig::new(3, RetryPolicy::fixed(Duration::from_secs(1)));

let result = retry_async(&config, |ctx| async move {
    let instance = discovery.discover_one("user-service").await?
        .ok_or("No instance")?;
    call_service(&instance).await
}).await;
```

### 如何实现本地开发环境的服务发现？

使用内存实现进行本地测试：

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use wae_service::InMemoryRegistry;

    #[tokio::test]
    async fn test_service_discovery() {
        let registry = InMemoryRegistry::new();
        // 测试逻辑...
    }
}
```
