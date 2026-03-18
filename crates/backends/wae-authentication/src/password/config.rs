//! 密码哈希配置

use wae_crypto::PasswordAlgorithm;

/// 密码哈希算法类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PasswordHashAlgorithm {
    /// bcrypt 算法
    Bcrypt,
    /// Argon2 算法
    Argon2,
}

/// 密码哈希配置
#[derive(Debug, Clone)]
pub struct PasswordHashConfig {
    /// 使用的哈希算法
    pub algorithm: PasswordHashAlgorithm,
    /// bcrypt 成本因子（4-31，推荐 12）
    pub bcrypt_cost: u32,
    /// argon2 内存成本（KB）
    pub argon2_memory_cost: u32,
    /// argon2 时间成本
    pub argon2_time_cost: u32,
    /// argon2 并行度
    pub argon2_parallelism: u32,
}

impl Default for PasswordHashConfig {
    fn default() -> Self {
        Self {
            algorithm: PasswordHashAlgorithm::Bcrypt,
            bcrypt_cost: 12,
            argon2_memory_cost: 19456,
            argon2_time_cost: 2,
            argon2_parallelism: 1,
        }
    }
}

impl PasswordHashConfig {
    /// 创建新的密码哈希配置
    ///
    /// # Arguments
    /// * `algorithm` - 使用的哈希算法
    pub fn new(algorithm: PasswordHashAlgorithm) -> Self {
        Self { algorithm, ..Default::default() }
    }

    /// 设置 bcrypt 成本因子
    ///
    /// # Arguments
    /// * `cost` - 成本因子（4-31）
    pub fn with_bcrypt_cost(mut self, cost: u32) -> Self {
        self.bcrypt_cost = cost.clamp(4, 31);
        self
    }

    /// 设置 argon2 内存成本
    ///
    /// # Arguments
    /// * `cost` - 内存成本（KB）
    pub fn with_argon2_memory_cost(mut self, cost: u32) -> Self {
        self.argon2_memory_cost = cost;
        self
    }

    /// 设置 argon2 时间成本
    ///
    /// # Arguments
    /// * `cost` - 时间成本
    pub fn with_argon2_time_cost(mut self, cost: u32) -> Self {
        self.argon2_time_cost = cost;
        self
    }

    /// 设置 argon2 并行度
    ///
    /// # Arguments
    /// * `parallelism` - 并行度
    pub fn with_argon2_parallelism(mut self, parallelism: u32) -> Self {
        self.argon2_parallelism = parallelism;
        self
    }
}

impl From<PasswordHashAlgorithm> for PasswordAlgorithm {
    fn from(algo: PasswordHashAlgorithm) -> Self {
        match algo {
            PasswordHashAlgorithm::Bcrypt => PasswordAlgorithm::Bcrypt,
            PasswordHashAlgorithm::Argon2 => PasswordAlgorithm::Argon2,
        }
    }
}

impl From<&PasswordHashConfig> for wae_crypto::PasswordHasherConfig {
    fn from(config: &PasswordHashConfig) -> Self {
        wae_crypto::PasswordHasherConfig {
            algorithm: config.algorithm.into(),
            bcrypt_cost: config.bcrypt_cost,
            argon2_memory_cost: config.argon2_memory_cost,
            argon2_time_cost: config.argon2_time_cost,
            argon2_parallelism: config.argon2_parallelism,
        }
    }
}
