//! 重试机制模块
//!
//! 提供灵活的重试策略，包括指数退避、固定间隔和线性退避。

use serde::{Deserialize, Serialize};
use std::time::Duration;
use tracing::debug;
use wae_types::{WaeError, WaeResult};

/// 重试策略
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RetryPolicy {
    /// 指数退避 - 每次重试间隔翻倍
    ExponentialBackoff {
        /// 初始延迟
        initial_delay: Duration,
        /// 最大延迟
        max_delay: Duration,
        /// 乘数
        multiplier: f64,
    },
    /// 固定间隔 - 每次重试间隔相同
    FixedInterval {
        /// 固定延迟
        interval: Duration,
    },
    /// 线性退避 - 每次重试间隔线性增加
    LinearBackoff {
        /// 初始延迟
        initial_delay: Duration,
        /// 增量
        increment: Duration,
        /// 最大延迟
        max_delay: Duration,
    },
}

impl RetryPolicy {
    /// 创建指数退避策略
    pub fn exponential(initial_delay: Duration, max_delay: Duration, multiplier: f64) -> Self {
        Self::ExponentialBackoff { initial_delay, max_delay, multiplier }
    }

    /// 创建固定间隔策略
    pub fn fixed(interval: Duration) -> Self {
        Self::FixedInterval { interval }
    }

    /// 创建线性退避策略
    pub fn linear(initial_delay: Duration, increment: Duration, max_delay: Duration) -> Self {
        Self::LinearBackoff { initial_delay, increment, max_delay }
    }

    /// 计算第 n 次重试的延迟时间
    pub fn delay_for_attempt(&self, attempt: u32) -> Duration {
        match self {
            Self::ExponentialBackoff { initial_delay, max_delay, multiplier } => {
                let delay_secs = initial_delay.as_secs_f64() * multiplier.powi(attempt as i32);
                let delay = Duration::from_secs_f64(delay_secs);
                delay.min(*max_delay)
            }
            Self::FixedInterval { interval } => *interval,
            Self::LinearBackoff { initial_delay, increment, max_delay } => {
                let delay = *initial_delay + *increment * attempt;
                delay.min(*max_delay)
            }
        }
    }
}

impl Default for RetryPolicy {
    fn default() -> Self {
        Self::exponential(Duration::from_millis(100), Duration::from_secs(30), 2.0)
    }
}

/// 重试配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryConfig {
    /// 最大重试次数
    pub max_retries: u32,
    /// 重试策略
    pub policy: RetryPolicy,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self { max_retries: 3, policy: RetryPolicy::default() }
    }
}

impl RetryConfig {
    /// 创建新的重试配置
    pub fn new(max_retries: u32, policy: RetryPolicy) -> Self {
        Self { max_retries, policy }
    }

    /// 设置最大重试次数
    pub fn max_retries(mut self, max: u32) -> Self {
        self.max_retries = max;
        self
    }

    /// 设置重试策略
    pub fn policy(mut self, policy: RetryPolicy) -> Self {
        self.policy = policy;
        self
    }
}

/// 重试上下文
#[derive(Debug, Clone)]
pub struct RetryContext {
    /// 当前尝试次数
    pub attempt: u32,
    /// 最大重试次数
    pub max_retries: u32,
    /// 上一次错误信息
    pub last_error: Option<String>,
}

impl RetryContext {
    /// 创建新的重试上下文
    pub fn new(max_retries: u32) -> Self {
        Self { attempt: 0, max_retries, last_error: None }
    }

    /// 是否还有重试机会
    pub fn can_retry(&self) -> bool {
        self.attempt < self.max_retries
    }
}

/// 执行带重试的异步操作
///
/// 根据配置的重试策略自动重试失败的操作。
pub async fn retry_async<F, T, E, Fut>(config: &RetryConfig, mut operation: F) -> WaeResult<T>
where
    F: FnMut(RetryContext) -> Fut,
    Fut: std::future::Future<Output = Result<T, E>>,
    E: std::fmt::Display,
{
    let mut context = RetryContext::new(config.max_retries);

    loop {
        context.attempt += 1;

        match operation(context.clone()).await {
            Ok(result) => return Ok(result),
            Err(e) => {
                context.last_error = Some(e.to_string());

                if !context.can_retry() {
                    return Err(WaeError::max_retries_exceeded(config.max_retries));
                }

                let delay = config.policy.delay_for_attempt(context.attempt - 1);
                debug!(
                    attempt = context.attempt,
                    max_retries = context.max_retries,
                    delay_ms = delay.as_millis(),
                    error = %e,
                    "Retrying operation"
                );

                tokio::time::sleep(delay).await;
            }
        }
    }
}

/// 执行带重试的异步操作 (带条件判断)
///
/// 只有当 should_retry 返回 true 时才会重试。
pub async fn retry_async_if<F, T, E, Fut, P>(config: &RetryConfig, mut operation: F, mut should_retry: P) -> WaeResult<T>
where
    F: FnMut(RetryContext) -> Fut,
    Fut: std::future::Future<Output = Result<T, E>>,
    P: FnMut(&E) -> bool,
    E: std::fmt::Display,
{
    let mut context = RetryContext::new(config.max_retries);

    loop {
        context.attempt += 1;

        match operation(context.clone()).await {
            Ok(result) => return Ok(result),
            Err(e) => {
                context.last_error = Some(e.to_string());

                if !context.can_retry() || !should_retry(&e) {
                    return Err(WaeError::max_retries_exceeded(config.max_retries));
                }

                let delay = config.policy.delay_for_attempt(context.attempt - 1);
                debug!(
                    attempt = context.attempt,
                    max_retries = context.max_retries,
                    delay_ms = delay.as_millis(),
                    error = %e,
                    "Retrying operation (conditional)"
                );

                tokio::time::sleep(delay).await;
            }
        }
    }
}
