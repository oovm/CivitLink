//! Prometheus 指标注册管理
//!
//! 提供轻量级 Prometheus 指标的注册、管理和导出功能。
//! 不依赖任何第三方库，纯自主实现。

use std::{
    collections::HashMap,
    fmt::Write,
    sync::{
        Arc,
        atomic::{AtomicU64, Ordering},
    },
};

/// 指标类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MetricType {
    /// 计数器（只增不减）
    Counter,
    /// 直方图（统计分布）
    Histogram,
    /// 仪表盘（可增可减）
    Gauge,
}

/// Prometheus 指标
pub struct Metric {
    /// 指标名称
    name: String,
    /// 指标帮助信息
    help: String,
    /// 指标类型
    metric_type: MetricType,
    /// 标签到值的映射
    values: parking_lot::RwLock<HashMap<Vec<String>, MetricValue>>,
    /// 标签键列表
    label_keys: Vec<String>,
}

impl Metric {
    /// 创建新的指标
    pub fn new(name: String, help: String, metric_type: MetricType, label_keys: Vec<String>) -> Self {
        Self { name, help, metric_type, values: parking_lot::RwLock::new(HashMap::new()), label_keys }
    }

    /// 获取或创建指标值
    fn get_or_create_value(&self, label_values: &[&str]) -> MetricValue {
        let key: Vec<String> = label_values.iter().map(|s| s.to_string()).collect();

        let mut values = self.values.write();
        values
            .entry(key)
            .or_insert_with(|| match self.metric_type {
                MetricType::Counter => MetricValue::Counter(Arc::new(AtomicU64::new(0))),
                MetricType::Gauge => MetricValue::Gauge(Arc::new(AtomicU64::new(0))),
                MetricType::Histogram => MetricValue::Histogram(Arc::new(parking_lot::RwLock::new(Histogram::new(vec![
                    0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0, 10.0,
                ])))),
            })
            .clone()
    }

    /// 增加计数器
    pub fn inc(&self, label_values: &[&str]) {
        let value = self.get_or_create_value(label_values);
        if let MetricValue::Counter(counter) = value {
            counter.fetch_add(1, Ordering::Relaxed);
        }
    }

    /// 增加计数器指定值
    pub fn inc_by(&self, label_values: &[&str], value: u64) {
        let metric_value = self.get_or_create_value(label_values);
        if let MetricValue::Counter(counter) = metric_value {
            counter.fetch_add(value, Ordering::Relaxed);
        }
    }

    /// 设置仪表盘值
    pub fn set(&self, label_values: &[&str], value: u64) {
        let metric_value = self.get_or_create_value(label_values);
        if let MetricValue::Gauge(gauge) = metric_value {
            gauge.store(value, Ordering::Relaxed);
        }
    }

    /// 观察直方图值
    pub fn observe(&self, label_values: &[&str], value: f64) {
        let metric_value = self.get_or_create_value(label_values);
        if let MetricValue::Histogram(histogram) = metric_value {
            histogram.write().observe(value);
        }
    }

    /// 导出为 Prometheus 格式
    pub fn export(&self) -> String {
        let mut output = String::new();

        writeln!(&mut output, "# HELP {} {}", self.name, self.help).unwrap();
        writeln!(&mut output, "# TYPE {} {}", self.name, self.metric_type.to_str()).unwrap();

        let values = self.values.read();
        for (label_values, value) in values.iter() {
            let labels_str = if !self.label_keys.is_empty() {
                let labels: Vec<String> = self
                    .label_keys
                    .iter()
                    .zip(label_values.iter())
                    .map(|(k, v)| format!("{}=\"{}\"", k, escape_label_value(v)))
                    .collect();
                format!("{{{}}}", labels.join(","))
            }
            else {
                String::new()
            };

            value.export(&mut output, &self.name, &labels_str);
        }

        output
    }
}

/// 指标值
#[derive(Clone)]
enum MetricValue {
    Counter(Arc<AtomicU64>),
    Gauge(Arc<AtomicU64>),
    Histogram(Arc<parking_lot::RwLock<Histogram>>),
}

impl MetricValue {
    fn export(&self, output: &mut String, name: &str, labels: &str) {
        match self {
            MetricValue::Counter(counter) => {
                let value = counter.load(Ordering::Relaxed);
                writeln!(output, "{}{} {}", name, labels, value).unwrap();
            }
            MetricValue::Gauge(gauge) => {
                let value = gauge.load(Ordering::Relaxed);
                writeln!(output, "{}{} {}", name, labels, value).unwrap();
            }
            MetricValue::Histogram(histogram) => {
                let h = histogram.read();
                h.export(output, name, labels);
            }
        }
    }
}

/// 直方图
struct Histogram {
    /// 桶边界
    buckets: Vec<f64>,
    /// 每个桶的计数
    bucket_counts: Vec<AtomicU64>,
    /// 总和
    sum: AtomicU64,
    /// 总数
    count: AtomicU64,
}

impl Histogram {
    /// 创建新的直方图
    pub fn new(buckets: Vec<f64>) -> Self {
        let bucket_counts = (0..buckets.len() + 1).map(|_| AtomicU64::new(0)).collect();
        Self { buckets, bucket_counts, sum: AtomicU64::new(0), count: AtomicU64::new(0) }
    }

