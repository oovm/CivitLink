//! WAE Request - 基于 hyper 的 HTTP 客户端
//!
//! 基于 hyper + hyper-tls 实现的 HTTP 客户端

#![warn(missing_docs)]

use bytes::Bytes;
use http_body_util::{BodyExt, Full};
use hyper::{Request, Uri};
use hyper_util::{client::legacy::Client, rt::TokioExecutor};
use serde::{Serialize, de::DeserializeOwned};
use std::{collections::HashMap, sync::Arc, time::Duration};
use tokio::time::timeout;
use tracing::{debug, error, info};
use url::Url;
use wae_types::{WaeError, WaeErrorKind, WaeResult};

/// HTTP 客户端配置
#[derive(Debug, Clone)]
pub struct HttpClientConfig {
    /// 请求超时时间
    pub timeout: Duration,
    /// 连接超时时间
    pub connect_timeout: Duration,
    /// 用户代理
    pub user_agent: String,
    /// 最大重试次数
    pub max_retries: u32,
    /// 重试间隔
    pub retry_delay: Duration,
    /// 默认请求头
    pub default_headers: HashMap<String, String>,
}

impl Default for HttpClientConfig {
    fn default() -> Self {
        Self {
            timeout: Duration::from_secs(30),
            connect_timeout: Duration::from_secs(10),
            user_agent: "wae-request/0.1.0".to_string(),
            max_retries: 3,
            retry_delay: Duration::from_millis(1000),
            default_headers: HashMap::new(),
        }
    }
}

/// HTTP 响应
#[derive(Debug)]
pub struct HttpResponse {
    /// HTTP 版本
    pub version: String,
    /// 状态码
    pub status: u16,
    /// 状态文本
    pub status_text: String,
    /// 响应头
    pub headers: HashMap<String, String>,
    /// 响应体
    pub body: Vec<u8>,
}

impl HttpResponse {
    /// 解析 JSON 响应体
    pub fn json<T: DeserializeOwned>(&self) -> WaeResult<T> {
        serde_json::from_slice(&self.body).map_err(|e| WaeError::parse_error("JSON", e.to_string()))
    }

    /// 获取文本响应体
    pub fn text(&self) -> WaeResult<String> {
        String::from_utf8(self.body.clone()).map_err(|e| WaeError::parse_error("UTF-8", e.to_string()))
    }

    /// 检查是否成功
    pub fn is_success(&self) -> bool {
        self.status >= 200 && self.status < 300
    }
}

/// hyper 客户端 (全局共享)
static HYPER_CLIENT: std::sync::OnceLock<
    Arc<Client<hyper_tls::HttpsConnector<hyper_util::client::legacy::connect::HttpConnector>, Full<Bytes>>>,
> = std::sync::OnceLock::new();

fn get_hyper_client() -> Arc<Client<hyper_tls::HttpsConnector<hyper_util::client::legacy::connect::HttpConnector>, Full<Bytes>>>
{
    HYPER_CLIENT
        .get_or_init(|| {
            let mut http = hyper_util::client::legacy::connect::HttpConnector::new();
            http.enforce_http(false);

            let https = hyper_tls::HttpsConnector::new_with_connector(http);

            let client = Client::builder(TokioExecutor::new()).build(https);

            Arc::new(client)
        })
        .clone()
}

/// HTTP 客户端
#[derive(Debug, Clone)]
pub struct HttpClient {
    config: HttpClientConfig,
}

impl Default for HttpClient {
    fn default() -> Self {
        Self::new(HttpClientConfig::default())
    }
}

impl HttpClient {
    /// 创建新的 HTTP 客户端
    pub fn new(config: HttpClientConfig) -> Self {
        Self { config }
    }

    /// 使用默认配置创建
    pub fn with_defaults() -> Self {
        Self::default()
    }

    /// 获取配置
    pub fn config(&self) -> &HttpClientConfig {
        &self.config
    }

    /// 发送 GET 请求
    pub async fn get(&self, url: &str) -> WaeResult<HttpResponse> {
        self.request("GET", url, None, None).await
    }

    /// 发送带请求头的 GET 请求
    pub async fn get_with_headers(&self, url: &str, headers: HashMap<String, String>) -> WaeResult<HttpResponse> {
        self.request("GET", url, None, Some(headers)).await
    }

    /// 发送 POST JSON 请求
    pub async fn post_json<T: Serialize>(&self, url: &str, body: &T) -> WaeResult<HttpResponse> {
        let json_body = serde_json::to_vec(body).map_err(|_e| WaeError::serialization_failed("JSON"))?;

        let mut headers = HashMap::new();
        headers.insert("Content-Type".to_string(), "application/json".to_string());

        self.request("POST", url, Some(json_body), Some(headers)).await
    }

    /// 发送带请求头的 POST 请求
    pub async fn post_with_headers(
        &self,
        url: &str,
        body: Vec<u8>,
        headers: HashMap<String, String>,
    ) -> WaeResult<HttpResponse> {
        self.request("POST", url, Some(body), Some(headers)).await
    }

