//! Session 配置模块
//!
//! 提供 Session 的配置选项。

use std::time::Duration;

/// Session 配置
///
/// 定义 Session 的各种配置选项。
#[derive(Debug, Clone)]
pub struct SessionConfig {
    /// Session Cookie 名称
    pub cookie_name: String,
    /// Session Cookie 域
    pub cookie_domain: Option<String>,
    /// Session Cookie 路径
    pub cookie_path: String,
    /// 是否仅 HTTPS
    pub cookie_secure: bool,
    /// 是否 HttpOnly
    pub cookie_http_only: bool,
    /// SameSite 策略
    pub cookie_same_site: SameSite,
    /// Session 过期时间
    pub ttl: Duration,
    /// Session ID 长度
    pub id_length: usize,
    /// 签名密钥
    pub secret_key: Option<String>,
    /// 是否启用签名
    pub signed: bool,
}

/// SameSite Cookie 策略
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SameSite {
    /// 严格模式：同站请求才发送 Cookie
    Strict,
    /// 宽松模式：跨站导航时发送 Cookie
    Lax,
    /// 无限制
    None,
}

impl Default for SessionConfig {
    fn default() -> Self {
        Self {
            cookie_name: "session".to_string(),
            cookie_domain: None,
            cookie_path: "/".to_string(),
            cookie_secure: false,
            cookie_http_only: true,
            cookie_same_site: SameSite::Lax,
            ttl: Duration::from_secs(3600),
            id_length: 32,
            secret_key: None,
            signed: false,
        }
    }
}

impl SessionConfig {
    /// 创建新的 Session 配置
    pub fn new() -> Self {
        Self::default()
    }

    /// 设置 Cookie 名称
    pub fn with_cookie_name(mut self, name: impl Into<String>) -> Self {
        self.cookie_name = name.into();
        self
    }

    /// 设置 Cookie 域
    pub fn with_cookie_domain(mut self, domain: impl Into<String>) -> Self {
        self.cookie_domain = Some(domain.into());
        self
    }

    /// 设置 Cookie 路径
    pub fn with_cookie_path(mut self, path: impl Into<String>) -> Self {
        self.cookie_path = path.into();
        self
    }

    /// 设置是否仅 HTTPS
    pub fn with_secure(mut self, secure: bool) -> Self {
        self.cookie_secure = secure;
        self
    }

    /// 设置是否 HttpOnly
    pub fn with_http_only(mut self, http_only: bool) -> Self {
        self.cookie_http_only = http_only;
        self
    }

    /// 设置 SameSite 策略
    pub fn with_same_site(mut self, same_site: SameSite) -> Self {
        self.cookie_same_site = same_site;
        self
    }

    /// 设置 Session 过期时间
    pub fn with_ttl(mut self, ttl: Duration) -> Self {
        self.ttl = ttl;
        self
    }

    /// 设置 Session ID 长度
    pub fn with_id_length(mut self, length: usize) -> Self {
        self.id_length = length;
        self
    }

    /// 设置签名密钥
    pub fn with_secret_key(mut self, key: impl Into<String>) -> Self {
        self.secret_key = Some(key.into());
        self.signed = true;
        self
    }

    /// 启用签名
    pub fn signed(mut self, key: impl Into<String>) -> Self {
        self.secret_key = Some(key.into());
        self.signed = true;
        self
    }

    /// 禁用签名
    pub fn unsigned(mut self) -> Self {
        self.secret_key = None;
        self.signed = false;
        self
    }

    /// 生成 Cookie 字符串
    pub fn build_cookie_header(&self, session_id: &str) -> String {
        let mut parts = vec![format!("{}={}", self.cookie_name, session_id)];

        if let Some(ref domain) = self.cookie_domain {
            parts.push(format!("Domain={}", domain));
        }

        parts.push(format!("Path={}", self.cookie_path));

        if self.cookie_secure {
            parts.push("Secure".to_string());
        }

        if self.cookie_http_only {
            parts.push("HttpOnly".to_string());
        }

        let same_site = match self.cookie_same_site {
            SameSite::Strict => "Strict",
            SameSite::Lax => "Lax",
            SameSite::None => "None",
        };
        parts.push(format!("SameSite={}", same_site));

        let max_age = self.ttl.as_secs();
        parts.push(format!("Max-Age={}", max_age));

        parts.join("; ")
    }
}
