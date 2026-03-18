//! HTTP 测试客户端模块
//!
//! 提供用于测试 HTTP 服务的客户端工具，支持链式 API 风格调用。

use bytes::Bytes;
use http::{HeaderMap, HeaderName, HeaderValue, StatusCode};
use serde::Serialize;
use std::collections::HashMap;
use wae_types::{WaeError, WaeErrorKind, WaeResult as TestingResult};

/// HTTP 测试客户端
///
/// 用于构建 HTTP 请求的测试工具。
#[derive(Debug, Clone)]
pub struct TestClient {
    base_url: Option<String>,
    headers: HeaderMap,
    query_params: HashMap<String, String>,
}

impl TestClient {
    /// 创建新的测试客户端
    pub fn new() -> Self {
        Self { base_url: None, headers: HeaderMap::new(), query_params: HashMap::new() }
    }

    /// 设置基础 URL
    pub fn base_url(mut self, url: impl Into<String>) -> Self {
        self.base_url = Some(url.into());
        self
    }

    /// 添加请求头
    pub fn header<K, V>(mut self, key: K, value: V) -> Self
    where
        K: TryInto<HeaderName>,
        V: TryInto<HeaderValue>,
    {
        if let Ok(key) = key.try_into()
            && let Ok(value) = value.try_into()
        {
            self.headers.append(key, value);
        }
        self
    }

    /// 设置多个请求头
    pub fn headers(mut self, headers: HeaderMap) -> Self {
        self.headers.extend(headers);
        self
    }

    /// 添加查询参数
    pub fn query_param<K, V>(mut self, key: K, value: V) -> Self
    where
        K: Into<String>,
        V: Into<String>,
    {
        self.query_params.insert(key.into(), value.into());
        self
    }

    /// 设置多个查询参数
    pub fn query_params(mut self, params: HashMap<String, String>) -> Self {
        self.query_params.extend(params);
        self
    }

    /// 创建 GET 请求
    pub fn get(self, url: impl Into<String>) -> RequestBuilder {
        self.request(http::Method::GET, url)
    }

    /// 创建 POST 请求
    pub fn post(self, url: impl Into<String>) -> RequestBuilder {
        self.request(http::Method::POST, url)
    }

    /// 创建 PUT 请求
    pub fn put(self, url: impl Into<String>) -> RequestBuilder {
        self.request(http::Method::PUT, url)
    }

    /// 创建 DELETE 请求
    pub fn delete(self, url: impl Into<String>) -> RequestBuilder {
        self.request(http::Method::DELETE, url)
    }

    /// 创建 PATCH 请求
    pub fn patch(self, url: impl Into<String>) -> RequestBuilder {
        self.request(http::Method::PATCH, url)
    }

    /// 创建 HEAD 请求
    pub fn head(self, url: impl Into<String>) -> RequestBuilder {
        self.request(http::Method::HEAD, url)
    }

    /// 创建 OPTIONS 请求
    pub fn options(self, url: impl Into<String>) -> RequestBuilder {
        self.request(http::Method::OPTIONS, url)
    }

    /// 创建自定义 HTTP 方法的请求
    pub fn request<M, U>(self, method: M, url: U) -> RequestBuilder
    where
        M: Into<http::Method>,
        U: Into<String>,
    {
        RequestBuilder::new(method, url, self.base_url, self.headers, self.query_params)
    }
}

impl Default for TestClient {
    fn default() -> Self {
        Self::new()
    }
}

/// 请求构建器
///
/// 用于构建 HTTP 请求的链式构建器。
#[derive(Debug, Clone)]
pub struct RequestBuilder {
    method: http::Method,
    url: String,
    base_url: Option<String>,
    headers: HeaderMap,
    query_params: HashMap<String, String>,
    body: Option<RequestBody>,
}

/// 请求体类型
#[derive(Debug, Clone)]
enum RequestBody {
    /// JSON 请求体
    Json(Bytes),
    /// 文本请求体
    Text(String),
    /// 字节请求体
    Bytes(Bytes),
}

