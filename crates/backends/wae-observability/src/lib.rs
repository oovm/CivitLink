//! WAE Observability - 可观测性服务抽象层
//!
//! 提供统一的可观测性能力抽象，包括健康检查、指标收集、日志记录和分布式追踪。
//!
//! 深度融合 tokio 运行时，所有 API 都是异步优先设计。
//! 微服务架构友好，支持 HTTP 集成、Prometheus、OpenTelemetry 等特性。

#![warn(missing_docs)]

pub mod health;

#[cfg(feature = "metrics")]
pub mod metrics;

#[cfg(feature = "otlp")]
pub mod tracing;

#[cfg(feature = "profiling")]
pub mod profiling;

#[cfg(feature = "json-log")]
pub mod logging;

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use wae_types::WaeError;

#[cfg(feature = "json-log")]
use ::tracing::Level;

#[cfg(any(feature = "metrics", feature = "health", feature = "profiling"))]
use std::sync::Arc;

/// 可观测性操作结果类型
pub type ObservabilityResult<T> = Result<T, WaeError>;

/// 健康状态枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum HealthStatus {
    /// 服务健康
    #[default]
    Passing,
    /// 服务警告
    Warning,
    /// 服务失败
    Critical,
}

/// 健康检查结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheck {
    /// 检查名称
    pub name: String,
    /// 健康状态
    pub status: HealthStatus,
    /// 检查详情
    pub details: Option<String>,
    /// 检查时间戳
    pub timestamp: i64,
}

impl HealthCheck {
    /// 创建新的健康检查
    pub fn new(name: String, status: HealthStatus) -> Self {
        Self { name, status, details: None, timestamp: chrono::Utc::now().timestamp() }
    }

    /// 设置检查详情
    pub fn with_details(mut self, details: String) -> Self {
        self.details = Some(details);
        self
    }
}

/// 健康检查聚合结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthReport {
    /// 整体健康状态
    pub overall: HealthStatus,
    /// 各个检查项
    pub checks: Vec<HealthCheck>,
}

impl HealthReport {
    /// 创建新的健康报告
    pub fn new(checks: Vec<HealthCheck>) -> Self {
        let overall = checks.iter().fold(HealthStatus::Passing, |acc, check| match (acc, check.status) {
            (HealthStatus::Critical, _) | (_, HealthStatus::Critical) => HealthStatus::Critical,
            (HealthStatus::Warning, _) | (_, HealthStatus::Warning) => HealthStatus::Warning,
            _ => HealthStatus::Passing,
        });
        Self { overall, checks }
    }
}

/// 健康检查器 trait
#[async_trait::async_trait]
pub trait HealthChecker: Send + Sync {
    /// 执行健康检查
    async fn check(&self) -> HealthCheck;
}

/// 日志输出目标
#[cfg(feature = "json-log")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum LogOutput {
    /// 输出到标准输出
    #[default]
    Stdout,
    /// 输出到标准错误
    Stderr,
    /// 输出到测试用的 writer（用于测试）
    Test,
}

/// JSON 日志配置
#[cfg(feature = "json-log")]
#[derive(Debug, Clone)]
pub struct JsonLogConfig {
    /// 日志级别
    pub level: Level,
    /// 日志输出目标
    pub output: LogOutput,
    /// 是否包含目标信息
    pub include_target: bool,
    /// 是否包含时间戳
    pub include_timestamp: bool,
}

#[cfg(feature = "json-log")]
impl Default for JsonLogConfig {
    fn default() -> Self {
        Self { level: Level::INFO, output: LogOutput::Stdout, include_target: true, include_timestamp: true }
    }
}

/// OTLP 导出配置
#[cfg(feature = "otlp")]
#[derive(Debug, Clone)]
pub struct OtlpConfig {
    /// OTLP 端点地址
    pub endpoint: String,
    /// 服务名称
    pub service_name: String,
    /// 服务版本
    pub service_version: String,
    /// 采样比率 (0.0-1.0)
    pub sample_ratio: f64,
    /// 是否启用
    pub enabled: bool,
}

