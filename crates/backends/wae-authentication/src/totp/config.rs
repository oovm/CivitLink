//! TOTP 配置模块

use std::time::Duration;

/// TOTP 哈希算法
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TotpAlgorithm {
    /// SHA-1 (默认，兼容性最好)
    #[default]
    SHA1,
    /// SHA-256
    SHA256,
    /// SHA-512
    SHA512,
}

impl TotpAlgorithm {
    /// 获取算法名称
    pub fn as_str(&self) -> &'static str {
        match self {
            TotpAlgorithm::SHA1 => "SHA1",
            TotpAlgorithm::SHA256 => "SHA256",
            TotpAlgorithm::SHA512 => "SHA512",
        }
    }

    /// 获取算法 OID (用于 URI)
    pub fn oid(&self) -> &'static str {
        match self {
            TotpAlgorithm::SHA1 => "SHA1",
            TotpAlgorithm::SHA256 => "SHA256",
            TotpAlgorithm::SHA512 => "SHA512",
        }
    }
}

/// TOTP 配置
#[derive(Debug, Clone)]
pub struct TotpConfig {
    /// 发行者 (应用名称)
    pub issuer: String,

    /// 用户账号
    pub account_name: String,

    /// 密钥 (Base32 编码)
    pub secret: String,

    /// 哈希算法
    pub algorithm: TotpAlgorithm,

    /// 数字位数 (通常为 6 或 8)
    pub digits: u32,

    /// 时间步长 (秒)
    pub time_step: u64,

    /// 有效窗口 (前后允许的步数)
    pub valid_window: u32,
}

impl TotpConfig {
    /// 创建新的 TOTP 配置
    ///
    /// # Arguments
    /// * `issuer` - 发行者名称
    /// * `account_name` - 用户账号
    /// * `secret` - Base32 编码的密钥
    pub fn new(issuer: impl Into<String>, account_name: impl Into<String>, secret: impl Into<String>) -> Self {
        Self {
            issuer: issuer.into(),
            account_name: account_name.into(),
            secret: secret.into(),
            algorithm: TotpAlgorithm::default(),
            digits: 6,
            time_step: 30,
            valid_window: 1,
        }
    }

    /// 设置哈希算法
    pub fn with_algorithm(mut self, algorithm: TotpAlgorithm) -> Self {
        self.algorithm = algorithm;
        self
    }

    /// 设置数字位数
    pub fn with_digits(mut self, digits: u32) -> Self {
        self.digits = digits;
        self
    }

    /// 设置时间步长
    pub fn with_time_step(mut self, time_step: Duration) -> Self {
        self.time_step = time_step.as_secs();
        self
    }

    /// 设置有效窗口
    pub fn with_valid_window(mut self, window: u32) -> Self {
        self.valid_window = window;
        self
    }

    /// 生成 otpauth:// URI
    pub fn to_uri(&self) -> String {
        let label = wae_types::url_encode(&format!("{}:{}", self.issuer, self.account_name));
        let params = [
            ("secret", self.secret.as_str()),
            ("issuer", &self.issuer),
            ("algorithm", self.algorithm.oid()),
            ("digits", &self.digits.to_string()),
            ("period", &self.time_step.to_string()),
        ];

        let query = params.iter().map(|(k, v)| format!("{}={}", k, wae_types::url_encode(v))).collect::<Vec<_>>().join("&");

        format!("otpauth://totp/{}?{}", label, query)
    }
}

/// HOTP (HMAC-based One-Time Password) 配置
#[derive(Debug, Clone)]
pub struct HotpConfig {
    /// 发行者
    pub issuer: String,

    /// 用户账号
    pub account_name: String,

    /// 密钥 (Base32 编码)
    pub secret: String,

    /// 哈希算法
    pub algorithm: TotpAlgorithm,

    /// 数字位数
    pub digits: u32,

    /// 计数器
    pub counter: u64,
}

impl HotpConfig {
    /// 创建新的 HOTP 配置
    pub fn new(issuer: impl Into<String>, account_name: impl Into<String>, secret: impl Into<String>) -> Self {
        Self {
            issuer: issuer.into(),
            account_name: account_name.into(),
            secret: secret.into(),
            algorithm: TotpAlgorithm::default(),
            digits: 6,
            counter: 0,
        }
    }

    /// 设置计数器
    pub fn with_counter(mut self, counter: u64) -> Self {
        self.counter = counter;
        self
    }

    /// 设置数字位数
    pub fn with_digits(mut self, digits: u32) -> Self {
        self.digits = digits;
        self
    }

    /// 设置哈希算法
    pub fn with_algorithm(mut self, algorithm: TotpAlgorithm) -> Self {
        self.algorithm = algorithm;
        self
    }

    /// 生成 otpauth:// URI
    pub fn to_uri(&self) -> String {
        let label = wae_types::url_encode(&format!("{}:{}", self.issuer, self.account_name));
        let params = [
            ("secret", self.secret.as_str()),
            ("issuer", &self.issuer),
            ("algorithm", self.algorithm.oid()),
            ("digits", &self.digits.to_string()),
            ("counter", &self.counter.to_string()),
        ];

        let query = params.iter().map(|(k, v)| format!("{}={}", k, wae_types::url_encode(v))).collect::<Vec<_>>().join("&");

        format!("otpauth://hotp/{}?{}", label, query)
    }
}