    /// 发送请求 (带重试)
    pub async fn request(
        &self,
        method: &str,
        url: &str,
        body: Option<Vec<u8>>,
        headers: Option<HashMap<String, String>>,
    ) -> WaeResult<HttpResponse> {
        let mut last_error = None;

        for attempt in 0..=self.config.max_retries {
            if attempt > 0 {
                let delay = self.config.retry_delay * attempt;
                debug!("Retry attempt {} after {:?}", attempt, delay);
                tokio::time::sleep(delay).await;
            }

            match self.request_once(method, url, body.clone(), headers.clone()).await {
                Ok(response) => {
                    if response.is_success() {
                        info!("Request succeeded on attempt {}", attempt);
                        return Ok(response);
                    }

                    if Self::is_retryable_status(response.status) && attempt < self.config.max_retries {
                        last_error = Some(WaeError::new(WaeErrorKind::RequestError {
                            url: url.to_string(),
                            reason: format!("HTTP {}: {}", response.status, String::from_utf8_lossy(&response.body)),
                        }));
                        continue;
                    }

                    return Err(WaeError::new(WaeErrorKind::RequestError {
                        url: url.to_string(),
                        reason: format!("HTTP {}: {}", response.status, String::from_utf8_lossy(&response.body)),
                    }));
                }
                Err(e) => {
                    error!("Request error on attempt {}: {}", attempt, e);
                    if Self::is_retryable_error(&e) && attempt < self.config.max_retries {
                        last_error = Some(e);
                        continue;
                    }
                    return Err(e);
                }
            }
        }

        Err(last_error.unwrap_or_else(|| WaeError::operation_timeout("request", self.config.timeout.as_millis() as u64)))
    }

    /// 发送单次请求
    async fn request_once(
        &self,
        method: &str,
        url_str: &str,
        body: Option<Vec<u8>>,
        extra_headers: Option<HashMap<String, String>>,
    ) -> WaeResult<HttpResponse> {
        let _url = Url::parse(url_str).map_err(|e| WaeError::invalid_params("url", e.to_string()))?;
        let uri = url_str.parse::<Uri>().map_err(|e| WaeError::invalid_params("url", e.to_string()))?;

        let client = get_hyper_client();

        let mut builder = Request::builder().method(method).uri(uri).header("User-Agent", &self.config.user_agent);

        for (key, value) in &self.config.default_headers {
            builder = builder.header(key, value);
        }

        if let Some(headers) = extra_headers {
            for (key, value) in headers {
                builder = builder.header(key, value);
            }
        }

        let request = match body {
            Some(b) => {
                let len = b.len();
                builder.header("Content-Length", len).body(Full::new(Bytes::from(b))).map_err(|e| {
                    WaeError::new(WaeErrorKind::RequestError { url: url_str.to_string(), reason: e.to_string() })
                })?
            }
            None => builder
                .body(Full::new(Bytes::new()))
                .map_err(|e| WaeError::new(WaeErrorKind::RequestError { url: url_str.to_string(), reason: e.to_string() }))?,
        };

        let response = timeout(self.config.timeout, client.request(request))
            .await
            .map_err(|_| WaeError::operation_timeout("request", self.config.timeout.as_millis() as u64))?
            .map_err(|_e| WaeError::new(WaeErrorKind::ConnectionFailed { target: url_str.to_string() }))?;

        let status = response.status().as_u16();
        let status_text = response.status().canonical_reason().unwrap_or("").to_string();

        let version = match response.version() {
            hyper::Version::HTTP_09 => "HTTP/0.9".to_string(),
            hyper::Version::HTTP_10 => "HTTP/1.0".to_string(),
            hyper::Version::HTTP_11 => "HTTP/1.1".to_string(),
            hyper::Version::HTTP_2 => "HTTP/2".to_string(),
            hyper::Version::HTTP_3 => "HTTP/3".to_string(),
            _ => "HTTP/Unknown".to_string(),
        };

        let mut headers = HashMap::new();
        for (key, value) in response.headers() {
            if let Ok(value_str) = value.to_str() {
                headers.insert(key.as_str().to_string(), value_str.to_string());
            }
        }

        let body_bytes = response
            .into_body()
            .collect()
            .await
            .map_err(|e| {
                WaeError::new(WaeErrorKind::ProtocolError {
                    protocol: "HTTP".to_string(),
                    reason: format!("Read body failed: {}", e),
                })
            })?
            .to_bytes();

        Ok(HttpResponse { version, status, status_text, headers, body: body_bytes.to_vec() })
    }

    /// 判断状态码是否可重试
    fn is_retryable_status(status: u16) -> bool {
        matches!(status, 408 | 429 | 500 | 502 | 503 | 504)
    }

