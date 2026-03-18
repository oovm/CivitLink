# wae-service

服务模块 - 提供服务发现和注册功能。

## 主要功能

- **服务注册**: 服务实例注册
- **服务发现**: 自动发现服务实例
- **健康检查**: 服务健康状态监控
- **负载均衡**: 多种负载均衡策略

## 技术栈

- **服务发现**: 多后端支持
- **异步运行时**: Tokio

## 使用示例

```rust
use wae_service::{ServiceRegistry, ServiceInstance, LoadBalancer};

#[tokio::main]
async fn main() {
    let registry = ServiceRegistry::new(RegistryConfig {
        backend: RegistryBackend::Consul("http://localhost:8500".to_string()),
    });
    
    registry.register(ServiceInstance {
        name: "user-service".to_string(),
        address: "192.168.1.100".to_string(),
        port: 8080,
        tags: vec!["v1".to_string()],
    }).await?;
    
    let instances = registry.discover("user-service").await?;
    
    let lb = LoadBalancer::round_robin(instances);
    let instance = lb.select();
}
```

## 负载均衡策略

| 策略 | 说明 |
|------|------|
| RoundRobin | 轮询 |
| Random | 随机 |
| WeightedRoundRobin | 加权轮询 |
| LeastConnections | 最少连接 |

## 支持的注册中心

- Consul
- etcd
- Nacos
