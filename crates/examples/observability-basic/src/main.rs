use async_trait::async_trait;
use std::sync::Arc;
use wae_observability::{HealthCheck, HealthChecker, HealthStatus, ObservabilityConfig, init_observability};

#[derive(Debug, Clone)]
struct MemoryHealthChecker {
    name: String,
    memory_usage: Arc<std::sync::Mutex<u64>>,
}

impl MemoryHealthChecker {
    fn new(name: String) -> Self {
        Self { name, memory_usage: Arc::new(std::sync::Mutex::new(0)) }
    }

    fn set_usage(&self, usage: u64) {
        let mut m = self.memory_usage.lock().unwrap();
        *m = usage;
    }
}

#[async_trait]
impl HealthChecker for MemoryHealthChecker {
    async fn check(&self) -> HealthCheck {
        let usage = *self.memory_usage.lock().unwrap();
        let status = if usage > 80 {
            HealthStatus::Critical
        }
        else if usage > 60 {
            HealthStatus::Warning
        }
        else {
            HealthStatus::Passing
        };
        HealthCheck::new(self.name.clone(), status).with_details(format!("当前内存使用率: {}%", usage))
    }
}

#[derive(Debug, Clone)]
struct InMemoryMetrics {
    request_count: Arc<std::sync::Mutex<u64>>,
    active_users: Arc<std::sync::Mutex<u64>>,
}

impl InMemoryMetrics {
    fn new() -> Self {
        Self { request_count: Arc::new(std::sync::Mutex::new(0)), active_users: Arc::new(std::sync::Mutex::new(0)) }
    }

    fn increment_request(&self) {
        let mut count = self.request_count.lock().unwrap();
        *count += 1;
    }

    fn set_active_users(&self, count: u64) {
        let mut users = self.active_users.lock().unwrap();
        *users = count;
    }

    fn get_request_count(&self) -> u64 {
        *self.request_count.lock().unwrap()
    }

    fn get_active_users(&self) -> u64 {
        *self.active_users.lock().unwrap()
    }
}

#[derive(Debug, Clone)]
struct ServiceHealthChecker {
    name: String,
}

impl ServiceHealthChecker {
    fn new(name: String) -> Self {
        Self { name }
    }
}

#[async_trait]
impl HealthChecker for ServiceHealthChecker {
    async fn check(&self) -> HealthCheck {
        HealthCheck::new(self.name.clone(), HealthStatus::Passing).with_details("服务运行正常".to_string())
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== WAE Observability 基础示例 ===\n");

    tracing::info!("初始化可观测性服务");

    let config = ObservabilityConfig::new("observability-demo".to_string(), "1.0.0".to_string()).with_health();

    let mut observability = init_observability(config)?;

    let memory_checker = MemoryHealthChecker::new("memory-check".to_string());

    observability.add_health_checker(Box::new(memory_checker.clone()));
    observability.add_health_checker(Box::new(ServiceHealthChecker::new("service-check".to_string())));

    let metrics = InMemoryMetrics::new();

    println!("=== 功能演示 ===");
    println!("1. 健康检查 - 模拟内存和服务状态检查");
    println!("2. 指标收集 - 使用内存数据结构模拟指标存储");
    println!("3. 日志记录 - 使用 tracing 进行结构化日志");
    println!();

    println!("正在执行健康检查...");
    let report = observability.health_check().await?;
    println!("健康检查结果:");
    println!("  整体状态: {:?}", report.overall);
    for check in report.checks {
        println!("  - {}: {:?} ({:?})", check.name, check.status, check.details);
    }
    println!();

    println!("正在设置指标数据...");
    metrics.set_active_users(150);
    metrics.increment_request();
    metrics.increment_request();
    println!("  请求计数: {}", metrics.get_request_count());
    println!("  活跃用户: {}", metrics.get_active_users());
    println!();

    println!("正在更新内存使用率...");
    memory_checker.set_usage(45);
    tracing::info!("内存使用率设置为 45%");
    let report = observability.health_check().await?;
    println!("更新后的健康检查:");
    println!("  整体状态: {:?}", report.overall);
    for check in report.checks {
        println!("  - {}: {:?} ({:?})", check.name, check.status, check.details);
    }
    println!();

    println!("正在模拟高内存使用率警告...");
    memory_checker.set_usage(70);
    tracing::warn!("内存使用率设置为 70% (警告阈值)");
    let report = observability.health_check().await?;
    println!("警告状态下的健康检查:");
    println!("  整体状态: {:?}", report.overall);
    for check in report.checks {
        println!("  - {}: {:?} ({:?})", check.name, check.status, check.details);
    }
    println!();

    println!("正在模拟严重内存使用率...");
    memory_checker.set_usage(90);
    tracing::error!("内存使用率设置为 90% (严重阈值)");
    let report = observability.health_check().await?;
    println!("严重状态下的健康检查:");
    println!("  整体状态: {:?}", report.overall);
    for check in report.checks {
        println!("  - {}: {:?} ({:?})", check.name, check.status, check.details);
    }
    println!();

    tracing::info!("示例演示完成");

    println!("=== 示例说明 ===");
    println!("本示例演示了 WAE Observability 模块的核心功能:");
    println!("- 自定义健康检查器的实现和集成");
    println!("- 使用内存数据结构进行指标收集");
    println!("- 结构化日志记录");
    println!("- 健康状态聚合");
    println!("- 不同健康状态（Passing/Warning/Critical）的演示");
    println!();

    Ok(())
}