    /// 判断错误是否可重试
    fn is_retryable_error(error: &WaeError) -> bool {
        matches!(
            error.kind.as_ref(),
            WaeErrorKind::OperationTimeout { .. }
                | WaeErrorKind::ConnectionFailed { .. }
                | WaeErrorKind::DnsResolutionFailed { .. }
        )
    }
}

/// 请求构建器
pub struct RequestBuilder {
    client: HttpClient,
    method: String,
    url: String,
    headers: HashMap<String, String>,
    body: Option<Vec<u8>>,
}

impl RequestBuilder {
    /// 创建新的请求构建器
    pub fn new(client: HttpClient, method: &str, url: &str) -> Self {
        Self { client, method: method.to_string(), url: url.to_string(), headers: HashMap::new(), body: None }
    }

    /// 添加请求头
    pub fn header(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.headers.insert(key.into(), value.into());
        self
    }

    /// 添加 Bearer Token
    pub fn bearer_auth(self, token: impl Into<String>) -> Self {
        self.header("Authorization", format!("Bearer {}", token.into()))
    }

    /// 设置 JSON 请求体
    pub fn json<T: Serialize>(mut self, body: &T) -> Self {
        if let Ok(json) = serde_json::to_vec(body) {
            self.headers.insert("Content-Type".into(), "application/json".into());
            self.body = Some(json);
        }
        self
    }

    /// 设置原始请求体
    pub fn body(mut self, body: Vec<u8>) -> Self {
        self.body = Some(body);
        self
    }

    /// 发送请求
    pub async fn send(self) -> WaeResult<HttpResponse> {
        self.client.request(&self.method, &self.url, self.body, Some(self.headers)).await
    }

    /// 发送请求并解析 JSON
    pub async fn send_json<T: DeserializeOwned>(self) -> WaeResult<T> {
        let response = self.send().await?;
        response.json()
    }
}

/// 便捷函数：创建 GET 请求
pub fn get(url: &str) -> RequestBuilder {
    RequestBuilder::new(HttpClient::default(), "GET", url)
}

/// 便捷函数：创建 POST 请求
pub fn post(url: &str) -> RequestBuilder {
    RequestBuilder::new(HttpClient::default(), "POST", url)
}

/// 兼容旧 API 的 RequestClient
#[derive(Debug, Clone)]
pub struct RequestClient {
    client: HttpClient,
    config: RequestConfig,
}

/// 兼容旧 API 的配置
#[derive(Debug, Clone)]
pub struct RequestConfig {
    /// 请求超时时间（秒）
    pub timeout_secs: u64,
    /// 连接超时时间（秒）
    pub connect_timeout_secs: u64,
    /// 最大重试次数
    pub max_retries: u32,
    /// 重试间隔（毫秒）
    pub retry_delay_ms: u64,
    /// 用户代理
    pub user_agent: String,
}

impl Default for RequestConfig {
    fn default() -> Self {
        Self {
            timeout_secs: 30,
            connect_timeout_secs: 10,
            max_retries: 3,
            retry_delay_ms: 1000,
            user_agent: "wae-request/0.1.0".to_string(),
        }
    }
}

impl RequestClient {
    /// 创建新的 HTTP 客户端
    pub fn new(config: RequestConfig) -> WaeResult<Self> {
        let http_config = HttpClientConfig {
            timeout: Duration::from_secs(config.timeout_secs),
            connect_timeout: Duration::from_secs(config.connect_timeout_secs),
            user_agent: config.user_agent.clone(),
            max_retries: config.max_retries,
            retry_delay: Duration::from_millis(config.retry_delay_ms),
            default_headers: HashMap::new(),
        };
        Ok(Self { client: HttpClient::new(http_config), config })
    }

    /// 使用默认配置创建
    pub fn with_defaults() -> WaeResult<Self> {
        Self::new(RequestConfig::default())
    }

    /// 获取配置
    pub fn config(&self) -> &RequestConfig {
        &self.config
    }

    /// 发送 GET 请求并解析 JSON
    pub async fn get<T: DeserializeOwned>(&self, url: &str) -> WaeResult<T> {
        let response = self.client.get(url).await?;
        response.json()
    }

    /// 发送 GET 请求返回原始响应
    pub async fn get_raw(&self, url: &str) -> WaeResult<HttpResponse> {
        self.client.get(url).await
    }

    /// 发送 POST JSON 请求并解析响应
    pub async fn post<T: DeserializeOwned, B: Serialize>(&self, url: &str, body: &B) -> WaeResult<T> {
        let response = self.client.post_json(url, body).await?;
        response.json()
    }

    /// 发送 POST JSON 请求返回原始响应
    pub async fn post_raw<B: Serialize>(&self, url: &str, body: &B) -> WaeResult<HttpResponse> {
        self.client.post_json(url, body).await
    }

    /// 创建请求构建器
    pub fn builder(&self) -> RequestBuilder {
        RequestBuilder::new(self.client.clone(), "GET", "")
    }
}
