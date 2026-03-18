//! WAE Monitoring - 监控和告警服务
//!
//! 提供统一的监控和告警能力，支持系统资源监控、异常告警和数据可视化。
//!
//! 深度融合 tokio 运行时，所有 API 都是异步优先设计。
//! 支持 AlertManager 和 PagerDuty 两种主流告警平台。

#![warn(missing_docs)]

use serde::{Deserialize, Serialize};
use std::{collections::HashMap, time::Duration};
use systemstat::{Platform, System};
use tokio::{sync::RwLock, time};
use wae_types::WaeError;

/// 告警操作结果类型
pub type AlertResult<T> = Result<T, WaeError>;

/// 告警级别
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AlertSeverity {
    /// 信息级别
    Info,
    /// 警告级别
    Warning,
    /// 错误级别
    Error,
    /// 严重级别
    Critical,
}

impl Default for AlertSeverity {
    fn default() -> Self {
        Self::Warning
    }
}

/// 告警状态
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AlertStatus {
    /// 告警触发中
    Firing,
    /// 告警已解决
    Resolved,
}

/// 告警标签
pub type AlertLabels = HashMap<String, String>;

/// 告警注释
pub type AlertAnnotations = HashMap<String, String>;

/// 告警配置
#[derive(Debug, Clone)]
pub struct AlertConfig {
    /// 告警服务端点 URL
    pub endpoint: String,
    /// 超时时间
    pub timeout: std::time::Duration,
    /// 自定义 HTTP 头
    pub headers: HashMap<String, String>,
}

impl Default for AlertConfig {
    fn default() -> Self {
        Self { endpoint: String::new(), timeout: std::time::Duration::from_secs(30), headers: HashMap::new() }
    }
}

/// 告警消息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Alert {
    /// 告警名称/标识
    pub name: String,
    /// 告警级别
    pub severity: AlertSeverity,
    /// 告警状态
    pub status: AlertStatus,
    /// 告警标签
    pub labels: AlertLabels,
    /// 告警注释
    pub annotations: AlertAnnotations,
    /// 告警开始时间戳
    pub starts_at: Option<i64>,
    /// 告警结束时间戳
    pub ends_at: Option<i64>,
    /// 告警生成器 URL
    pub generator_url: Option<String>,
}

impl Alert {
    /// 创建新的告警
    pub fn new(name: String, severity: AlertSeverity) -> Self {
        Self {
            name,
            severity,
            status: AlertStatus::Firing,
            labels: HashMap::new(),
            annotations: HashMap::new(),
            starts_at: Some(chrono::Utc::now().timestamp()),
            ends_at: None,
            generator_url: None,
        }
    }

    /// 添加标签
    pub fn with_label(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.labels.insert(key.into(), value.into());
        self
    }

    /// 添加注释
    pub fn with_annotation(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.annotations.insert(key.into(), value.into());
        self
    }

    /// 设置告警状态
    pub fn with_status(mut self, status: AlertStatus) -> Self {
        self.status = status;
        self
    }

    /// 设置生成器 URL
    pub fn with_generator_url(mut self, url: impl Into<String>) -> Self {
        self.generator_url = Some(url.into());
        self
    }
}

/// 告警后端核心 trait
///
/// 定义统一的告警操作接口，所有告警后端都需要实现此 trait。
#[async_trait::async_trait]
pub trait AlertBackend: Send + Sync {
    /// 发送单个告警
    async fn send_alert(&self, alert: Alert) -> AlertResult<()>;

    /// 批量发送告警
    async fn send_alerts(&self, alerts: &[Alert]) -> AlertResult<()>;

    /// 解决告警
    async fn resolve_alert(&self, alert_name: &str) -> AlertResult<()>;

    /// 获取配置
    fn config(&self) -> &AlertConfig;
}

/// 告警服务
pub struct AlertService {
    backend: Box<dyn AlertBackend>,
}

impl AlertService {
    /// 从后端创建告警服务
    pub fn new(backend: Box<dyn AlertBackend>) -> Self {
        Self { backend }
    }

    /// 发送单个告警
    pub async fn send_alert(&self, alert: Alert) -> AlertResult<()> {
        self.backend.send_alert(alert).await
    }

    /// 批量发送告警
    pub async fn send_alerts(&self, alerts: &[Alert]) -> AlertResult<()> {
        self.backend.send_alerts(alerts).await
    }

