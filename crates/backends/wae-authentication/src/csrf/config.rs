//! CSRF 配置

/// CSRF 保护配置
#[derive(Debug, Clone)]
pub struct CsrfConfig {
    /// 令牌过期时间（秒），默认 3600 秒（1小时）
    pub token_ttl: u64,
    /// 令牌长度（字节），默认 32 字节
    pub token_length: usize,
}

impl Default for CsrfConfig {
    fn default() -> Self {
        Self { token_ttl: 3600, token_length: 32 }
    }
}

impl CsrfConfig {
    /// 创建新的 CSRF 配置
    ///
    /// # Arguments
    /// * `token_ttl` - 令牌过期时间（秒）
    /// * `token_length` - 令牌长度（字节）
    pub fn new(token_ttl: u64, token_length: usize) -> Self {
        Self { token_ttl, token_length: token_length.clamp(16, 64) }
    }

    /// 设置令牌过期时间
    ///
    /// # Arguments
    /// * `ttl` - 过期时间（秒）
    pub fn with_token_ttl(mut self, ttl: u64) -> Self {
        self.token_ttl = ttl;
        self
    }

    /// 设置令牌长度
    ///
    /// # Arguments
    /// * `length` - 令牌长度（字节）
    pub fn with_token_length(mut self, length: usize) -> Self {
        self.token_length = length.clamp(16, 64);
        self
    }
}
