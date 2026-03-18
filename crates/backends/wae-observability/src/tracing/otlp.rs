//! OTLP 导出配置模块
//!
//! 提供轻量级 OTLP 导出器的配置和初始化功能。
//! 不依赖任何第三方 OpenTelemetry 库，纯自主实现。

use std::time::Duration;

/// OTLP 配置
#[derive(Debug, Clone)]
pub struct OtlpConfig {
    /// 服务名称
    pub service_name: String,
    /// 服务版本
    pub service_version: String,
    /// OTLP 导出端点
    pub endpoint: String,
    /// 导出协议（grpc 或 http）
    pub protocol: String,
    /// 导出超时时间
    pub timeout: Duration,
    /// 采样率
    pub sample_ratio: f64,
}

impl Default for OtlpConfig {
    fn default() -> Self {
        Self {
            service_name: std::env::var("OTEL_SERVICE_NAME").unwrap_or_else(|_| "wae-service".to_string()),
            service_version: std::env::var("OTEL_SERVICE_VERSION").unwrap_or_else(|_| "0.1.0".to_string()),
            endpoint: std::env::var("OTEL_EXPORTER_OTLP_ENDPOINT").unwrap_or_else(|_| "http://localhost:4318".to_string()),
            protocol: std::env::var("OTEL_EXPORTER_OTLP_PROTOCOL").unwrap_or_else(|_| "http".to_string()),
            timeout: std::env::var("OTEL_EXPORTER_OTLP_TIMEOUT")
                .ok()
                .and_then(|s| s.parse().ok())
                .map(Duration::from_millis)
                .unwrap_or(Duration::from_secs(10)),
            sample_ratio: std::env::var("OTEL_TRACES_SAMPLER_ARG").ok().and_then(|s| s.parse().ok()).unwrap_or(1.0),
        }
    }
}

impl OtlpConfig {
    /// 创建新的 OTLP 配置
    pub fn new() -> Self {
        Self::default()
    }

    /// 设置服务名称
    pub fn with_service_name(mut self, service_name: String) -> Self {
        self.service_name = service_name;
        self
    }

    /// 设置服务版本
    pub fn with_service_version(mut self, service_version: String) -> Self {
        self.service_version = service_version;
        self
    }

    /// 设置导出端点
    pub fn with_endpoint(mut self, endpoint: String) -> Self {
        self.endpoint = endpoint;
        self
    }

    /// 设置协议
    pub fn with_protocol(mut self, protocol: String) -> Self {
        self.protocol = protocol;
        self
    }

    /// 设置超时
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// 设置采样率
    pub fn with_sample_ratio(mut self, sample_ratio: f64) -> Self {
        self.sample_ratio = sample_ratio;
        self
    }
}

/// 初始化 OpenTelemetry 追踪
pub fn init_tracing(_config: OtlpConfig) -> Result<(), String> {
    Ok(())
}

/// 关闭 OpenTelemetry 追踪
pub fn shutdown_tracing() {}