    /// 解决告警
    pub async fn resolve_alert(&self, alert_name: &str) -> AlertResult<()> {
        self.backend.resolve_alert(alert_name).await
    }

    /// 获取配置
    pub fn config(&self) -> &AlertConfig {
        self.backend.config()
    }
}

/// AlertManager 后端实现
#[cfg(feature = "alertmanager")]
pub mod alertmanager {
    use super::*;
    use serde_json::json;
    use url::Url;
    use wae_request::{HttpClient, HttpClientConfig};

    /// AlertManager 配置
    #[derive(Debug, Clone)]
    pub struct AlertManagerConfig {
        /// 基础配置
        pub base: AlertConfig,
    }

    impl From<AlertConfig> for AlertManagerConfig {
        fn from(base: AlertConfig) -> Self {
            Self { base }
        }
    }

    /// AlertManager 告警后端
    pub struct AlertManagerBackend {
        config: AlertManagerConfig,
        client: HttpClient,
    }

    impl AlertManagerBackend {
        /// 创建新的 AlertManager 后端
        pub fn new(config: AlertManagerConfig) -> AlertResult<Self> {
            let http_config = HttpClientConfig {
                timeout: config.base.timeout,
                connect_timeout: std::time::Duration::from_secs(10),
                user_agent: "wae-monitoring/0.1.0".to_string(),
                max_retries: 0,
                retry_delay: std::time::Duration::from_millis(1000),
                default_headers: config.base.headers.clone(),
            };
            let client = HttpClient::new(http_config);

            Ok(Self { config, client })
        }

        fn build_alert_url(&self) -> AlertResult<Url> {
            let mut url = Url::parse(&self.config.base.endpoint)
                .map_err(|e| WaeError::invalid_argument(format!("Invalid endpoint URL: {}", e)))?;

            url.set_path("/api/v2/alerts");
            Ok(url)
        }

        fn convert_to_alertmanager_alert(&self, alert: &Alert) -> serde_json::Value {
            let mut am_alert = json!({
                "labels": alert.labels.clone(),
                "annotations": alert.annotations.clone(),
            });

            am_alert["labels"]["alertname"] = json!(alert.name);
            am_alert["labels"]["severity"] = json!(match alert.severity {
                AlertSeverity::Info => "info",
                AlertSeverity::Warning => "warning",
                AlertSeverity::Error => "error",
                AlertSeverity::Critical => "critical",
            });

            if let Some(starts_at) = alert.starts_at {
                if let Some(dt) = chrono::DateTime::from_timestamp(starts_at, 0) {
                    am_alert["startsAt"] = json!(dt.to_rfc3339());
                }
            }

            if let Some(ends_at) = alert.ends_at {
                if let Some(dt) = chrono::DateTime::from_timestamp(ends_at, 0) {
                    am_alert["endsAt"] = json!(dt.to_rfc3339());
                }
            }

            if let Some(generator_url) = &alert.generator_url {
                am_alert["generatorURL"] = json!(generator_url);
            }

            am_alert
        }
    }

    #[async_trait::async_trait]
    impl AlertBackend for AlertManagerBackend {
        async fn send_alert(&self, alert: Alert) -> AlertResult<()> {
            self.send_alerts(&[alert]).await
        }

        async fn send_alerts(&self, alerts: &[Alert]) -> AlertResult<()> {
            let url = self.build_alert_url()?;

            let am_alerts: Vec<serde_json::Value> =
                alerts.iter().map(|alert| self.convert_to_alertmanager_alert(alert)).collect();

            let response = self.client.post_json(url.as_str(), &am_alerts).await?;

            if !response.is_success() {
                let status = response.status;
                let text = response.text().unwrap_or_else(|_| "No response body".to_string());
                return Err(WaeError::internal_error(format!("AlertManager returned error status {}: {}", status, text)));
            }

            Ok(())
        }

        async fn resolve_alert(&self, alert_name: &str) -> AlertResult<()> {
            let mut alert = Alert::new(alert_name.to_string(), AlertSeverity::Warning);
            alert.status = AlertStatus::Resolved;
            alert.ends_at = Some(chrono::Utc::now().timestamp());
            self.send_alert(alert).await
        }

        fn config(&self) -> &AlertConfig {
            &self.config.base
        }
    }
}

/// PagerDuty 后端实现
#[cfg(feature = "pagerduty")]
pub mod pagerduty {
    use super::*;
    use serde_json::json;
    use url::Url;
    use wae_request::{HttpClient, HttpClientConfig};

