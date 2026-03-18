# wae-observability

可观测性模块 - 提供统一的可观测性抽象层，包括健康检查、指标收集、日志记录和分布式追踪。

## 主要功能

- **健康检查**: 可插拔的健康检查接口
- **指标收集**: Prometheus 集成
- **日志记录**: JSON 格式化日志
- **分布式追踪**: OpenTelemetry OTLP 导出
- **性能分析**: pprof 集成
- **HTTP 集成**: 内置 Axum 中间件和路由

## 技术栈

- **异步运行时**: Tokio
- **Web 框架**: Axum
- **指标**: Prometheus (可选)
- **追踪**: OpenTelemetry (可选)
- **日志**: tracing-subscriber (可选)
- **性能分析**: pprof (可选)

## 使用示例

```rust
use wae_observability::{ObservabilityConfig, ObservabilityService, HealthCheck, HealthStatus};

#[tokio::main]
async fn main() {
    let config = ObservabilityConfig {
        service_name: "my-service".to_string(),
        service_version: "1.0.0".to_string(),
        ..Default::default()
    };
    
    let mut service = ObservabilityService::new(config);
    
    let report = service.health_check().await.unwrap();
    println!("Health status: {:?}", report.overall);
}
```

## 特性标志

- `default`: 无默认特性
- `health`: 启用健康检查功能
- `metrics`: 启用 Prometheus 指标收集
- `json-log`: 启用 JSON 格式化日志
- `otlp`: 启用 OpenTelemetry OTLP 导出
- `profiling`: 启用性能分析功能
- `full`: 启用所有功能
