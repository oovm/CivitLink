//! WAE Resilience - 弹性容错模块
//!
//! 提供微服务架构中的弹性容错能力，包括熔断器、限流器、重试、超时和舱壁隔离。
//!
//! 深度融合 tokio 运行时，所有 API 都是异步优先设计。
//! 微服务架构友好，支持分布式系统的高可用性保障。

#![warn(missing_docs)]

mod bulkhead;
mod circuit_breaker;
mod pipeline;
mod rate_limiter;
mod retry;
mod timeout;

pub use wae_types::{WaeError, WaeResult};

/// 弹性容错操作结果类型
pub type ResilienceResult<T> = WaeResult<T>;

pub use circuit_breaker::{CircuitBreaker, CircuitBreakerConfig, CircuitState};

pub use rate_limiter::{RateLimiter, SlidingWindow, SlidingWindowConfig, TokenBucket, TokenBucketConfig};

pub use retry::{RetryConfig, RetryContext, RetryPolicy, retry_async, retry_async_if};

pub use timeout::{TimeoutConfig, with_timeout, with_timeout_raw};

pub use bulkhead::{Bulkhead, BulkheadConfig, BulkheadPermit};

pub use pipeline::{ResiliencePipeline, ResiliencePipelineBuilder, ResiliencePipelineConfig};
