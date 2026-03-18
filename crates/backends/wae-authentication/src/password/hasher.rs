//! 密码哈希器实现

use wae_crypto::PasswordHasher;
use zeroize::Zeroize;

use crate::password::{PasswordHashConfig, PasswordHashError, PasswordHashResult};

/// 密码哈希器
#[derive(Debug, Clone)]
pub struct PasswordHasherService {
    hasher: PasswordHasher,
}

impl PasswordHasherService {
    /// 创建新的密码哈希器
    ///
    /// # Arguments
    /// * `config` - 密码哈希配置
    pub fn new(config: PasswordHashConfig) -> Self {
        let crypto_config = wae_crypto::PasswordHasherConfig::from(&config);
        Self { hasher: PasswordHasher::new(crypto_config) }
    }

    /// 使用默认配置创建密码哈希器
    pub fn default() -> Self {
        Self::new(PasswordHashConfig::default())
    }

    /// 哈希密码
    ///
    /// # Arguments
    /// * `password` - 明文密码
    pub fn hash_password(&self, password: &str) -> PasswordHashResult<String> {
        let mut password_bytes = password.as_bytes().to_vec();

        let result = self.hasher.hash_password(password).map_err(PasswordHashError::from);

        password_bytes.zeroize();
        result
    }

    /// 验证密码
    ///
    /// # Arguments
    /// * `password` - 明文密码
    /// * `hash` - 密码哈希
    pub fn verify_password(&self, password: &str, hash: &str) -> PasswordHashResult<bool> {
        let mut password_bytes = password.as_bytes().to_vec();

        let result = self.hasher.verify_password(password, hash).map_err(PasswordHashError::from);

        password_bytes.zeroize();
        result
    }
}

impl Default for PasswordHasherService {
    fn default() -> Self {
        Self::new(PasswordHashConfig::default())
    }
}
