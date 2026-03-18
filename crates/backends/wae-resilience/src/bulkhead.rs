//! 舱壁隔离模块
//!
//! 使用信号量限制并发调用数量，防止资源耗尽。

use parking_lot::Mutex;
use serde::{Deserialize, Serialize};
use std::{sync::Arc, time::Duration};
use tokio::{sync::Semaphore, time::timeout as tokio_timeout};
use wae_types::{WaeError, WaeErrorKind};

/// 舱壁配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BulkheadConfig {
    /// 最大并发调用数
    pub max_concurrent_calls: usize,
    /// 最大等待时间
    pub max_wait_duration: Duration,
}

impl Default for BulkheadConfig {
    fn default() -> Self {
        Self { max_concurrent_calls: 10, max_wait_duration: Duration::from_secs(5) }
    }
}

impl BulkheadConfig {
    /// 创建新的舱壁配置
    pub fn new(max_concurrent_calls: usize, max_wait_duration: Duration) -> Self {
        Self { max_concurrent_calls, max_wait_duration }
    }

    /// 设置最大并发调用数
    pub fn max_concurrent_calls(mut self, max: usize) -> Self {
        self.max_concurrent_calls = max;
        self
    }

    /// 设置最大等待时间
    pub fn max_wait_duration(mut self, duration: Duration) -> Self {
        self.max_wait_duration = duration;
        self
    }
}

/// 舱壁隔离
///
/// 使用信号量限制并发调用数量，防止资源耗尽。
/// 当并发数达到上限时，新请求会等待或被拒绝。
#[derive(Clone)]
pub struct Bulkhead {
    /// 配置
    config: BulkheadConfig,
    /// 信号量
    semaphore: Arc<Semaphore>,
    /// 当前并发数
    concurrent_count: Arc<Mutex<usize>>,
}

impl Bulkhead {
    /// 创建新的舱壁隔离
    pub fn new(config: BulkheadConfig) -> Self {
        Self {
            semaphore: Arc::new(Semaphore::new(config.max_concurrent_calls)),
            concurrent_count: Arc::new(Mutex::new(0)),
            config,
        }
    }

    /// 使用默认配置创建
    pub fn with_defaults() -> Self {
        Self::new(BulkheadConfig::default())
    }

    /// 尝试获取许可
    pub async fn acquire(&self) -> Result<BulkheadPermit, WaeError> {
        let acquire_result = if self.config.max_wait_duration.is_zero() {
            self.semaphore.clone().try_acquire_owned().map_err(|_| WaeError::bulkhead_full(self.config.max_concurrent_calls))
        }
        else {
            match tokio_timeout(self.config.max_wait_duration, self.semaphore.clone().acquire_owned()).await {
                Ok(Ok(permit)) => Ok(permit),
                Ok(Err(_)) => Err(WaeError::bulkhead_full(self.config.max_concurrent_calls)),
                Err(_) => Err(WaeError::new(WaeErrorKind::BulkheadWaitTimeout)),
            }
        };

        match acquire_result {
            Ok(permit) => {
                let mut count = self.concurrent_count.lock();
                *count += 1;
                Ok(BulkheadPermit { permit: Some(permit), concurrent_count: self.concurrent_count.clone() })
            }
            Err(e) => Err(e),
        }
    }

    /// 尝试获取许可 (非阻塞)
    pub fn try_acquire(&self) -> Result<BulkheadPermit, WaeError> {
        match self.semaphore.clone().try_acquire_owned() {
            Ok(permit) => {
                let mut count = self.concurrent_count.lock();
                *count += 1;
                Ok(BulkheadPermit { permit: Some(permit), concurrent_count: self.concurrent_count.clone() })
            }
            Err(_) => Err(WaeError::bulkhead_full(self.config.max_concurrent_calls)),
        }
    }

    /// 获取当前并发数
    pub fn concurrent_count(&self) -> usize {
        let count = self.concurrent_count.lock();
        *count
    }

    /// 获取最大并发数
    pub fn max_concurrent_calls(&self) -> usize {
        self.config.max_concurrent_calls
    }

    /// 获取可用许可数
    pub fn available_permits(&self) -> usize {
        self.semaphore.available_permits()
    }

    /// 执行受保护的异步操作
    pub async fn execute<F, T, E, Fut>(&self, operation: F) -> Result<T, WaeError>
    where
        F: FnOnce() -> Fut,
        Fut: std::future::Future<Output = Result<T, E>>,
        E: std::fmt::Display,
    {
        let _permit = self.acquire().await?;
        operation().await.map_err(|_e| WaeError::internal("Operation failed"))
    }

    /// 获取配置
    pub fn config(&self) -> &BulkheadConfig {
        &self.config
    }
}

/// 舱壁许可
///
/// RAII 守卫，释放时自动归还许可。
pub struct BulkheadPermit {
    permit: Option<tokio::sync::OwnedSemaphorePermit>,
    concurrent_count: Arc<Mutex<usize>>,
}

impl Drop for BulkheadPermit {
    fn drop(&mut self) {
        let mut count = self.concurrent_count.lock();
        *count = count.saturating_sub(1);
        drop(self.permit.take());
    }
}
