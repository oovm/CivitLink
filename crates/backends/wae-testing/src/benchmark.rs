//! 性能基准测试工具
//!
//! 提供对 criterion 基准测试库的封装，简化性能测试的编写。

use std::time::Duration;

#[cfg(feature = "bench")]
use criterion::{Criterion, Throughput};

/// 简单性能计时器
pub struct SimpleTimer {
    start: std::time::Instant,
}

impl SimpleTimer {
    /// 创建新的计时器并开始计时
    pub fn start() -> Self {
        Self { start: std::time::Instant::now() }
    }

    /// 停止计时并返回经过的时间
    pub fn elapsed(&self) -> Duration {
        self.start.elapsed()
    }

    /// 重置计时器
    pub fn reset(&mut self) {
        self.start = std::time::Instant::now();
    }

    /// 停止计时并返回经过的毫秒数
    pub fn elapsed_ms(&self) -> u128 {
        self.start.elapsed().as_millis()
    }

    /// 停止计时并返回经过的微秒数
    pub fn elapsed_us(&self) -> u128 {
        self.start.elapsed().as_micros()
    }

    /// 停止计时并返回经过的纳秒数
    pub fn elapsed_ns(&self) -> u128 {
        self.start.elapsed().as_nanos()
    }
}

/// 性能统计收集器
pub struct PerformanceStats {
    samples: Vec<Duration>,
}

impl PerformanceStats {
    /// 创建新的性能统计收集器
    pub fn new() -> Self {
        Self { samples: Vec::new() }
    }

    /// 添加一个测量样本
    pub fn add_sample(&mut self, duration: Duration) {
        self.samples.push(duration);
    }

    /// 测量并添加一个函数的执行时间
    pub fn measure<F, R>(&mut self, f: F) -> R
    where
        F: FnOnce() -> R,
    {
        let start = std::time::Instant::now();
        let result = f();
        self.samples.push(start.elapsed());
        result
    }

    /// 获取样本数量
    pub fn sample_count(&self) -> usize {
        self.samples.len()
    }

    /// 获取平均执行时间
    pub fn mean(&self) -> Option<Duration> {
        if self.samples.is_empty() {
            return None;
        }
        let total: u128 = self.samples.iter().map(|d| d.as_nanos()).sum();
        Some(Duration::from_nanos((total / self.samples.len() as u128) as u64))
    }

    /// 获取最小执行时间
    pub fn min(&self) -> Option<Duration> {
        self.samples.iter().min().copied()
    }

    /// 获取最大执行时间
    pub fn max(&self) -> Option<Duration> {
        self.samples.iter().max().copied()
    }

    /// 获取中位数执行时间
    pub fn median(&self) -> Option<Duration> {
        if self.samples.is_empty() {
            return None;
        }
        let mut sorted = self.samples.clone();
        sorted.sort();
        let mid = sorted.len() / 2;
        Some(sorted[mid])
    }

    /// 获取百分位数执行时间
    pub fn percentile(&self, p: f64) -> Option<Duration> {
        if self.samples.is_empty() || p < 0.0 || p > 1.0 {
            return None;
        }
        let mut sorted = self.samples.clone();
        sorted.sort();
        let idx = (sorted.len() as f64 * p) as usize;
        let idx = idx.min(sorted.len() - 1);
        Some(sorted[idx])
    }

    /// 计算标准差
    pub fn std_dev(&self) -> Option<Duration> {
        if self.samples.len() < 2 {
            return None;
        }
        let mean = self.mean()?.as_nanos() as f64;
        let variance: f64 =
            self.samples.iter().map(|d| (d.as_nanos() as f64 - mean).powi(2)).sum::<f64>() / (self.samples.len() - 1) as f64;
        Some(Duration::from_nanos(variance.sqrt() as u64))
    }

    /// 清除所有样本
    pub fn clear(&mut self) {
        self.samples.clear();
    }
}

impl Default for PerformanceStats {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(feature = "bench")]
/// 基准测试组配置
pub struct BenchmarkConfig {
    /// 样本大小
    pub sample_size: usize,
    /// 测量时间
    pub measurement_time: Duration,
    /// 预热时间
    pub warm_up_time: Duration,
    /// 是否启用噪声滤波
    pub noise_threshold: f64,
    /// 显著性水平
    pub significance_level: f64,
}

#[cfg(feature = "bench")]
impl Default for BenchmarkConfig {
    fn default() -> Self {
        Self {
            sample_size: 100,
            measurement_time: Duration::from_secs(5),
            warm_up_time: Duration::from_secs(3),
            noise_threshold: 0.05,
            significance_level: 0.05,
        }
    }
}

#[cfg(feature = "bench")]
impl BenchmarkConfig {
    /// 创建新的基准测试配置
    pub fn new() -> Self {
        Self::default()
    }

    /// 设置样本大小
    pub fn sample_size(mut self, size: usize) -> Self {
        self.sample_size = size;
        self
    }

    /// 设置测量时间
    pub fn measurement_time(mut self, time: Duration) -> Self {
        self.measurement_time = time;
        self
    }

    /// 设置预热时间
    pub fn warm_up_time(mut self, time: Duration) -> Self {
        self.warm_up_time = time;
        self
    }

    /// 设置噪声阈值
    pub fn noise_threshold(mut self, threshold: f64) -> Self {
        self.noise_threshold = threshold;
        self
    }

    /// 设置显著性水平
    pub fn significance_level(mut self, level: f64) -> Self {
        self.significance_level = level;
        self
    }
}