    /// PagerDuty 配置
    #[derive(Debug, Clone)]
    pub struct PagerDutyConfig {
        /// 基础配置
        pub base: AlertConfig,
        /// 集成密钥
        pub integration_key: String,
        /// 服务 ID（可选）
        pub service_id: Option<String>,
    }

    impl PagerDutyConfig {
        /// 创建新的 PagerDuty 配置
        pub fn new(endpoint: String, integration_key: String) -> Self {
            let base = AlertConfig { endpoint, ..Default::default() };
            Self { base, integration_key, service_id: None }
        }
    }

    /// PagerDuty 告警后端
    pub struct PagerDutyBackend {
        config: PagerDutyConfig,
        client: HttpClient,
    }

    impl PagerDutyBackend {
        /// 创建新的 PagerDuty 后端
        pub fn new(config: PagerDutyConfig) -> AlertResult<Self> {
            let http_config = HttpClientConfig {
                timeout: config.base.timeout,
                connect_timeout: std::time::Duration::from_secs(10),
                user_agent: "wae-monitoring/0.1.0".to_string(),
                max_retries: 0,
                retry_delay: std::time::Duration::from_millis(1000),
                default_headers: config.base.headers.clone(),
            };
            let client = HttpClient::new(http_config);

            Ok(Self { config, client })
        }

        fn build_event_url(&self) -> AlertResult<Url> {
            let endpoint = if self.config.base.endpoint.is_empty() {
                "https://events.pagerduty.com".to_string()
            }
            else {
                self.config.base.endpoint.clone()
            };

            let mut url =
                Url::parse(&endpoint).map_err(|e| WaeError::invalid_argument(format!("Invalid endpoint URL: {}", e)))?;

            url.set_path("/v2/enqueue");
            Ok(url)
        }

        fn convert_to_pagerduty_event(&self, alert: &Alert) -> serde_json::Value {
            let event_action = match alert.status {
                AlertStatus::Firing => "trigger",
                AlertStatus::Resolved => "resolve",
            };

            let severity = match alert.severity {
                AlertSeverity::Info => "info",
                AlertSeverity::Warning => "warning",
                AlertSeverity::Error => "error",
                AlertSeverity::Critical => "critical",
            };

            let mut event = json!({
                "routing_key": self.config.integration_key,
                "event_action": event_action,
                "dedup_key": alert.name,
                "payload": {
                    "summary": alert.name.clone(),
                    "severity": severity,
                    "source": alert.generator_url.clone().unwrap_or_else(|| "wae-monitoring".to_string()),
                    "custom_details": alert.annotations.clone(),
                }
            });

            event["payload"]["class"] = json!(alert.name.clone());

            if !alert.labels.is_empty() {
                event["payload"]["group"] = json!(alert.labels.get("group").cloned().unwrap_or_else(|| "default".to_string()));
            }

            event
        }
    }

    #[async_trait::async_trait]
    impl AlertBackend for PagerDutyBackend {
        async fn send_alert(&self, alert: Alert) -> AlertResult<()> {
            let url = self.build_event_url()?;

            let pd_event = self.convert_to_pagerduty_event(&alert);

            let mut headers = std::collections::HashMap::new();
            headers.insert("Content-Type".to_string(), "application/json".to_string());

            let body = serde_json::to_vec(&pd_event).map_err(|e| WaeError::serialization_failed("JSON"))?;

            let response = self.client.post_with_headers(url.as_str(), body, headers).await?;

            if !response.is_success() {
                let status = response.status;
                let text = response.text().unwrap_or_else(|_| "No response body".to_string());
                return Err(WaeError::internal_error(format!("PagerDuty returned error status {}: {}", status, text)));
            }

            Ok(())
        }

        async fn send_alerts(&self, alerts: &[Alert]) -> AlertResult<()> {
            for alert in alerts {
                self.send_alert(alert.clone()).await?;
            }
            Ok(())
        }

        async fn resolve_alert(&self, alert_name: &str) -> AlertResult<()> {
            let mut alert = Alert::new(alert_name.to_string(), AlertSeverity::Warning);
            alert.status = AlertStatus::Resolved;
            self.send_alert(alert).await
        }

        fn config(&self) -> &AlertConfig {
            &self.config.base
        }
    }
}

