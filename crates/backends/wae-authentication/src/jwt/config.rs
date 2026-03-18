//! JWT 配置模块

use std::time::Duration;

/// JWT 签名算法
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JwtAlgorithm {
    /// HMAC SHA-256
    HS256,
    /// HMAC SHA-384
    HS384,
    /// HMAC SHA-512
    HS512,
    /// RSASSA-PKCS1-v1_5 SHA-256 (not implemented)
    RS256,
    /// RSASSA-PKCS1-v1_5 SHA-384 (not implemented)
    RS384,
    /// RSASSA-PKCS1-v1_5 SHA-512 (not implemented)
    RS512,
    /// ECDSA SHA-256 (not implemented)
    ES256,
    /// ECDSA SHA-384 (not implemented)
    ES384,
}

impl Default for JwtAlgorithm {
    fn default() -> Self {
        Self::HS256
    }
}

/// JWT 配置
#[derive(Debug, Clone)]
pub struct JwtConfig {
    /// 签名密钥
    pub secret: String,

    /// 签名算法
    pub algorithm: JwtAlgorithm,

    /// 签发者
    pub issuer: Option<String>,

    /// 受众
    pub audience: Option<String>,

    /// 访问令牌有效期
    pub access_token_ttl: Duration,

    /// 刷新令牌有效期
    pub refresh_token_ttl: Duration,

    /// 是否验证签发者
    pub validate_issuer: bool,

    /// 是否验证受众
    pub validate_audience: bool,

    /// 时钟偏移容忍度（秒）
    pub leeway_seconds: i64,
}

impl JwtConfig {
    /// 创建新的 JWT 配置
    ///
    /// # Arguments
    /// * `secret` - 签名密钥
    pub fn new(secret: impl Into<String>) -> Self {
        Self {
            secret: secret.into(),
            algorithm: JwtAlgorithm::HS256,
            issuer: None,
            audience: None,
            access_token_ttl: Duration::from_secs(3600),
            refresh_token_ttl: Duration::from_secs(86400 * 7),
            validate_issuer: false,
            validate_audience: false,
            leeway_seconds: 60,
        }
    }

    /// 设置签名算法
    pub fn with_algorithm(mut self, algorithm: JwtAlgorithm) -> Self {
        self.algorithm = algorithm;
        self
    }

    /// 设置签发者
    pub fn with_issuer(mut self, issuer: impl Into<String>) -> Self {
        self.issuer = Some(issuer.into());
        self.validate_issuer = true;
        self
    }

    /// 设置受众
    pub fn with_audience(mut self, audience: impl Into<String>) -> Self {
        self.audience = Some(audience.into());
        self.validate_audience = true;
        self
    }

    /// 设置访问令牌有效期
    pub fn with_access_token_ttl(mut self, ttl: Duration) -> Self {
        self.access_token_ttl = ttl;
        self
    }

    /// 设置刷新令牌有效期
    pub fn with_refresh_token_ttl(mut self, ttl: Duration) -> Self {
        self.refresh_token_ttl = ttl;
        self
    }

    /// 设置时钟偏移容忍度
    pub fn with_leeway(mut self, seconds: i64) -> Self {
        self.leeway_seconds = seconds;
        self
    }

    /// 获取访问令牌过期时间（秒）
    pub fn access_token_expires_in(&self) -> i64 {
        self.access_token_ttl.as_secs() as i64
    }

    /// 获取刷新令牌过期时间（秒）
    pub fn refresh_token_expires_in(&self) -> i64 {
        self.refresh_token_ttl.as_secs() as i64
    }
}

impl Default for JwtConfig {
    fn default() -> Self {
        Self::new("default-secret-key-please-change-in-production")
    }
}

/// JWT 验证选项
#[derive(Debug, Clone)]
pub struct JwtValidation {
    /// 是否验证签名
    pub validate_signature: bool,

    /// 是否验证过期时间
    pub validate_exp: bool,

    /// 是否验证生效时间
    pub validate_nbf: bool,

    /// 是否验证签发者
    pub validate_iss: bool,

    /// 是否验证受众
    pub validate_aud: bool,

    /// 时钟偏移容忍度
    pub leeway: i64,
}

impl Default for JwtValidation {
    fn default() -> Self {
        Self {
            validate_signature: true,
            validate_exp: true,
            validate_nbf: true,
            validate_iss: false,
            validate_aud: false,
            leeway: 60,
        }
    }
}

impl JwtValidation {
    /// 创建新的验证选项
    pub fn new() -> Self {
        Self::default()
    }

    /// 设置是否验证签发者
    pub fn with_issuer_validation(mut self, validate: bool) -> Self {
        self.validate_iss = validate;
        self
    }

    /// 设置是否验证受众
    pub fn with_audience_validation(mut self, validate: bool) -> Self {
        self.validate_aud = validate;
        self
    }

    /// 设置时钟偏移容忍度
    pub fn with_leeway(mut self, seconds: i64) -> Self {
        self.leeway = seconds;
        self
    }
}