    /// 观察一个值
    pub fn observe(&self, value: f64) {
        let value_u64 = (value * 1000.0) as u64;

        self.sum.fetch_add(value_u64, Ordering::Relaxed);
        self.count.fetch_add(1, Ordering::Relaxed);

        let mut bucket_index = self.buckets.len();
        for (i, &bucket) in self.buckets.iter().enumerate() {
            if value <= bucket {
                bucket_index = i;
                break;
            }
        }

        for i in bucket_index..self.bucket_counts.len() {
            self.bucket_counts[i].fetch_add(1, Ordering::Relaxed);
        }
    }

    /// 导出为 Prometheus 格式
    pub fn export(&self, output: &mut String, name: &str, labels: &str) {
        let count = self.count.load(Ordering::Relaxed);
        let sum = self.sum.load(Ordering::Relaxed) as f64 / 1000.0;

        for (i, &bucket) in self.buckets.iter().enumerate() {
            let bucket_count = self.bucket_counts[i].load(Ordering::Relaxed);
            let bucket_label = if labels.is_empty() {
                format!("{{le=\"{}\"}}", bucket)
            }
            else {
                format!("{},le=\"{}\"", &labels[1..labels.len() - 1], bucket)
            };
            writeln!(output, "{}_bucket{{{}}} {}", name, bucket_label, bucket_count).unwrap();
        }

        let inf_count = self.bucket_counts.last().unwrap().load(Ordering::Relaxed);
        let inf_label = if labels.is_empty() {
            "{le=\"+Inf\"}".to_string()
        }
        else {
            format!("{},le=\"+Inf\"", &labels[1..labels.len() - 1])
        };
        writeln!(output, "{}_bucket{{{}}} {}", name, inf_label, inf_count).unwrap();
        writeln!(output, "{}_sum{} {}", name, labels, sum).unwrap();
        writeln!(output, "{}_count{} {}", name, labels, count).unwrap();
    }
}

/// 转义标签值
fn escape_label_value(value: &str) -> String {
    value.replace("\\", "\\\\").replace("\"", "\\\"").replace("\n", "\\n")
}

impl MetricType {
    #[allow(clippy::wrong_self_convention)]
    fn to_str(&self) -> &'static str {
        match self {
            MetricType::Counter => "counter",
            MetricType::Gauge => "gauge",
            MetricType::Histogram => "histogram",
        }
    }
}

/// Prometheus 指标注册器
///
/// 管理所有 Prometheus 指标，包括 HTTP 请求计数和请求延迟直方图。
pub struct MetricsRegistry {
    /// 指标集合
    metrics: parking_lot::RwLock<HashMap<String, Arc<Metric>>>,
}

impl MetricsRegistry {
    /// 创建新的指标注册器
    pub fn new() -> Self {
        let registry = Self { metrics: parking_lot::RwLock::new(HashMap::new()) };

        registry.register_counter(
            "http_requests_total".to_string(),
            "Total number of HTTP requests".to_string(),
            vec!["method".to_string(), "path".to_string(), "status".to_string()],
        );

        registry.register_histogram(
            "http_request_duration_seconds".to_string(),
            "HTTP request duration in seconds".to_string(),
            vec!["method".to_string(), "path".to_string()],
        );

        registry
    }

    /// 注册计数器
    pub fn register_counter(&self, name: String, help: String, label_keys: Vec<String>) {
        let metric = Arc::new(Metric::new(name.clone(), help, MetricType::Counter, label_keys));
        self.metrics.write().insert(name, metric);
    }

    /// 注册仪表盘
    pub fn register_gauge(&self, name: String, help: String, label_keys: Vec<String>) {
        let metric = Arc::new(Metric::new(name.clone(), help, MetricType::Gauge, label_keys));
        self.metrics.write().insert(name, metric);
    }

    /// 注册直方图
    pub fn register_histogram(&self, name: String, help: String, label_keys: Vec<String>) {
        let metric = Arc::new(Metric::new(name.clone(), help, MetricType::Histogram, label_keys));
        self.metrics.write().insert(name, metric);
    }

    /// 获取指标
    pub fn get_metric(&self, name: &str) -> Option<Arc<Metric>> {
        self.metrics.read().get(name).cloned()
    }

    /// 记录 HTTP 请求
    pub fn record_http_request(&self, method: &str, path: &str, status: &str, duration: f64) {
        if let Some(metric) = self.get_metric("http_requests_total") {
            metric.inc(&[method, path, status]);
        }
        if let Some(metric) = self.get_metric("http_request_duration_seconds") {
            metric.observe(&[method, path], duration);
        }
    }

    /// 导出所有指标为 Prometheus 格式
    pub fn export(&self) -> String {
        let mut output = String::new();
        let metrics = self.metrics.read();
        for metric in metrics.values() {
            output.push_str(&metric.export());
        }
        output
    }
}

impl Default for MetricsRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl Clone for MetricsRegistry {
    fn clone(&self) -> Self {
        Self { metrics: parking_lot::RwLock::new(self.metrics.read().clone()) }
    }
}