/// 便捷函数：创建 AlertManager 告警服务
#[cfg(feature = "alertmanager")]
pub fn alertmanager_service(config: alertmanager::AlertManagerConfig) -> AlertResult<AlertService> {
    let backend = alertmanager::AlertManagerBackend::new(config)?;
    Ok(AlertService::new(Box::new(backend)))
}

/// 便捷函数：创建 PagerDuty 告警服务
#[cfg(feature = "pagerduty")]
pub fn pagerduty_service(config: pagerduty::PagerDutyConfig) -> AlertResult<AlertService> {
    let backend = pagerduty::PagerDutyBackend::new(config)?;
    Ok(AlertService::new(Box::new(backend)))
}

/// 系统资源监控数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemResource {
    /// 时间戳
    pub timestamp: i64,
    /// CPU 使用率
    pub cpu_usage: f32,
    /// 内存使用情况
    pub memory: MemoryUsage,
    /// 网络流量
    pub network: NetworkUsage,
    /// 磁盘使用情况
    pub disk: DiskUsage,
}

/// 内存使用情况
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryUsage {
    /// 总内存（字节）
    pub total: u64,
    /// 已使用内存（字节）
    pub used: u64,
    /// 使用率（百分比）
    pub usage_percent: f32,
}

/// 网络流量
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkUsage {
    /// 接收字节数
    pub bytes_received: u64,
    /// 发送字节数
    pub bytes_sent: u64,
}

/// 磁盘使用情况
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiskUsage {
    /// 总空间（字节）
    pub total: u64,
    /// 已使用空间（字节）
    pub used: u64,
    /// 使用率（百分比）
    pub usage_percent: f32,
}

/// 监控配置
#[derive(Debug, Clone)]
pub struct MonitorConfig {
    /// 数据采集间隔
    pub collection_interval: Duration,
    /// 数据保留时间
    pub retention_time: Duration,
    /// 告警阈值配置
    pub alert_thresholds: AlertThresholds,
}

/// 告警阈值配置
#[derive(Debug, Clone)]
pub struct AlertThresholds {
    /// CPU 使用率阈值（百分比）
    pub cpu_threshold: f32,
    /// 内存使用率阈值（百分比）
    pub memory_threshold: f32,
    /// 磁盘使用率阈值（百分比）
    pub disk_threshold: f32,
}

impl Default for MonitorConfig {
    fn default() -> Self {
        Self {
            collection_interval: Duration::from_secs(5),
            retention_time: Duration::from_hours(24),
            alert_thresholds: AlertThresholds { cpu_threshold: 80.0, memory_threshold: 85.0, disk_threshold: 90.0 },
        }
    }
}

/// 监控服务
pub struct MonitorService {
    config: MonitorConfig,
    resources: RwLock<Vec<SystemResource>>,
    alert_service: Option<AlertService>,
    system: System,
}

impl MonitorService {
    /// 创建新的监控服务
    pub fn new(config: MonitorConfig, alert_service: Option<AlertService>) -> Self {
        Self { config, resources: RwLock::new(Vec::new()), alert_service, system: System::new() }
    }

    /// 开始监控
    pub async fn start(&self) {
        let interval = self.config.collection_interval;
        let mut interval = time::interval(interval);

        loop {
            interval.tick().await;
            if let Err(e) = self.collect_and_store().await {
                tracing::error!("Failed to collect system resources: {:?}", e);
            }
        }
    }

    /// 采集并存储系统资源数据
    async fn collect_and_store(&self) -> AlertResult<()> {
        let resource = self.collect_resources().await?;

        // 存储数据
        let mut resources = self.resources.write().await;
        resources.push(resource.clone());

        // 清理过期数据
        self.cleanup_old_data(&mut resources).await;

        // 检查告警
        self.check_alerts(&resource).await;

        Ok(())
    }

