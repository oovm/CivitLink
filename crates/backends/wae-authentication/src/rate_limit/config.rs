//! 速率限制配置

/// 速率限制键类型
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum RateLimitKey {
    /// 按 IP 地址
    Ip(String),
    /// 按用户 ID
    UserId(String),
    /// 按用户名/邮箱
    Identifier(String),
}

/// 速率限制配置
#[derive(Debug, Clone)]
pub struct RateLimitConfig {
    /// 最大请求数
    pub max_requests: u32,
    /// 时间窗口（秒）
    pub window_seconds: u64,
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self { max_requests: 5, window_seconds: 60 }
    }
}

impl RateLimitConfig {
    /// 创建新的速率限制配置
    ///
    /// # Arguments
    /// * `max_requests` - 最大请求数
    /// * `window_seconds` - 时间窗口（秒）
    pub fn new(max_requests: u32, window_seconds: u64) -> Self {
        Self { max_requests, window_seconds }
    }

    /// 设置最大请求数
    ///
    /// # Arguments
    /// * `max` - 最大请求数
    pub fn with_max_requests(mut self, max: u32) -> Self {
        self.max_requests = max;
        self
    }

    /// 设置时间窗口
    ///
    /// # Arguments
    /// * `seconds` - 时间窗口（秒）
    pub fn with_window_seconds(mut self, seconds: u64) -> Self {
        self.window_seconds = seconds;
        self
    }
}
