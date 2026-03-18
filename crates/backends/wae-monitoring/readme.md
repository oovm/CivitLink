# wae-monitoring

告警服务模块 - 提供统一的告警能力抽象，支持多种告警后端。

## 主要功能

- **多后端支持**: 支持 AlertManager 和 PagerDuty
- **异步 API**: 全异步接口设计
- **告警级别**: 支持 Info、Warning、Error、Critical 四级
- **标签和注释**: 灵活的告警元数据支持
- **批量发送**: 支持批量发送告警

## 技术栈

- **异步运行时**: Tokio
- **HTTP 客户端**: wae-request
- **序列化**: serde

## 使用示例

```rust
use wae_monitoring::{Alert, AlertSeverity, AlertManagerConfig, alertmanager_service};

#[tokio::main]
async fn main() {
    let config = AlertManagerConfig {
        base: AlertConfig {
            endpoint: "http://localhost:9093".to_string(),
            ..Default::default()
        },
    };
    
    let service = alertmanager_service(config).await?;
    
    let alert = Alert::new("service_down".to_string(), AlertSeverity::Critical)
        .with_label("service", "api-gateway")
        .with_annotation("description", "API Gateway is not responding");
    
    service.send_alert(alert).await?;
}
```

## 特性标志

- `default`: 无默认特性
- `alertmanager`: 启用 AlertManager 后端
- `pagerduty`: 启用 PagerDuty 后端
- `full`: 启用所有功能
