//! 熔断器模块
//!
//! 实现熔断器模式，防止级联故障，提供系统保护。

use parking_lot::Mutex;
use serde::{Deserialize, Serialize};
use std::{
    collections::VecDeque,
    sync::Arc,
    time::{Duration, Instant},
};
use tracing::{debug, warn};
use wae_types::WaeError;

/// 熔断器状态
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CircuitState {
    /// 关闭状态 - 正常运行，允许请求通过
    Closed,
    /// 开启状态 - 熔断保护，拒绝所有请求
    Open,
    /// 半开状态 - 试探性恢复，允许部分请求通过
    HalfOpen,
}

/// 熔断器配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CircuitBreakerConfig {
    /// 失败阈值 - 触发熔断的连续失败次数
    pub failure_threshold: u32,
    /// 成功阈值 - 半开状态下恢复正常所需的成功次数
    pub success_threshold: u32,
    /// 超时时间 - 熔断器从开启状态转换到半开状态的等待时间
    pub timeout: Duration,
    /// 时间窗口 - 统计失败次数的时间窗口
    pub time_window: Duration,
}

impl Default for CircuitBreakerConfig {
    fn default() -> Self {
        Self {
            failure_threshold: 5,
            success_threshold: 3,
            timeout: Duration::from_secs(30),
            time_window: Duration::from_secs(60),
        }
    }
}

impl CircuitBreakerConfig {
    /// 创建新的熔断器配置
    pub fn new() -> Self {
        Self::default()
    }

    /// 设置失败阈值
    pub fn failure_threshold(mut self, threshold: u32) -> Self {
        self.failure_threshold = threshold;
        self
    }

    /// 设置成功阈值
    pub fn success_threshold(mut self, threshold: u32) -> Self {
        self.success_threshold = threshold;
        self
    }

    /// 设置超时时间
    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// 设置时间窗口
    pub fn time_window(mut self, window: Duration) -> Self {
        self.time_window = window;
        self
    }
}

/// 熔断器内部状态
struct CircuitBreakerInner {
    /// 当前状态
    state: CircuitState,
    /// 失败计数
    failure_count: u32,
    /// 成功计数 (半开状态使用)
    success_count: u32,
    /// 最后一次失败时间
    last_failure_time: Option<Instant>,
    /// 时间窗口内的失败时间戳
    failure_timestamps: VecDeque<Instant>,
}

impl CircuitBreakerInner {
    fn new() -> Self {
        Self {
            state: CircuitState::Closed,
            failure_count: 0,
            success_count: 0,
            last_failure_time: None,
            failure_timestamps: VecDeque::new(),
        }
    }
}

/// 熔断器
///
/// 实现熔断器模式，防止级联故障，提供系统保护。
/// 支持三种状态转换：关闭 -> 开启 -> 半开 -> 关闭/开启
#[derive(Clone)]
pub struct CircuitBreaker {
    /// 熔断器名称
    name: String,
    /// 配置
    config: CircuitBreakerConfig,
    /// 内部状态
    inner: Arc<Mutex<CircuitBreakerInner>>,
}

impl CircuitBreaker {
    /// 创建新的熔断器
    pub fn new(name: impl Into<String>, config: CircuitBreakerConfig) -> Self {
        Self { name: name.into(), config, inner: Arc::new(Mutex::new(CircuitBreakerInner::new())) }
    }

    /// 使用默认配置创建熔断器
    pub fn with_name(name: impl Into<String>) -> Self {
        Self::new(name, CircuitBreakerConfig::default())
    }

    /// 获取熔断器名称
    pub fn name(&self) -> &str {
        &self.name
    }

    /// 获取当前状态
    pub fn state(&self) -> CircuitState {
        let inner = self.inner.lock();
        inner.state
    }

    /// 检查是否允许调用
    ///
    /// 返回 true 表示允许调用，false 表示拒绝调用
    pub fn is_call_allowed(&self) -> bool {
        let mut inner = self.inner.lock();

        match inner.state {
            CircuitState::Closed => true,
            CircuitState::Open => {
                if let Some(last_failure) = inner.last_failure_time
                    && last_failure.elapsed() >= self.config.timeout
                {
                    debug!(
                        circuit_breaker = %self.name,
                        "Circuit breaker transitioning from Open to HalfOpen"
                    );
                    inner.state = CircuitState::HalfOpen;
                    inner.success_count = 0;
                    return true;
                }
                false
            }
            CircuitState::HalfOpen => true,
        }
    }

    /// 记录成功
    pub fn record_success(&self) {
        let mut inner = self.inner.lock();

        match inner.state {
            CircuitState::Closed => {
                inner.failure_count = 0;
                inner.failure_timestamps.clear();
            }
            CircuitState::HalfOpen => {
                inner.success_count += 1;
                if inner.success_count >= self.config.success_threshold {
                    debug!(
                        circuit_breaker = %self.name,
                        success_count = inner.success_count,
                        "Circuit breaker transitioning from HalfOpen to Closed"
                    );
                    inner.state = CircuitState::Closed;
                    inner.failure_count = 0;
                    inner.success_count = 0;
                    inner.failure_timestamps.clear();
                }
            }
            CircuitState::Open => {}
        }
    }

    /// 记录失败
    pub fn record_failure(&self) {
        let mut inner = self.inner.lock();
        let now = Instant::now();

        inner.last_failure_time = Some(now);
        inner.failure_timestamps.push_back(now);

        while let Some(&front) = inner.failure_timestamps.front() {
            if now.duration_since(front) > self.config.time_window {
                inner.failure_timestamps.pop_front();
            }
            else {
                break;
            }
        }

        match inner.state {
            CircuitState::Closed => {
                inner.failure_count = inner.failure_timestamps.len() as u32;
                if inner.failure_count >= self.config.failure_threshold {
                    warn!(
                        circuit_breaker = %self.name,
                        failure_count = inner.failure_count,
                        "Circuit breaker transitioning from Closed to Open"
                    );
                    inner.state = CircuitState::Open;
                }
            }
            CircuitState::HalfOpen => {
                warn!(
                    circuit_breaker = %self.name,
                    "Circuit breaker transitioning from HalfOpen to Open due to failure"
                );
                inner.state = CircuitState::Open;
                inner.success_count = 0;
            }
            CircuitState::Open => {}
        }
    }

    /// 执行受保护的异步操作
    pub async fn execute<F, T, E, Fut>(&self, operation: F) -> Result<T, WaeError>
    where
        F: FnOnce() -> Fut,
        Fut: std::future::Future<Output = Result<T, E>>,
        E: std::fmt::Display,
    {
        if !self.is_call_allowed() {
            return Err(WaeError::circuit_breaker_open(&self.name));
        }

        match operation().await {
            Ok(result) => {
                self.record_success();
                Ok(result)
            }
            Err(e) => {
                self.record_failure();
                Err(WaeError::internal(e.to_string()))
            }
        }
    }

    /// 强制打开熔断器
    pub fn force_open(&self) {
        let mut inner = self.inner.lock();
        inner.state = CircuitState::Open;
        inner.last_failure_time = Some(Instant::now());
    }

    /// 强制关闭熔断器
    pub fn force_close(&self) {
        let mut inner = self.inner.lock();
        inner.state = CircuitState::Closed;
        inner.failure_count = 0;
        inner.success_count = 0;
        inner.failure_timestamps.clear();
    }

    /// 获取配置
    pub fn config(&self) -> &CircuitBreakerConfig {
        &self.config
    }

    /// 获取当前失败计数
    pub fn failure_count(&self) -> u32 {
        let inner = self.inner.lock();
        inner.failure_count
    }
}
