//! 组合管道模块
//!
//! 组合多种弹性策略，提供一站式的容错保护。

use crate::{
    bulkhead::{Bulkhead, BulkheadConfig},
    circuit_breaker::{CircuitBreaker, CircuitBreakerConfig},
    rate_limiter::{RateLimiter, TokenBucket, TokenBucketConfig},
    retry::{RetryConfig, retry_async},
    timeout::{TimeoutConfig, with_timeout},
};
use std::sync::Arc;
use wae_types::WaeError;

/// 弹性管道配置
#[derive(Debug, Clone, Default)]
pub struct ResiliencePipelineConfig {
    /// 熔断器配置
    pub circuit_breaker: Option<CircuitBreakerConfig>,
    /// 限流器配置 (令牌桶)
    pub rate_limiter: Option<TokenBucketConfig>,
    /// 重试配置
    pub retry: Option<RetryConfig>,
    /// 超时配置
    pub timeout: Option<TimeoutConfig>,
    /// 舱壁配置
    pub bulkhead: Option<BulkheadConfig>,
}

/// 弹性管道
///
/// 组合多种弹性策略，提供一站式的容错保护。
pub struct ResiliencePipeline {
    /// 名称
    name: String,
    /// 熔断器
    circuit_breaker: Option<CircuitBreaker>,
    /// 限流器
    rate_limiter: Option<Arc<dyn RateLimiter>>,
    /// 重试配置
    retry_config: Option<RetryConfig>,
    /// 超时配置
    timeout_config: Option<TimeoutConfig>,
    /// 舱壁
    bulkhead: Option<Bulkhead>,
}

impl ResiliencePipeline {
    /// 创建新的弹性管道
    pub fn new(name: impl Into<String>, config: ResiliencePipelineConfig) -> Self {
        let name = name.into();

        Self {
            name: name.clone(),
            circuit_breaker: config
                .circuit_breaker
                .as_ref()
                .map(|c| CircuitBreaker::new(format!("{}-circuit", name), c.clone())),
            rate_limiter: config.rate_limiter.as_ref().map(|c| Arc::new(TokenBucket::new(c.clone())) as Arc<dyn RateLimiter>),
            retry_config: config.retry.clone(),
            timeout_config: config.timeout.clone(),
            bulkhead: config.bulkhead.as_ref().map(|c| Bulkhead::new(c.clone())),
        }
    }

    /// 执行受保护的异步操作
    pub async fn execute<F, T, E, Fut>(&self, operation: F) -> Result<T, WaeError>
    where
        F: FnOnce() -> Fut + Clone,
        Fut: std::future::Future<Output = Result<T, E>>,
        E: std::fmt::Display + Clone,
    {
        if let Some(ref limiter) = self.rate_limiter {
            limiter.acquire().await?;
        }

        if let Some(ref cb) = self.circuit_breaker
            && !cb.is_call_allowed()
        {
            return Err(WaeError::circuit_breaker_open(&self.name));
        }

        let _permit = if let Some(ref bh) = self.bulkhead { Some(bh.acquire().await?) } else { None };

        let retry_config = self.retry_config.clone().unwrap_or_default();
        let timeout_duration = self.timeout_config.as_ref().map(|t| t.operation_timeout);

        let result = retry_async(&retry_config, |_ctx| {
            let op = operation.clone();
            async move {
                if let Some(duration) = timeout_duration {
                    with_timeout(duration, op()).await
                }
                else {
                    op().await.map_err(|e| WaeError::internal(e.to_string()))
                }
            }
        })
        .await;

        match &result {
            Ok(_) => {
                if let Some(ref cb) = self.circuit_breaker {
                    cb.record_success();
                }
            }
            Err(_) => {
                if let Some(ref cb) = self.circuit_breaker {
                    cb.record_failure();
                }
            }
        }

        result
    }

    /// 获取熔断器
    pub fn circuit_breaker(&self) -> Option<&CircuitBreaker> {
        self.circuit_breaker.as_ref()
    }

    /// 获取舱壁
    pub fn bulkhead(&self) -> Option<&Bulkhead> {
        self.bulkhead.as_ref()
    }

    /// 获取名称
    pub fn name(&self) -> &str {
        &self.name
    }
}

/// 弹性管道构建器
pub struct ResiliencePipelineBuilder {
    name: String,
    config: ResiliencePipelineConfig,
}

impl ResiliencePipelineBuilder {
    /// 创建新的构建器
    pub fn new(name: impl Into<String>) -> Self {
        Self { name: name.into(), config: ResiliencePipelineConfig::default() }
    }

    /// 添加熔断器
    pub fn circuit_breaker(mut self, config: CircuitBreakerConfig) -> Self {
        self.config.circuit_breaker = Some(config);
        self
    }

    /// 添加限流器
    pub fn rate_limiter(mut self, config: TokenBucketConfig) -> Self {
        self.config.rate_limiter = Some(config);
        self
    }

    /// 添加重试
    pub fn retry(mut self, config: RetryConfig) -> Self {
        self.config.retry = Some(config);
        self
    }

    /// 添加超时
    pub fn timeout(mut self, config: TimeoutConfig) -> Self {
        self.config.timeout = Some(config);
        self
    }

    /// 添加舱壁
    pub fn bulkhead(mut self, config: BulkheadConfig) -> Self {
        self.config.bulkhead = Some(config);
        self
    }

    /// 构建弹性管道
    pub fn build(self) -> ResiliencePipeline {
        ResiliencePipeline::new(self.name, self.config)
    }
}
