//! 超时控制模块
//!
//! 为异步操作提供超时保护。

use serde::{Deserialize, Serialize};
use std::time::Duration;
use tokio::time::timeout as tokio_timeout;
use wae_types::{WaeError, WaeResult};

/// 超时配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeoutConfig {
    /// 全局超时时间
    pub global_timeout: Duration,
    /// 操作超时时间
    pub operation_timeout: Duration,
}

impl Default for TimeoutConfig {
    fn default() -> Self {
        Self { global_timeout: Duration::from_secs(30), operation_timeout: Duration::from_secs(10) }
    }
}

impl TimeoutConfig {
    /// 创建新的超时配置
    pub fn new(global_timeout: Duration, operation_timeout: Duration) -> Self {
        Self { global_timeout, operation_timeout }
    }

    /// 设置全局超时
    pub fn global_timeout(mut self, timeout: Duration) -> Self {
        self.global_timeout = timeout;
        self
    }

    /// 设置操作超时
    pub fn operation_timeout(mut self, timeout: Duration) -> Self {
        self.operation_timeout = timeout;
        self
    }
}

/// 为异步操作添加超时
///
/// 如果操作在指定时间内未完成，返回超时错误。
pub async fn with_timeout<T, E, Fut>(duration: Duration, operation: Fut) -> WaeResult<T>
where
    Fut: std::future::Future<Output = Result<T, E>>,
    E: std::fmt::Display,
{
    match tokio_timeout(duration, operation).await {
        Ok(Ok(result)) => Ok(result),
        Ok(Err(e)) => Err(WaeError::internal(e.to_string())),
        Err(_) => Err(WaeError::operation_timeout("operation", duration.as_millis() as u64)),
    }
}

/// 为异步操作添加超时 (返回原始错误类型)
pub async fn with_timeout_raw<T, E, Fut>(duration: Duration, operation: Fut) -> Result<Result<T, E>, WaeError>
where
    Fut: std::future::Future<Output = Result<T, E>>,
{
    match tokio_timeout(duration, operation).await {
        Ok(result) => Ok(result),
        Err(_) => Err(WaeError::operation_timeout("operation", duration.as_millis() as u64)),
    }
}
