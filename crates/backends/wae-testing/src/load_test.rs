//! 负载测试框架
//!
//! 提供负载测试工具，支持并发请求、延迟测量、吞吐量计算和错误率统计。

use std::{
    collections::HashMap,
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    },
    time::{Duration, Instant},
};

/// 负载测试结果
#[derive(Debug, Clone)]
pub struct LoadTestResult {
    /// 总请求数
    pub total_requests: usize,
    /// 成功请求数
    pub success_count: usize,
    /// 失败请求数
    pub error_count: usize,
    /// 错误率
    pub error_rate: f64,
    /// 总测试时间
    pub total_time: Duration,
    /// 吞吐量（请求/秒）
    pub throughput: f64,
    /// 延迟统计
    pub latency_stats: LatencyStats,
    /// 按错误类型统计的错误数
    pub error_types: HashMap<String, usize>,
}

/// 延迟统计
#[derive(Debug, Clone)]
pub struct LatencyStats {
    /// 最小延迟
    pub min: Duration,
    /// 最大延迟
    pub max: Duration,
    /// 平均延迟
    pub mean: Duration,
    /// 中位数延迟
    pub median: Duration,
    /// p50 延迟
    pub p50: Duration,
    /// p90 延迟
    pub p90: Duration,
    /// p95 延迟
    pub p95: Duration,
    /// p99 延迟
    pub p99: Duration,
    /// 标准差
    pub std_dev: Option<Duration>,
}

impl Default for LatencyStats {
    fn default() -> Self {
        Self {
            min: Duration::ZERO,
            max: Duration::ZERO,
            mean: Duration::ZERO,
            median: Duration::ZERO,
            p50: Duration::ZERO,
            p90: Duration::ZERO,
            p95: Duration::ZERO,
            p99: Duration::ZERO,
            std_dev: None,
        }
    }
}

/// 负载测试配置
#[derive(Debug, Clone)]
pub struct LoadTestConfig {
    /// 并发数
    pub concurrency: usize,
    /// 总请求数
    pub total_requests: usize,
    /// 每个工作线程的请求数
    pub requests_per_worker: usize,
    /// 预热请求数
    pub warmup_requests: usize,
    /// 请求之间的延迟
    pub request_delay: Option<Duration>,
    /// 超时时间
    pub timeout: Option<Duration>,
    /// 是否启用详细日志
    pub verbose: bool,
}

impl Default for LoadTestConfig {
    fn default() -> Self {
        Self {
            concurrency: 10,
            total_requests: 1000,
            requests_per_worker: 0,
            warmup_requests: 100,
            request_delay: None,
            timeout: None,
            verbose: false,
        }
    }
}

impl LoadTestConfig {
    /// 创建新的负载测试配置
    pub fn new() -> Self {
        Self::default()
    }

    /// 设置并发数
    pub fn concurrency(mut self, concurrency: usize) -> Self {
        self.concurrency = concurrency;
        self
    }

    /// 设置总请求数
    pub fn total_requests(mut self, total: usize) -> Self {
        self.total_requests = total;
        self
    }

    /// 设置每个工作线程的请求数
    pub fn requests_per_worker(mut self, count: usize) -> Self {
        self.requests_per_worker = count;
        self
    }

    /// 设置预热请求数
    pub fn warmup_requests(mut self, count: usize) -> Self {
        self.warmup_requests = count;
        self
    }

    /// 设置请求之间的延迟
    pub fn request_delay(mut self, delay: Duration) -> Self {
        self.request_delay = Some(delay);
        self
    }

    /// 设置超时时间
    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = Some(timeout);
        self
    }

    /// 启用详细日志
    pub fn verbose(mut self, verbose: bool) -> Self {
        self.verbose = verbose;
        self
    }
}

/// 请求结果
#[derive(Debug, Clone)]
pub struct RequestResult {
    /// 是否成功
    pub success: bool,
    /// 延迟
    pub latency: Duration,
    /// 错误信息（如果失败）
    pub error: Option<String>,
}

impl RequestResult {
    /// 创建成功的请求结果
    pub fn success(latency: Duration) -> Self {
        Self { success: true, latency, error: None }
    }

    /// 创建失败的请求结果
    pub fn error(latency: Duration, error: String) -> Self {
        Self { success: false, latency, error: Some(error) }
    }
}

/// 计算延迟统计
fn calculate_latency_stats(latencies: &[Duration]) -> LatencyStats {
    if latencies.is_empty() {
        return LatencyStats::default();
    }

    let mut sorted = latencies.to_vec();
    sorted.sort();

    let count = sorted.len();
    let total_ns: u128 = sorted.iter().map(|d| d.as_nanos()).sum();
    let mean_ns = total_ns / count as u128;

    let min = sorted[0];
    let max = sorted[count - 1];
    let mean = Duration::from_nanos(mean_ns as u64);
    let median = sorted[count / 2];

    let p50 = sorted[(count as f64 * 0.5) as usize];
    let p90_idx = ((count as f64 * 0.9) as usize).min(count - 1);
    let p95_idx = ((count as f64 * 0.95) as usize).min(count - 1);
    let p99_idx = ((count as f64 * 0.99) as usize).min(count - 1);
    let p90 = sorted[p90_idx];
    let p95 = sorted[p95_idx];
    let p99 = sorted[p99_idx];

    let std_dev = if count >= 2 {
        let mean_f64 = mean_ns as f64;
        let variance: f64 = sorted.iter().map(|d| (d.as_nanos() as f64 - mean_f64).powi(2)).sum::<f64>() / (count - 1) as f64;
        Some(Duration::from_nanos(variance.sqrt() as u64))
    }
    else {
        None
    };

    LatencyStats { min, max, mean, median, p50, p90, p95, p99, std_dev }
}