#[cfg(feature = "otlp")]
impl Default for OtlpConfig {
    fn default() -> Self {
        Self {
            endpoint: "http://localhost:4317".to_string(),
            service_name: "wae-service".to_string(),
            service_version: "0.1.0".to_string(),
            sample_ratio: 1.0,
            enabled: false,
        }
    }
}

/// 可观测性服务配置
#[derive(Debug, Clone)]
pub struct ObservabilityConfig {
    /// 服务名称
    pub service_name: String,
    /// 服务版本
    pub service_version: String,
    /// 启用健康检查
    pub enable_health: bool,
    /// 启用指标收集
    pub enable_metrics: bool,
    /// 启用 JSON 日志
    pub enable_json_log: bool,
    /// 启用 OTLP 导出
    pub enable_otlp: bool,
    /// 启用性能分析
    pub enable_profiling: bool,
    /// JSON 日志配置
    #[cfg(feature = "json-log")]
    pub json_log_config: JsonLogConfig,
    /// OTLP 配置
    #[cfg(feature = "otlp")]
    pub otlp_config: OtlpConfig,
    /// Profiling 配置
    #[cfg(feature = "profiling")]
    pub profiling_config: profiling::ConsoleConfig,
}

impl Default for ObservabilityConfig {
    fn default() -> Self {
        Self {
            service_name: "wae-service".to_string(),
            service_version: "0.1.0".to_string(),
            enable_health: true,
            enable_metrics: false,
            enable_json_log: false,
            enable_otlp: false,
            enable_profiling: false,
            #[cfg(feature = "json-log")]
            json_log_config: JsonLogConfig::default(),
            #[cfg(feature = "otlp")]
            otlp_config: OtlpConfig::default(),
            #[cfg(feature = "profiling")]
            profiling_config: profiling::ConsoleConfig::default(),
        }
    }
}

impl ObservabilityConfig {
    /// 创建新的可观测性配置
    pub fn new(service_name: String, service_version: String) -> Self {
        Self { service_name, service_version, ..Default::default() }
    }

    /// 启用健康检查
    pub fn with_health(mut self) -> Self {
        self.enable_health = true;
        self
    }

    /// 启用指标收集
    #[cfg(feature = "metrics")]
    pub fn with_metrics(mut self) -> Self {
        self.enable_metrics = true;
        self
    }

    /// 启用 JSON 日志
    #[cfg(feature = "json-log")]
    pub fn with_json_log(mut self) -> Self {
        self.enable_json_log = true;
        self
    }

    /// 启用 OTLP 导出
    #[cfg(feature = "otlp")]
    pub fn with_otlp(mut self) -> Self {
        self.enable_otlp = true;
        self
    }

    /// 启用性能分析
    #[cfg(feature = "profiling")]
    pub fn with_profiling(mut self) -> Self {
        self.enable_profiling = true;
        self
    }

    /// 设置 JSON 日志配置
    #[cfg(feature = "json-log")]
    pub fn with_json_log_config(mut self, config: JsonLogConfig) -> Self {
        self.json_log_config = config;
        self
    }

    /// 设置 OTLP 配置
    #[cfg(feature = "otlp")]
    pub fn with_otlp_config(mut self, config: OtlpConfig) -> Self {
        self.otlp_config = config;
        self
    }

    /// 设置 Profiling 配置
    #[cfg(feature = "profiling")]
    pub fn with_profiling_config(mut self, config: profiling::ConsoleConfig) -> Self {
        self.profiling_config = config;
        self
    }
}

/// 可观测性服务
pub struct ObservabilityService {
    /// 服务配置
    config: ObservabilityConfig,
    /// 健康检查器列表
    health_checkers: Vec<Box<dyn HealthChecker>>,
    /// 健康检查注册器
    #[cfg(feature = "health")]
    health_registry: Option<Arc<health::HealthRegistry>>,
    /// 指标注册器
    #[cfg(feature = "metrics")]
    metrics_registry: Option<Arc<metrics::MetricsRegistry>>,
}

