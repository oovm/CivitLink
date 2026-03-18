# observability-basic

可观测性基础示例 - 演示 WAE Observability 模块的健康检查、日志记录和指标收集功能。

## 示例说明

本示例展示如何使用 `wae-observability` 模块构建具备完整可观测性能力的服务，包括：

- 自定义健康检查器的实现
- 健康状态聚合与报告
- 使用内存数据结构进行指标收集
- 结构化日志记录
- 不同健康状态（Passing/Warning/Critical）的演示

## 主要功能演示

### 1. 健康检查

实现了两个自定义健康检查器：

- **内存健康检查器** (`MemoryHealthChecker`) - 模拟内存使用率监控
  - 0-60%: Passing (健康)
  - 60-80%: Warning (警告)
  - 80%+: Critical (严重)
  
- **服务健康检查器** (`ServiceHealthChecker`) - 服务运行状态检查

### 2. 指标收集

使用内存数据结构模拟指标存储，包含：

- 请求计数 (`request_count`)
- 活跃用户数 (`active_users`)

### 3. 日志记录

使用 `tracing` 进行结构化日志记录，支持：

- 不同日志级别 (info, warn, error)
- 事件追踪

## 运行示例

```bash
cd examples/observability-basic
cargo run
```

## 代码示例

### 创建健康检查器

```rust
use std::sync::Arc;
use wae_observability::{HealthChecker, HealthCheck, HealthStatus};

#[derive(Debug, Clone)]
struct MemoryHealthChecker {
    name: String,
    memory_usage: Arc<std::sync::Mutex<u64>>,
}

#[async_trait::async_trait]
impl HealthChecker for MemoryHealthChecker {
    async fn check(&self) -> HealthCheck {
        let usage = *self.memory_usage.lock().unwrap();
        let status = if usage > 80 {
            HealthStatus::Critical
        } else if usage > 60 {
            HealthStatus::Warning
        } else {
            HealthStatus::Passing
        };
        HealthCheck::new(self.name.clone(), status)
            .with_details(format!("当前内存使用率: {}%", usage))
    }
}
```

### 初始化可观测性服务

```rust
use wae_observability::{ObservabilityConfig, init_observability};

let config = ObservabilityConfig::new("my-service".to_string(), "1.0.0".to_string())
    .with_health();

let mut observability = init_observability(config)?;
observability.add_health_checker(Box::new(my_checker));
```

### 执行健康检查

```rust
let report = observability.health_check().await?;
println!("整体状态: {:?}", report.overall);
for check in report.checks {
    println!("  - {}: {:?}", check.name, check.status);
}
```

## 输出示例

```
=== WAE Observability 基础示例 ===

=== 功能演示 ===
1. 健康检查 - 模拟内存和服务状态检查
2. 指标收集 - 使用内存数据结构模拟指标存储
3. 日志记录 - 使用 tracing 进行结构化日志

正在执行健康检查...
健康检查结果:
  整体状态: Passing
  - memory-check: Passing (Some("当前内存使用率: 0%"))
  - service-check: Passing (Some("服务运行正常"))

正在设置指标数据...
  请求计数: 2
  活跃用户: 150

正在更新内存使用率...
更新后的健康检查:
  整体状态: Passing
  - memory-check: Passing (Some("当前内存使用率: 45%"))
  - service-check: Passing (Some("服务运行正常"))

正在模拟高内存使用率警告...
警告状态下的健康检查:
  整体状态: Warning
  - memory-check: Warning (Some("当前内存使用率: 70%"))
  - service-check: Passing (Some("服务运行正常"))

正在模拟严重内存使用率...
严重状态下的健康检查:
  整体状态: Critical
  - memory-check: Critical (Some("当前内存使用率: 90%"))
  - service-check: Passing (Some("服务运行正常"))

=== 示例说明 ===
本示例演示了 WAE Observability 模块的核心功能:
- 自定义健康检查器的实现和集成
- 使用内存数据结构进行指标收集
- 结构化日志记录
- 健康状态聚合
- 不同健康状态（Passing/Warning/Critical）的演示
```

## 依赖

- `wae-observability` - 可观测性服务核心模块
- `wae-types` - 通用类型定义
- `tokio` - 异步运行时
- `serde` - 序列化/反序列化
- `tracing` - 结构化日志