/// 简单的同步负载测试器
pub struct SyncLoadTester {
    config: LoadTestConfig,
}

impl SyncLoadTester {
    /// 创建新的同步负载测试器
    pub fn new(config: LoadTestConfig) -> Self {
        Self { config }
    }

    /// 运行同步负载测试
    pub fn run<F>(&self, request_fn: F) -> LoadTestResult
    where
        F: Fn() -> RequestResult + Send + Sync + 'static,
    {
        let start_time = Instant::now();

        let total_success = Arc::new(AtomicUsize::new(0));
        let total_error = Arc::new(AtomicUsize::new(0));
        let latencies = Arc::new(parking_lot::Mutex::new(Vec::new()));
        let error_types = Arc::new(parking_lot::Mutex::new(HashMap::new()));

        let concurrency = self.config.concurrency;
        let requests_per_worker = if self.config.requests_per_worker > 0 {
            self.config.requests_per_worker
        }
        else {
            (self.config.total_requests + concurrency - 1) / concurrency
        };

        let request_fn = Arc::new(request_fn);
        let config = self.config.clone();

        let mut handles = Vec::with_capacity(concurrency);

        for _ in 0..concurrency {
            let request_fn = Arc::clone(&request_fn);
            let total_success = Arc::clone(&total_success);
            let total_error = Arc::clone(&total_error);
            let latencies = Arc::clone(&latencies);
            let error_types = Arc::clone(&error_types);
            let config = config.clone();

            let handle = std::thread::spawn(move || {
                for i in 0..(requests_per_worker + config.warmup_requests / concurrency) {
                    let is_warmup = i < config.warmup_requests / concurrency;

                    let request_start = Instant::now();
                    let result = request_fn();
                    let latency = request_start.elapsed();

                    if !is_warmup {
                        latencies.lock().push(latency);

                        if result.success {
                            total_success.fetch_add(1, Ordering::Relaxed);
                        }
                        else {
                            total_error.fetch_add(1, Ordering::Relaxed);
                            if let Some(error) = result.error {
                                let mut errors = error_types.lock();
                                *errors.entry(error).or_insert(0) += 1;
                            }
                        }
                    }

                    if let Some(delay) = config.request_delay {
                        std::thread::sleep(delay);
                    }
                }
            });

            handles.push(handle);
        }

        for handle in handles {
            let _ = handle.join();
        }

        let total_time = start_time.elapsed();
        let total_requests = total_success.load(Ordering::Relaxed) + total_error.load(Ordering::Relaxed);
        let success_count = total_success.load(Ordering::Relaxed);
        let error_count = total_error.load(Ordering::Relaxed);
        let error_rate = if total_requests > 0 { error_count as f64 / total_requests as f64 } else { 0.0 };
        let throughput = if total_time.as_secs_f64() > 0.0 { total_requests as f64 / total_time.as_secs_f64() } else { 0.0 };

        let latency_stats = calculate_latency_stats(&latencies.lock());
        let error_types = error_types.lock().clone();

        LoadTestResult {
            total_requests,
            success_count,
            error_count,
            error_rate,
            total_time,
            throughput,
            latency_stats,
            error_types,
        }
    }
}

impl LoadTestResult {
    /// 将结果格式化为可读的字符串
    pub fn to_string(&self) -> String {
        format!(
            "Load Test Result:\n\
             Total Requests: {}\n\
             Success: {}\n\
             Errors: {}\n\
             Error Rate: {:.2}%\n\
             Total Time: {:.2}s\n\
             Throughput: {:.2} req/s\n\
             Latency:\n\
               Min: {:?}\n\
               Max: {:?}\n\
               Mean: {:?}\n\
               Median: {:?}\n\
               P50: {:?}\n\
               P90: {:?}\n\
               P95: {:?}\n\
               P99: {:?}\n\
               Std Dev: {:?}\n\
             Error Types: {:?}",
            self.total_requests,
            self.success_count,
            self.error_count,
            self.error_rate * 100.0,
            self.total_time.as_secs_f64(),
            self.throughput,
            self.latency_stats.min,
            self.latency_stats.max,
            self.latency_stats.mean,
            self.latency_stats.median,
            self.latency_stats.p50,
            self.latency_stats.p90,
            self.latency_stats.p95,
            self.latency_stats.p99,
            self.latency_stats.std_dev,
            self.error_types,
        )
    }
}