    /// 采集系统资源数据
    async fn collect_resources(&self) -> AlertResult<SystemResource> {
        let sys = &self.system;
        let timestamp = chrono::Utc::now().timestamp();

        // 采集 CPU 使用率
        let cpu_usage = match sys.cpu_load_aggregate() {
            Ok(cpu) => {
                std::thread::sleep(Duration::from_millis(100));
                match cpu.done() {
                    Ok(load) => (load.user + load.system) * 100.0,
                    Err(_) => 0.0,
                }
            }
            Err(_) => 0.0,
        };

        // 采集内存使用情况
        let memory = match sys.memory() {
            Ok(mem) => MemoryUsage {
                total: mem.total.as_u64(),
                used: mem.total.as_u64() - mem.free.as_u64(),
                usage_percent: ((mem.total.as_u64() - mem.free.as_u64()) as f32 / mem.total.as_u64() as f32) * 100.0,
            },
            Err(_) => MemoryUsage { total: 0, used: 0, usage_percent: 0.0 },
        };

        // 采集网络流量
        let network = match sys.networks() {
            Ok(networks) => {
                let mut bytes_received = 0;
                let mut bytes_sent = 0;

                for (name, _) in networks {
                    if let Ok(data) = sys.network_stats(&name) {
                        bytes_received += data.rx_bytes.as_u64();
                        bytes_sent += data.tx_bytes.as_u64();
                    }
                }

                NetworkUsage { bytes_received, bytes_sent }
            }
            Err(_) => NetworkUsage { bytes_received: 0, bytes_sent: 0 },
        };

        // 采集磁盘使用情况
        let disk = match sys.mounts() {
            Ok(mounts) => {
                let mut total = 0;
                let mut used = 0;

                for mount in mounts {
                    total += mount.total.as_u64();
                    used += mount.total.as_u64() - mount.free.as_u64();
                }

                let usage_percent = if total > 0 { (used as f32 / total as f32) * 100.0 } else { 0.0 };

                DiskUsage { total, used, usage_percent }
            }
            Err(_) => DiskUsage { total: 0, used: 0, usage_percent: 0.0 },
        };

        Ok(SystemResource { timestamp, cpu_usage, memory, network, disk })
    }

    /// 清理过期数据
    async fn cleanup_old_data(&self, resources: &mut Vec<SystemResource>) {
        let cutoff_time = chrono::Utc::now().timestamp() - self.config.retention_time.as_secs() as i64;
        resources.retain(|r| r.timestamp >= cutoff_time);
    }

    /// 检查告警
    async fn check_alerts(&self, resource: &SystemResource) {
        if let Some(alert_service) = &self.alert_service {
            let thresholds = &self.config.alert_thresholds;

            // 检查 CPU 告警
            if resource.cpu_usage > thresholds.cpu_threshold {
                let alert = Alert::new("high_cpu_usage".to_string(), AlertSeverity::Warning)
                    .with_label("resource", "cpu")
                    .with_annotation("usage", format!("{:.2}%", resource.cpu_usage))
                    .with_annotation("threshold", format!("{:.2}%", thresholds.cpu_threshold));

                if let Err(e) = alert_service.send_alert(alert).await {
                    tracing::error!("Failed to send CPU alert: {:?}", e);
                }
            }

            // 检查内存告警
            if resource.memory.usage_percent > thresholds.memory_threshold {
                let alert = Alert::new("high_memory_usage".to_string(), AlertSeverity::Warning)
                    .with_label("resource", "memory")
                    .with_annotation("usage", format!("{:.2}%", resource.memory.usage_percent))
                    .with_annotation("threshold", format!("{:.2}%", thresholds.memory_threshold));

                if let Err(e) = alert_service.send_alert(alert).await {
                    tracing::error!("Failed to send memory alert: {:?}", e);
                }
            }

            // 检查磁盘告警
            if resource.disk.usage_percent > thresholds.disk_threshold {
                let alert = Alert::new("high_disk_usage".to_string(), AlertSeverity::Warning)
                    .with_label("resource", "disk")
                    .with_annotation("usage", format!("{:.2}%", resource.disk.usage_percent))
                    .with_annotation("threshold", format!("{:.2}%", thresholds.disk_threshold));

                if let Err(e) = alert_service.send_alert(alert).await {
                    tracing::error!("Failed to send disk alert: {:?}", e);
                }
            }
        }
    }

    /// 获取监控数据
    pub async fn get_resources(&self, start_time: Option<i64>, end_time: Option<i64>) -> Vec<SystemResource> {
        let resources = self.resources.read().await;

        resources
            .iter()
            .filter(|r| {
                let after_start = start_time.map(|t| r.timestamp >= t).unwrap_or(true);
                let before_end = end_time.map(|t| r.timestamp <= t).unwrap_or(true);
                after_start && before_end
            })
            .cloned()
            .collect()
    }

    /// 获取最新的监控数据
    pub async fn get_latest_resource(&self) -> Option<SystemResource> {
        let resources = self.resources.read().await;
        resources.last().cloned()
    }
}
