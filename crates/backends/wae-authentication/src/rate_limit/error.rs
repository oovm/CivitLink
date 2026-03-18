//! 速率限制错误类型

use std::fmt;
use wae_types::WaeError;

/// 速率限制错误
#[derive(Debug)]
pub enum RateLimitError {
    /// 超出速率限制
    RateLimitExceeded {
        /// 剩余等待时间（秒）
        retry_after: u64,
    },
}

impl fmt::Display for RateLimitError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RateLimitError::RateLimitExceeded { retry_after } => {
                write!(f, "请求过于频繁，请在 {} 秒后重试", retry_after)
            }
        }
    }
}

impl std::error::Error for RateLimitError {}

impl From<RateLimitError> for WaeError {
    fn from(err: RateLimitError) -> Self {
        match err {
            RateLimitError::RateLimitExceeded { retry_after } => WaeError::rate_limit_exceeded(retry_after),
        }
    }
}

/// 速率限制结果类型
pub type RateLimitResult<T> = Result<T, RateLimitError>;
