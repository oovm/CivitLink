//! 限流器模块
//!
//! 提供令牌桶和滑动窗口两种限流算法实现。

use async_trait::async_trait;
use parking_lot::Mutex;
use serde::{Deserialize, Serialize};
use std::{
    collections::VecDeque,
    sync::Arc,
    time::{Duration, Instant},
};
use wae_types::WaeError;

/// 限流器 trait
#[async_trait]
pub trait RateLimiter: Send + Sync {
    /// 获取许可 (阻塞直到获取成功)
    async fn acquire(&self) -> Result<(), WaeError>;

    /// 尝试获取许可 (非阻塞)
    fn try_acquire(&self) -> Result<(), WaeError>;

    /// 获取当前可用许可数
    fn available_permits(&self) -> u64;
}

/// 令牌桶配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenBucketConfig {
    /// 桶容量
    pub capacity: u64,
    /// 每秒补充的令牌数
    pub refill_rate: u64,
}

impl Default for TokenBucketConfig {
    fn default() -> Self {
        Self { capacity: 100, refill_rate: 10 }
    }
}

impl TokenBucketConfig {
    /// 创建新的令牌桶配置
    pub fn new(capacity: u64, refill_rate: u64) -> Self {
        Self { capacity, refill_rate }
    }
}

/// 令牌桶限流器
///
/// 实现令牌桶算法，支持突发流量。
/// 以固定速率向桶中添加令牌，请求消耗令牌，桶空时拒绝请求。
pub struct TokenBucket {
    /// 配置
    config: TokenBucketConfig,
    /// 当前令牌数
    tokens: Arc<Mutex<(u64, Instant)>>,
}

impl TokenBucket {
    /// 创建新的令牌桶限流器
    pub fn new(config: TokenBucketConfig) -> Self {
        Self { tokens: Arc::new(Mutex::new((config.capacity, Instant::now()))), config }
    }

    /// 使用默认配置创建
    pub fn with_defaults() -> Self {
        Self::new(TokenBucketConfig::default())
    }

    /// 补充令牌
    fn refill(&self) {
        let mut tokens = self.tokens.lock();
        let now = Instant::now();
        let elapsed = now.duration_since(tokens.1);
        let tokens_to_add = (elapsed.as_secs_f64() * self.config.refill_rate as f64) as u64;

        if tokens_to_add > 0 {
            tokens.0 = (tokens.0 + tokens_to_add).min(self.config.capacity);
            tokens.1 = now;
        }
    }
}

#[async_trait]
impl RateLimiter for TokenBucket {
    async fn acquire(&self) -> Result<(), WaeError> {
        loop {
            self.refill();

            let wait_duration = {
                let mut tokens = self.tokens.lock();
                if tokens.0 > 0 {
                    tokens.0 -= 1;
                    return Ok(());
                }

                let tokens_needed = 1;
                let wait_secs = tokens_needed as f64 / self.config.refill_rate as f64;
                Duration::from_secs_f64(wait_secs)
            };

            tokio::time::sleep(wait_duration).await;
        }
    }

    fn try_acquire(&self) -> Result<(), WaeError> {
        self.refill();

        let mut tokens = self.tokens.lock();
        if tokens.0 > 0 {
            tokens.0 -= 1;
            Ok(())
        }
        else {
            Err(WaeError::rate_limit_exceeded(self.config.capacity))
        }
    }

    fn available_permits(&self) -> u64 {
        self.refill();
        let tokens = self.tokens.lock();
        tokens.0
    }
}

/// 滑动窗口配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlidingWindowConfig {
    /// 时间窗口大小
    pub window_size: Duration,
    /// 窗口内最大请求数
    pub max_requests: u64,
}

impl Default for SlidingWindowConfig {
    fn default() -> Self {
        Self { window_size: Duration::from_secs(1), max_requests: 100 }
    }
}

impl SlidingWindowConfig {
    /// 创建新的滑动窗口配置
    pub fn new(window_size: Duration, max_requests: u64) -> Self {
        Self { window_size, max_requests }
    }
}

/// 滑动窗口限流器
///
/// 实现滑动窗口算法，精确控制请求速率。
/// 记录时间窗口内的请求时间戳，超过限制时拒绝请求。
pub struct SlidingWindow {
    /// 配置
    config: SlidingWindowConfig,
    /// 请求时间戳队列
    timestamps: Arc<Mutex<VecDeque<Instant>>>,
}

impl SlidingWindow {
    /// 创建新的滑动窗口限流器
    pub fn new(config: SlidingWindowConfig) -> Self {
        Self { timestamps: Arc::new(Mutex::new(VecDeque::new())), config }
    }

    /// 使用默认配置创建
    pub fn with_defaults() -> Self {
        Self::new(SlidingWindowConfig::default())
    }

    /// 清理过期的时间戳
    fn cleanup(&self) {
        let now = Instant::now();
        let mut timestamps = self.timestamps.lock();

        while let Some(&front) = timestamps.front() {
            if now.duration_since(front) > self.config.window_size {
                timestamps.pop_front();
            }
            else {
                break;
            }
        }
    }

    /// 获取当前窗口内的请求数
    fn current_count(&self) -> u64 {
        self.cleanup();
        let timestamps = self.timestamps.lock();
        timestamps.len() as u64
    }
}

#[async_trait]
impl RateLimiter for SlidingWindow {
    async fn acquire(&self) -> Result<(), WaeError> {
        loop {
            self.cleanup();

            let wait_duration = {
                let mut timestamps = self.timestamps.lock();
                if (timestamps.len() as u64) < self.config.max_requests {
                    timestamps.push_back(Instant::now());
                    return Ok(());
                }

                if let Some(&oldest) = timestamps.front() {
                    let elapsed = Instant::now().duration_since(oldest);
                    if elapsed < self.config.window_size { self.config.window_size - elapsed } else { Duration::ZERO }
                }
                else {
                    Duration::ZERO
                }
            };

            if wait_duration.is_zero() {
                continue;
            }

            tokio::time::sleep(wait_duration).await;
        }
    }

    fn try_acquire(&self) -> Result<(), WaeError> {
        self.cleanup();

        let mut timestamps = self.timestamps.lock();
        if (timestamps.len() as u64) < self.config.max_requests {
            timestamps.push_back(Instant::now());
            Ok(())
        }
        else {
            Err(WaeError::rate_limit_exceeded(self.config.max_requests))
        }
    }

    fn available_permits(&self) -> u64 {
        let current = self.current_count();
        self.config.max_requests.saturating_sub(current)
    }
}
