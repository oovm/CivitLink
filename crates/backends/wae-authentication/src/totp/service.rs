//! TOTP 服务实现

use crate::totp::{
    HotpConfig, SecretFormat, TotpAlgorithm, TotpConfig, TotpSecret, generate_hotp, generate_totp, verify_hotp, verify_totp,
};
use std::time::{SystemTime, UNIX_EPOCH};
use wae_types::WaeError;

/// TOTP 结果类型
pub type TotpResult<T> = Result<T, WaeError>;

/// TOTP 服务
#[derive(Debug, Clone)]
pub struct TotpService {
    config: TotpConfig,
    secret: TotpSecret,
}

impl TotpService {
    /// 创建新的 TOTP 服务
    ///
    /// # Arguments
    /// * `config` - TOTP 配置
    pub fn new(config: TotpConfig) -> TotpResult<Self> {
        let secret = TotpSecret::from_base32(&config.secret)?;
        Ok(Self { config, secret })
    }

    /// 创建新的 TOTP 服务 (使用随机密钥)
    ///
    /// # Arguments
    /// * `issuer` - 发行者名称
    /// * `account_name` - 用户账号
    pub fn create(issuer: impl Into<String>, account_name: impl Into<String>) -> TotpResult<Self> {
        let secret = TotpSecret::generate_default()?;
        let config = TotpConfig::new(issuer, account_name, secret.as_base32());
        Ok(Self { config, secret })
    }

    /// 生成当前 TOTP 码
    pub fn generate_code(&self) -> TotpResult<String> {
        let timestamp = self.current_timestamp();
        generate_totp(self.secret.as_bytes(), timestamp, self.config.time_step, self.config.digits, self.config.algorithm)
    }

    /// 验证 TOTP 码
    ///
    /// # Arguments
    /// * `code` - 用户输入的验证码
    pub fn verify(&self, code: &str) -> TotpResult<bool> {
        let timestamp = self.current_timestamp();
        verify_totp(
            self.secret.as_bytes(),
            code,
            timestamp,
            self.config.time_step,
            self.config.digits,
            self.config.algorithm,
            self.config.valid_window,
        )
    }

    /// 验证 TOTP 码 (严格模式，不使用窗口)
    pub fn verify_strict(&self, code: &str) -> TotpResult<bool> {
        let timestamp = self.current_timestamp();
        verify_totp(
            self.secret.as_bytes(),
            code,
            timestamp,
            self.config.time_step,
            self.config.digits,
            self.config.algorithm,
            0,
        )
    }

    /// 获取当前步剩余秒数
    pub fn remaining_seconds(&self) -> u64 {
        let timestamp = self.current_timestamp();
        crate::totp::algorithm::get_remaining_seconds(timestamp, self.config.time_step)
    }

    /// 获取进度百分比 (0.0 - 1.0)
    pub fn progress(&self) -> f32 {
        let remaining = self.remaining_seconds() as f32;
        1.0 - (remaining / self.config.time_step as f32)
    }

    /// 获取 otpauth:// URI
    pub fn to_uri(&self) -> String {
        self.config.to_uri()
    }

    /// 获取密钥
    pub fn secret(&self) -> &TotpSecret {
        &self.secret
    }

    /// 获取格式化的密钥
    pub fn formatted_secret(&self, format: SecretFormat) -> String {
        self.secret.format(format)
    }

    /// 获取配置
    pub fn config(&self) -> &TotpConfig {
        &self.config
    }

    fn current_timestamp(&self) -> u64 {
        SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs()
    }
}

/// HOTP 服务
#[derive(Debug, Clone)]
pub struct HotpService {
    config: HotpConfig,
    secret: TotpSecret,
}

impl HotpService {
    /// 创建新的 HOTP 服务
    pub fn new(config: HotpConfig) -> TotpResult<Self> {
        let secret = TotpSecret::from_base32(&config.secret)?;
        Ok(Self { config, secret })
    }

    /// 创建新的 HOTP 服务 (使用随机密钥)
    pub fn create(issuer: impl Into<String>, account_name: impl Into<String>) -> TotpResult<Self> {
        let secret = TotpSecret::generate_default()?;
        let config = HotpConfig::new(issuer, account_name, secret.as_base32());
        Ok(Self { config, secret })
    }

    /// 生成当前计数器的 HOTP 码
    pub fn generate_code(&self) -> TotpResult<String> {
        generate_hotp(self.secret.as_bytes(), self.config.counter, self.config.digits, self.config.algorithm)
    }

    /// 生成指定计数器的 HOTP 码
    pub fn generate_code_at(&self, counter: u64) -> TotpResult<String> {
        generate_hotp(self.secret.as_bytes(), counter, self.config.digits, self.config.algorithm)
    }

    /// 验证 HOTP 码
    pub fn verify(&self, code: &str) -> TotpResult<bool> {
        verify_hotp(self.secret.as_bytes(), code, self.config.counter, self.config.digits, self.config.algorithm)
    }

    /// 验证 HOTP 码并递增计数器
    pub fn verify_and_increment(&mut self, code: &str) -> TotpResult<bool> {
        let valid = self.verify(code)?;
        if valid {
            self.config.counter += 1;
        }
        Ok(valid)
    }

    /// 获取当前计数器值
    pub fn counter(&self) -> u64 {
        self.config.counter
    }

    /// 设置计数器值
    pub fn set_counter(&mut self, counter: u64) {
        self.config.counter = counter;
    }

    /// 获取 otpauth:// URI
    pub fn to_uri(&self) -> String {
        self.config.to_uri()
    }

    /// 获取密钥
    pub fn secret(&self) -> &TotpSecret {
        &self.secret
    }

    /// 获取配置
    pub fn config(&self) -> &HotpConfig {
        &self.config
    }
}

/// 便捷函数：创建 TOTP 服务
pub fn create_totp(issuer: impl Into<String>, account_name: impl Into<String>) -> TotpResult<TotpService> {
    TotpService::create(issuer, account_name)
}

/// 便捷函数：从密钥创建 TOTP 服务
pub fn totp_from_secret(
    issuer: impl Into<String>,
    account_name: impl Into<String>,
    secret: impl Into<String>,
) -> TotpResult<TotpService> {
    let config = TotpConfig::new(issuer, account_name, secret);
    TotpService::new(config)
}

/// 便捷函数：生成 TOTP 码
pub fn generate_totp_code(secret: &str) -> TotpResult<String> {
    let secret = TotpSecret::from_base32(secret)?;
    let timestamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs();
    generate_totp(secret.as_bytes(), timestamp, 30, 6, TotpAlgorithm::default())
}

/// 便捷函数：验证 TOTP 码
pub fn verify_totp_code(secret: &str, code: &str) -> TotpResult<bool> {
    let secret = TotpSecret::from_base32(secret)?;
    let timestamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs();
    verify_totp(secret.as_bytes(), code, timestamp, 30, 6, TotpAlgorithm::default(), 1)
}