impl RequestBuilder {
    /// 创建新的请求构建器
    fn new<M, U>(method: M, url: U, base_url: Option<String>, headers: HeaderMap, query_params: HashMap<String, String>) -> Self
    where
        M: Into<http::Method>,
        U: Into<String>,
    {
        Self { method: method.into(), url: url.into(), base_url, headers, query_params, body: None }
    }

    /// 添加请求头
    pub fn header<K, V>(mut self, key: K, value: V) -> Self
    where
        K: TryInto<HeaderName>,
        V: TryInto<HeaderValue>,
    {
        if let Ok(key) = key.try_into()
            && let Ok(value) = value.try_into()
        {
            self.headers.append(key, value);
        }
        self
    }

    /// 设置多个请求头
    pub fn headers(mut self, headers: HeaderMap) -> Self {
        self.headers.extend(headers);
        self
    }

    /// 添加查询参数
    pub fn query_param<K, V>(mut self, key: K, value: V) -> Self
    where
        K: Into<String>,
        V: Into<String>,
    {
        self.query_params.insert(key.into(), value.into());
        self
    }

    /// 设置多个查询参数
    pub fn query_params(mut self, params: HashMap<String, String>) -> Self {
        self.query_params.extend(params);
        self
    }

    /// 设置 JSON 请求体
    pub fn json<T: Serialize>(mut self, data: &T) -> TestingResult<Self> {
        let bytes = serde_json::to_vec(data).map_err(|e| WaeError::new(WaeErrorKind::JsonError { reason: e.to_string() }))?;
        self.body = Some(RequestBody::Json(Bytes::from(bytes)));
        Ok(self)
    }

    /// 设置文本请求体
    pub fn text(mut self, data: impl Into<String>) -> Self {
        self.body = Some(RequestBody::Text(data.into()));
        self
    }

    /// 设置字节请求体
    pub fn bytes(mut self, data: impl Into<Bytes>) -> Self {
        self.body = Some(RequestBody::Bytes(data.into()));
        self
    }

    /// 构建完整 URL
    fn build_url(&self) -> String {
        let mut url_str = if let Some(ref base_url) = self.base_url {
            if self.url.starts_with("http://") || self.url.starts_with("https://") {
                self.url.clone()
            }
            else {
                format!("{}{}", base_url, self.url)
            }
        }
        else {
            self.url.clone()
        };

        if !self.query_params.is_empty() {
            let separator = if url_str.contains('?') { "&" } else { "?" };
            let params: Vec<String> = self.query_params.iter().map(|(k, v)| format!("{}={}", k, v)).collect();
            url_str.push_str(separator);
            url_str.push_str(&params.join("&"));
        }

        url_str
    }

    /// 获取 HTTP 方法
    pub fn method(&self) -> &http::Method {
        &self.method
    }

    /// 获取 URL
    pub fn url(&self) -> String {
        self.build_url()
    }

    /// 获取请求头
    pub fn get_headers(&self) -> &HeaderMap {
        &self.headers
    }

    /// 获取请求体
    pub fn body(&self) -> Option<Bytes> {
        match &self.body {
            Some(RequestBody::Json(b)) => Some(b.clone()),
            Some(RequestBody::Text(t)) => Some(Bytes::from(t.clone())),
            Some(RequestBody::Bytes(b)) => Some(b.clone()),
            None => None,
        }
    }

    /// 创建测试响应（用于单元测试）
    pub fn create_response(&self, status: StatusCode, headers: HeaderMap, body: Bytes) -> TestResponse {
        TestResponse { status, headers, body, request_method: self.method.clone(), request_url: self.build_url() }
    }
}

/// HTTP 响应
///
/// 包含 HTTP 响应的状态码、响应头和响应体。
#[derive(Debug, Clone)]
pub struct TestResponse {
    /// 响应状态码
    pub status: StatusCode,
    /// 响应头
    pub headers: HeaderMap,
    /// 响应体
    pub body: Bytes,
    /// 请求方法（用于测试追踪）
    request_method: http::Method,
    /// 请求 URL（用于测试追踪）
    request_url: String,
}

