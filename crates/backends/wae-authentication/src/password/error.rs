//! 密码哈希错误类型

use std::fmt;
use wae_crypto::CryptoError;
use wae_types::WaeError;

/// 密码哈希错误
#[derive(Debug)]
pub enum PasswordHashError {
    /// 无效配置
    InvalidConfig,
    /// 哈希生成失败
    HashFailed,
    /// 验证失败
    VerifyFailed,
    /// 无效哈希格式
    InvalidHashFormat,
}

impl fmt::Display for PasswordHashError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PasswordHashError::InvalidConfig => write!(f, "无效的密码哈希配置"),
            PasswordHashError::HashFailed => write!(f, "密码哈希生成失败"),
            PasswordHashError::VerifyFailed => write!(f, "密码验证失败"),
            PasswordHashError::InvalidHashFormat => write!(f, "无效的哈希格式"),
        }
    }
}

impl std::error::Error for PasswordHashError {}

impl From<PasswordHashError> for WaeError {
    fn from(err: PasswordHashError) -> Self {
        match err {
            PasswordHashError::InvalidConfig => WaeError::config_invalid("password_hash", "invalid configuration"),
            PasswordHashError::HashFailed => WaeError::internal("password hash failed"),
            PasswordHashError::VerifyFailed => WaeError::invalid_credentials(),
            PasswordHashError::InvalidHashFormat => WaeError::invalid_format("hash", "valid hash format"),
        }
    }
}

impl From<CryptoError> for PasswordHashError {
    fn from(err: CryptoError) -> Self {
        match err {
            CryptoError::PasswordHashError => PasswordHashError::HashFailed,
            CryptoError::PasswordVerifyError => PasswordHashError::VerifyFailed,
            _ => PasswordHashError::HashFailed,
        }
    }
}

/// 密码哈希结果类型
pub type PasswordHashResult<T> = Result<T, PasswordHashError>;
