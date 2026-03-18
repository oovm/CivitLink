//! CSRF 错误类型

use std::fmt;
use wae_types::WaeError;

/// CSRF 错误
#[derive(Debug)]
pub enum CsrfError {
    /// 无效令牌
    InvalidToken,
    /// 令牌已过期
    TokenExpired,
    /// 令牌生成失败
    TokenGenerationFailed,
}

impl fmt::Display for CsrfError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CsrfError::InvalidToken => write!(f, "无效的 CSRF 令牌"),
            CsrfError::TokenExpired => write!(f, "CSRF 令牌已过期"),
            CsrfError::TokenGenerationFailed => write!(f, "CSRF 令牌生成失败"),
        }
    }
}

impl std::error::Error for CsrfError {}

impl From<CsrfError> for WaeError {
    fn from(err: CsrfError) -> Self {
        match err {
            CsrfError::InvalidToken => WaeError::invalid_params("csrf_token", "invalid token"),
            CsrfError::TokenExpired => WaeError::invalid_params("csrf_token", "token expired"),
            CsrfError::TokenGenerationFailed => WaeError::internal("csrf token generation failed"),
        }
    }
}

/// CSRF 结果类型
pub type CsrfResult<T> = Result<T, CsrfError>;
