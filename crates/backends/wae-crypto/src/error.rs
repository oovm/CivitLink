//! 加密错误类型

use std::fmt;
use wae_types::{WaeError, WaeErrorKind};

/// 加密错误
#[derive(Debug)]
pub enum CryptoError {
    /// Base64 编解码错误
    Base64Error(base64::DecodeError),

    /// 哈希错误
    HashError(String),

    /// HMAC 错误
    HmacError(String),

    /// 密码哈希错误
    PasswordHashError,

    /// 密码验证错误
    PasswordVerifyError,

    /// 无效的算法
    InvalidAlgorithm,

    /// 密钥错误
    KeyError,

    /// Base32 编解码错误
    Base32Error(String),

    /// 签名无效
    InvalidSignature,
}

impl fmt::Display for CryptoError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CryptoError::Base64Error(e) => write!(f, "base64 error: {e}"),
            CryptoError::HashError(e) => write!(f, "hash error: {e}"),
            CryptoError::HmacError(e) => write!(f, "hmac error: {e}"),
            CryptoError::PasswordHashError => write!(f, "password hash error"),
            CryptoError::PasswordVerifyError => write!(f, "password verify error"),
            CryptoError::InvalidAlgorithm => write!(f, "invalid algorithm"),
            CryptoError::KeyError => write!(f, "key error"),
            CryptoError::Base32Error(e) => write!(f, "base32 error: {e}"),
            CryptoError::InvalidSignature => write!(f, "invalid signature"),
        }
    }
}

impl std::error::Error for CryptoError {}

impl From<base64::DecodeError> for CryptoError {
    fn from(err: base64::DecodeError) -> Self {
        CryptoError::Base64Error(err)
    }
}

/// 加密结果类型
pub type CryptoResult<T> = Result<T, CryptoError>;

impl From<CryptoError> for WaeError {
    fn from(err: CryptoError) -> Self {
        match err {
            CryptoError::Base64Error(_) => WaeError::invalid_token("invalid base64"),
            CryptoError::HashError(reason) => WaeError::new(WaeErrorKind::HmacError { reason }),
            CryptoError::HmacError(reason) => WaeError::new(WaeErrorKind::HmacError { reason }),
            CryptoError::PasswordHashError => {
                WaeError::new(WaeErrorKind::HmacError { reason: "password hash failed".to_string() })
            }
            CryptoError::PasswordVerifyError => {
                WaeError::new(WaeErrorKind::HmacError { reason: "password verify failed".to_string() })
            }
            CryptoError::InvalidAlgorithm => WaeError::new(WaeErrorKind::InvalidAlgorithm),
            CryptoError::KeyError => WaeError::new(WaeErrorKind::KeyError),
            CryptoError::Base32Error(reason) => WaeError::new(WaeErrorKind::Base32Error { reason }),
            CryptoError::InvalidSignature => WaeError::invalid_signature(),
        }
    }
}