impl TestResponse {
    /// 创建新的测试响应
    pub fn new(status: StatusCode, headers: HeaderMap, body: Bytes) -> Self {
        Self { status, headers, body, request_method: http::Method::GET, request_url: String::new() }
    }

    /// 获取响应状态码
    pub fn status(&self) -> StatusCode {
        self.status
    }

    /// 检查响应状态码是否成功 (200-299)
    pub fn is_success(&self) -> bool {
        self.status.is_success()
    }

    /// 获取响应头
    pub fn headers(&self) -> &HeaderMap {
        &self.headers
    }

    /// 获取指定响应头
    pub fn header<K: TryInto<HeaderName>>(&self, key: K) -> Option<&HeaderValue> {
        if let Ok(key) = key.try_into() { self.headers.get(key) } else { None }
    }

    /// 获取响应体字节
    pub fn body(&self) -> &Bytes {
        &self.body
    }

    /// 将响应体解析为字符串
    pub fn text(&self) -> TestingResult<String> {
        String::from_utf8(self.body.to_vec())
            .map_err(|e| WaeError::new(WaeErrorKind::ParseError { type_name: "String".to_string(), reason: e.to_string() }))
    }

    /// 将响应体解析为 JSON
    pub fn json<T: serde::de::DeserializeOwned>(&self) -> TestingResult<T> {
        serde_json::from_slice(&self.body).map_err(|e| WaeError::new(WaeErrorKind::JsonError { reason: e.to_string() }))
    }

    /// 断言响应状态码
    pub fn assert_status(&self, status: StatusCode) -> TestingResult<&Self> {
        if self.status != status {
            return Err(WaeError::new(WaeErrorKind::AssertionFailed {
                message: format!("Expected status {}, got {}", status, self.status),
            }));
        }
        Ok(self)
    }

    /// 断言响应是成功状态码 (200-299)
    pub fn assert_success(&self) -> TestingResult<&Self> {
        if !self.is_success() {
            return Err(WaeError::new(WaeErrorKind::AssertionFailed {
                message: format!("Expected success status, got {}", self.status),
            }));
        }
        Ok(self)
    }

    /// 断言响应头存在
    pub fn assert_header<K: TryInto<HeaderName>>(&self, key: K) -> TestingResult<&Self> {
        if let Ok(key) = key.try_into()
            && self.headers.contains_key(&key)
        {
            return Ok(self);
        }
        Err(WaeError::new(WaeErrorKind::AssertionFailed { message: "Expected header not found".to_string() }))
    }

    /// 断言响应头值
    pub fn assert_header_eq<K, V>(&self, key: K, value: V) -> TestingResult<&Self>
    where
        K: TryInto<HeaderName>,
        V: AsRef<str>,
    {
        if let Ok(key) = key.try_into()
            && let Some(header_value) = self.headers.get(&key)
            && let Ok(hv_str) = header_value.to_str()
            && hv_str == value.as_ref()
        {
            return Ok(self);
        }
        Err(WaeError::new(WaeErrorKind::AssertionFailed { message: format!("Expected header value {}", value.as_ref()) }))
    }

    /// 断言响应体包含指定文本
    pub fn assert_body_contains(&self, text: &str) -> TestingResult<&Self> {
        let body_str = self.text()?;
        if !body_str.contains(text) {
            return Err(WaeError::new(WaeErrorKind::AssertionFailed {
                message: "Expected text not found in response body".to_string(),
            }));
        }
        Ok(self)
    }

    /// 获取请求方法（用于测试追踪）
    pub fn request_method(&self) -> &http::Method {
        &self.request_method
    }

    /// 获取请求 URL（用于测试追踪）
    pub fn request_url(&self) -> &str {
        &self.request_url
    }
}