impl ObservabilityService {
    /// 创建新的可观测性服务
    pub fn new(config: ObservabilityConfig) -> Self {
        Self {
            config,
            health_checkers: Vec::new(),
            #[cfg(feature = "health")]
            health_registry: None,
            #[cfg(feature = "metrics")]
            metrics_registry: None,
        }
    }

    /// 添加健康检查器
    pub fn add_health_checker(&mut self, checker: Box<dyn HealthChecker>) {
        self.health_checkers.push(checker);
    }

    /// 执行健康检查并返回报告
    pub async fn health_check(&self) -> ObservabilityResult<HealthReport> {
        let mut checks = Vec::with_capacity(self.health_checkers.len());
        for checker in &self.health_checkers {
            checks.push(checker.check().await);
        }
        Ok(HealthReport::new(checks))
    }

    /// 获取服务配置
    pub fn config(&self) -> &ObservabilityConfig {
        &self.config
    }

    /// 获取健康检查注册器
    #[cfg(feature = "health")]
    pub fn health_registry(&self) -> Option<&Arc<health::HealthRegistry>> {
        self.health_registry.as_ref()
    }

    /// 获取指标注册器
    #[cfg(feature = "metrics")]
    pub fn metrics_registry(&self) -> Option<&Arc<metrics::MetricsRegistry>> {
        self.metrics_registry.as_ref()
    }
}

/// 初始化可观测性服务
///
/// 根据配置初始化所有启用的可观测性功能，包括日志、追踪、健康检查等。
///
/// # Arguments
///
/// * `config` - 可观测性配置
///
/// # Returns
///
/// 初始化后的可观测性服务实例
pub fn init_observability(config: ObservabilityConfig) -> ObservabilityResult<ObservabilityService> {
    #[cfg(feature = "json-log")]
    let _ = config.enable_json_log;

    #[cfg(feature = "otlp")]
    let _ = config.enable_otlp;

    #[cfg(feature = "profiling")]
    if config.enable_profiling {
        let console = profiling::TokioConsole::new();
        console.init(&config.profiling_config);
    }

    #[allow(unused_mut)]
    let mut service = ObservabilityService::new(config);

    #[cfg(feature = "health")]
    {
        let registry = Arc::new(health::HealthRegistry::new());
        service.health_registry = Some(registry);
    }

    #[cfg(feature = "metrics")]
    if service.config.enable_metrics {
        let registry = Arc::new(metrics::MetricsRegistry::new());
        service.metrics_registry = Some(registry);
    }

    Ok(service)
}

/// 指标标签
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricLabels {
    /// 标签集合
    labels: HashMap<String, String>,
}

impl MetricLabels {
    /// 创建新的指标标签
    pub fn new() -> Self {
        Self { labels: HashMap::new() }
    }

    /// 添加标签
    pub fn insert(&mut self, key: String, value: String) {
        self.labels.insert(key, value);
    }

    /// 获取标签
    pub fn get(&self, key: &str) -> Option<&String> {
        self.labels.get(key)
    }
}

impl Default for MetricLabels {
    fn default() -> Self {
        Self::new()
    }
}

/// 基础 HTTP 健康检查器
pub struct HttpHealthChecker {
    /// 检查名称
    name: String,
    /// 检查 URL
    url: String,
}

impl HttpHealthChecker {
    /// 创建新的 HTTP 健康检查器
    pub fn new(name: String, url: String) -> Self {
        Self { name, url }
    }
}

#[async_trait::async_trait]
impl HealthChecker for HttpHealthChecker {
    async fn check(&self) -> HealthCheck {
        HealthCheck::new(self.name.clone(), HealthStatus::Passing).with_details(format!("HTTP check for {}", self.url))
    }
}
