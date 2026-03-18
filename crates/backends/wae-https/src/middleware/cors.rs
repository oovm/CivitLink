#![doc = include_str!("readme.md")]

use std::time::Duration;

use http::{HeaderName, HeaderValue, Method, header};
use tower_http::cors::{AllowHeaders, AllowMethods, AllowOrigin, CorsLayer};

/// CORS 配置结构体
///
/// 用于配置跨域资源共享的各项参数。
pub struct CorsConfig {
    /// 允许的源列表
    pub allowed_origins: Vec<HeaderValue>,
    /// 允许的 HTTP 方法列表
    pub allowed_methods: Vec<Method>,
    /// 允许的请求头列表
    pub allowed_headers: Vec<HeaderName>,
    /// 是否允许携带凭证
    pub allow_credentials: bool,
    /// 预检请求缓存时间（秒）
    pub max_age: u64,
}

impl CorsConfig {
    /// 创建默认的 CORS 配置
    ///
    /// 默认配置：
    /// - 允许所有源
    /// - 允许所有方法
    /// - 允许所有请求头
    /// - 不允许凭证
    /// - 预检缓存时间为 600 秒
    pub fn new() -> Self {
        Self {
            allowed_origins: Vec::new(),
            allowed_methods: Vec::new(),
            allowed_headers: Vec::new(),
            allow_credentials: false,
            max_age: 600,
        }
    }

    /// 添加允许的源
    ///
    /// # 参数
    ///
    /// * `origin` - 允许的源地址，如 "https://example.com"
    pub fn allow_origin(mut self, origin: impl Into<String>) -> Self {
        if let Ok(value) = HeaderValue::from_str(&origin.into()) {
            self.allowed_origins.push(value);
        }
        self
    }

    /// 设置允许的源列表
    ///
    /// # 参数
    ///
    /// * `origins` - 允许的源地址迭代器
    pub fn allow_origins<I, S>(mut self, origins: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.allowed_origins = origins.into_iter().filter_map(|s| HeaderValue::from_str(&s.into()).ok()).collect();
        self
    }

    /// 添加允许的 HTTP 方法
    ///
    /// # 参数
    ///
    /// * `method` - 允许的 HTTP 方法，如 "GET"、"POST"
    pub fn allow_method(mut self, method: impl Into<String>) -> Self {
        if let Ok(m) = Method::from_bytes(method.into().as_bytes()) {
            self.allowed_methods.push(m);
        }
        self
    }

    /// 设置允许的 HTTP 方法列表
    ///
    /// # 参数
    ///
    /// * `methods` - 允许的 HTTP 方法迭代器
    pub fn allow_methods<I, S>(mut self, methods: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.allowed_methods = methods.into_iter().filter_map(|m| Method::from_bytes(m.into().as_bytes()).ok()).collect();
        self
    }

    /// 添加允许的请求头
    ///
    /// # 参数
    ///
    /// * `header_name` - 允许的请求头名称，如 "Content-Type"
    pub fn allow_header(mut self, header_name: impl Into<String>) -> Self {
        if let Ok(name) = HeaderName::try_from(header_name.into()) {
            self.allowed_headers.push(name);
        }
        self
    }

    /// 设置允许的请求头列表
    ///
    /// # 参数
    ///
    /// * `headers` - 允许的请求头名称迭代器
    pub fn allow_headers<I, S>(mut self, headers: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.allowed_headers = headers.into_iter().filter_map(|h| HeaderName::try_from(h.into()).ok()).collect();
        self
    }

    /// 设置是否允许携带凭证
    ///
    /// # 参数
    ///
    /// * `allow` - 是否允许凭证
    ///
    /// # 注意
    ///
    /// 当启用凭证模式时，不能使用通配符源。
    pub fn allow_credentials(mut self, allow: bool) -> Self {
        self.allow_credentials = allow;
        self
    }

    /// 设置预检请求缓存时间
    ///
    /// # 参数
    ///
    /// * `seconds` - 缓存时间（秒）
    pub fn max_age(mut self, seconds: u64) -> Self {
        self.max_age = seconds;
        self
    }

    /// 将配置转换为 CorsLayer
    ///
    /// # 返回
    ///
    /// 返回配置好的 `CorsLayer` 实例。
    pub fn into_layer(self) -> CorsLayer {
        let mut cors = CorsLayer::new();

        cors = if self.allowed_origins.is_empty() {
            cors.allow_origin(AllowOrigin::any())
        }
        else {
            cors.allow_origin(AllowOrigin::list(self.allowed_origins))
        };

        cors = if self.allowed_methods.is_empty() {
            cors.allow_methods(AllowMethods::any())
        }
        else {
            cors.allow_methods(AllowMethods::list(self.allowed_methods))
        };

        cors = if self.allowed_headers.is_empty() {
            cors.allow_headers(AllowHeaders::any())
        }
        else {
            cors.allow_headers(AllowHeaders::list(self.allowed_headers))
        };

        cors = if self.allow_credentials { cors.allow_credentials(true) } else { cors };

        cors.max_age(Duration::from_secs(self.max_age))
    }
}

impl Default for CorsConfig {
    fn default() -> Self {
        Self::new()
    }
}

/// 创建允许所有请求的 CORS 配置
///
/// 适用于开发环境，允许所有源、方法和请求头。
pub fn cors_permissive() -> CorsLayer {
    CorsLayer::permissive()
}

/// 创建严格模式的 CORS 配置
///
/// 仅允许常见的方法和请求头，不包含凭证支持。
pub fn cors_strict() -> CorsLayer {
    CorsConfig::new()
        .allow_methods(["GET", "POST", "PUT", "DELETE", "PATCH", "OPTIONS"])
        .allow_headers([header::CONTENT_TYPE.as_str(), header::AUTHORIZATION.as_str(), header::ACCEPT.as_str()])
        .max_age(3600)
        .into_layer()
}
