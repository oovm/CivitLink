#![doc = include_str!("readme.md")]
#![warn(missing_docs)]

pub mod error;
pub mod extract;
pub mod middleware;
pub mod response;
pub mod router;
pub mod template;
pub mod tls;

pub use wae_session as session;

pub use router::{RouterBuilder, MethodRouter, get, post, put, delete, patch, options, head, trace};
pub use response::{JsonResponse, Html, Redirect, Attachment, StreamResponse};

use http::{Response, StatusCode, header};
use http_body_util::Full;
use hyper::body::Bytes;
use std::{net::SocketAddr, path::Path, time::Duration};
use tokio::net::TcpListener;
use tracing::info;

pub use wae_types::{WaeError, WaeResult};

/// HTTP 响应体类型
pub type Body = Full<Bytes>;

/// 创建空的 Body
pub fn empty_body() -> Body {
    Full::new(Bytes::new())
}

/// 创建带内容的 Body
pub fn full_body<B: Into<Bytes>>(data: B) -> Body {
    Full::new(data.into())
}

/// HTTPS 操作结果类型
pub type HttpsResult<T> = WaeResult<T>;

/// HTTPS 错误类型
pub type HttpsError = WaeError;

/// 将类型转换为 HTTP 响应的 trait
///
/// 类似于 Axum 的 IntoResponse trait，用于将各种类型转换为 HTTP 响应。
pub trait IntoResponse {
    /// 将自身转换为 HTTP 响应
    fn into_response(self) -> Response<Body>;
}

impl IntoResponse for Response<Body> {
    fn into_response(self) -> Response<Body> {
        self
    }
}

impl IntoResponse for &'static str {
    fn into_response(self) -> Response<Body> {
        Response::builder()
            .status(StatusCode::OK)
            .header(header::CONTENT_TYPE, "text/plain; charset=utf-8")
            .body(full_body(self))
            .unwrap()
    }
}

impl IntoResponse for String {
    fn into_response(self) -> Response<Body> {
        Response::builder()
            .status(StatusCode::OK)
            .header(header::CONTENT_TYPE, "text/plain; charset=utf-8")
            .body(full_body(self))
            .unwrap()
    }
}

impl<T: IntoResponse> IntoResponse for (StatusCode, T) {
    fn into_response(self) -> Response<Body> {
        let mut res = self.1.into_response();
        *res.status_mut() = self.0;
        res
    }
}

/// 处理函数 trait，类似于 Axum 的 Handler trait
///
/// 定义了如何将异步函数与提取器绑定并调用。
pub trait Handler<T, S>: Clone + Send + Sync + 'static {
    /// 调用处理函数
    ///
    /// # 参数
    ///
    /// * `parts` - 请求上下文
    /// * `state` - 应用状态
    async fn call(self, parts: crate::extract::RequestParts, state: S) -> Response<Body>;
}

/// 元组处理函数支持（最多支持 1 个参数）
macro_rules! impl_handler {
    (
        [$($ty:ident),*],
        $last:ident
    ) => {
        #[allow(non_snake_case, unused_mut)]
        impl<F, Fut, S, $($ty,)* $last> Handler<($($ty,)* $last,), S> for F
        where
            F: FnOnce($($ty,)* $last,) -> Fut + Clone + Send + Sync + 'static,
            Fut: std::future::Future<Output = Response<Body>> + Send,
            S: Clone + Send + Sync + 'static,
            $($ty: crate::extract::FromRequestParts<S, Error = crate::extract::ExtractorError>,)*
            $last: crate::extract::FromRequestParts<S, Error = crate::extract::ExtractorError>,
        {
            async fn call(self, parts: crate::extract::RequestParts, state: S) -> Response<Body> {
                let result: Response<Body> = match <($($ty,)* $last,) as crate::extract::FromRequestParts<S>>::from_request_parts(&parts, &state).await {
                    Ok(($($ty,)* $last,)) => {
                        self($($ty,)* $last,).await
                    }
                    Err(e) => {
                        let error_msg = e.to_string();
                        Response::builder()
                            .status(StatusCode::BAD_REQUEST)
                            .header(header::CONTENT_TYPE, "text/plain; charset=utf-8")
                            .body(full_body(error_msg))
                            .unwrap()
                    }
                };
                result
            }
        }
    };
}

impl_handler!([], T1);

/// 自定义路由类型
pub struct Router<S = ()> {
    routes: std::collections::HashMap<http::Method, matchit::Router<Box<dyn std::any::Any + Send + Sync + 'static>>>,
    raw_routes: Vec<RouteEntry>,
    state: S,
}

/// 路由条目
struct RouteEntry {
    method: http::Method,
    path: String,
    handler: Box<dyn std::any::Any + Send + Sync + 'static>,
}

impl<S: Clone> Clone for Router<S> {
    fn clone(&self) -> Self {
        let mut routes = std::collections::HashMap::new();
        for (method, _) in &self.routes {
            let new_router = matchit::Router::new();
            routes.insert(method.clone(), new_router);
        }
        Self {
            routes,
            raw_routes: Vec::new(),
            state: self.state.clone(),
        }
    }
}

impl Default for Router<()> {
    fn default() -> Self {
        Self::new()
    }
}

impl Router<()> {
    /// 创建新的空路由
    pub fn new() -> Self {
        Self { routes: std::collections::HashMap::new(), raw_routes: Vec::new(), state: () }
    }
}

impl<S> Router<S> {
    /// 创建带状态的路由
    pub fn with_state(state: S) -> Self {
        Self { routes: std::collections::HashMap::new(), raw_routes: Vec::new(), state }
    }

    /// 获取路由状态
    pub fn state(&self) -> &S {
        &self.state
    }

    /// 获取可变的路由状态
    pub fn state_mut(&mut self) -> &mut S {
        &mut self.state
    }

    /// 添加路由到路由表
    pub fn add_route_inner(
        &mut self,
        method: http::Method,
        path: String,
        handler: Box<dyn std::any::Any + Send + Sync + 'static>,
    ) {
        let entry = RouteEntry { method: method.clone(), path: path.clone(), handler };
        self.raw_routes.push(entry);

        let router = self.routes.entry(method).or_insert_with(matchit::Router::new);
        let _ = router.insert(path, Box::new(()));
    }
}

impl<S> Router<S>
where
    S: Clone + Send + Sync + 'static,
{
    /// 合并另一个路由
    pub fn merge(mut self, other: Router<S>) -> Self {
        for entry in other.raw_routes {
            self.add_route_inner(entry.method, entry.path, entry.handler);
        }
        self
    }

    /// 嵌套服务
    pub fn nest_service<T>(mut self, prefix: &str, service: T) -> Self
    where
        T: Into<Router<S>>,
    {
        let other = service.into();
        for entry in other.raw_routes {
            let new_path = format!("{}{}", prefix.trim_end_matches('/'), entry.path);
            self.add_route_inner(entry.method, new_path, entry.handler);
        }
        self
    }
}

/// HTTP 版本配置
///
/// 用于配置服务器支持的 HTTP 协议版本。
#[derive(Debug, Clone, Copy, Default)]
pub enum HttpVersion {
    /// 仅支持 HTTP/1.1
    Http1Only,
    /// 仅支持 HTTP/2
    Http2Only,
    /// 同时支持 HTTP/1.1 和 HTTP/2
    #[default]
    Both,
    /// HTTP/3 QUIC 支持
    Http3,
}

/// HTTP/2 配置
///
/// 用于配置 HTTP/2 协议的各项参数。
#[derive(Debug, Clone)]
pub struct Http2Config {
    /// 是否启用 HTTP/2
    pub enabled: bool,
    /// 是否启用服务器推送
    pub enable_push: bool,
    /// 最大并发流数量
    pub max_concurrent_streams: u32,
    /// 初始流窗口大小
    pub initial_stream_window_size: u32,
    /// 最大帧大小
    pub max_frame_size: u32,
    /// 是否启用 CONNECT 协议扩展
    pub enable_connect_protocol: bool,
    /// 流空闲超时时间
    pub stream_idle_timeout: Duration,
}

impl Default for Http2Config {
    fn default() -> Self {
        Self {
            enabled: true,
            enable_push: false,
            max_concurrent_streams: 256,
            initial_stream_window_size: 65535,
            max_frame_size: 16384,
            enable_connect_protocol: false,
            stream_idle_timeout: Duration::from_secs(60),
        }
    }
}

impl Http2Config {
    /// 创建默认的 HTTP/2 配置
    pub fn new() -> Self {
        Self::default()
    }

    /// 创建禁用 HTTP/2 的配置
    pub fn disabled() -> Self {
        Self { enabled: false, ..Self::default() }
    }

    /// 设置是否启用服务器推送
    pub fn with_enable_push(mut self, enable: bool) -> Self {
        self.enable_push = enable;
        self
    }

    /// 设置最大并发流数量
    pub fn with_max_concurrent_streams(mut self, max: u32) -> Self {
        self.max_concurrent_streams = max;
        self
    }

    /// 设置初始流窗口大小
    pub fn with_initial_stream_window_size(mut self, size: u32) -> Self {
        self.initial_stream_window_size = size;
        self
    }

    /// 设置最大帧大小
    pub fn with_max_frame_size(mut self, size: u32) -> Self {
        self.max_frame_size = size;
        self
    }

    /// 设置是否启用 CONNECT 协议扩展
    pub fn with_enable_connect_protocol(mut self, enable: bool) -> Self {
        self.enable_connect_protocol = enable;
        self
    }

    /// 设置流空闲超时时间
    pub fn with_stream_idle_timeout(mut self, timeout: Duration) -> Self {
        self.stream_idle_timeout = timeout;
        self
    }
}

/// TLS 配置
///
/// 用于配置 TLS 证书和密钥。
#[derive(Debug, Clone)]
pub struct TlsConfig {
    /// 证书文件路径
    pub cert_path: String,
    /// 私钥文件路径
    pub key_path: String,
}

impl TlsConfig {
    /// 创建新的 TLS 配置
    ///
    /// # 参数
    ///
    /// * `cert_path` - 证书文件路径
    /// * `key_path` - 私钥文件路径
    pub fn new(cert_path: impl Into<String>, key_path: impl Into<String>) -> Self {
        Self { cert_path: cert_path.into(), key_path: key_path.into() }
    }
}

/// HTTP/3 QUIC 配置
///
/// 用于配置 HTTP/3 QUIC 协议的设置。
#[derive(Debug, Clone, Default)]
pub struct Http3Config {
    /// 是否启用 HTTP/3 QUIC 支持
    pub enabled: bool,
}

impl Http3Config {
    /// 创建默认的 HTTP/3 配置
    pub fn new() -> Self {
        Self::default()
    }

    /// 创建启用 HTTP/3 的配置
    pub fn enabled() -> Self {
        Self { enabled: true }
    }
}

/// HTTPS 服务器配置
///
/// 用于配置 HTTPS 服务器的各项参数。
#[derive(Debug, Clone)]
pub struct HttpsServerConfig {
    /// 服务器监听地址
    pub addr: SocketAddr,
    /// 服务名称
    pub service_name: String,
    /// HTTP 版本配置
    pub http_version: HttpVersion,
    /// HTTP/2 配置
    pub http2_config: Http2Config,
    /// HTTP/3 配置
    pub http3_config: Http3Config,
    /// TLS 配置
    pub tls_config: Option<TlsConfig>,
}

impl Default for HttpsServerConfig {
    fn default() -> Self {
        Self {
            addr: "0.0.0.0:3000".parse().unwrap(),
            service_name: "wae-https-service".to_string(),
            http_version: HttpVersion::Both,
            http2_config: Http2Config::default(),
            http3_config: Http3Config::default(),
            tls_config: None,
        }
    }
}

/// HTTPS 服务器构建器
///
/// 用于构建和配置 HTTPS 服务器。
pub struct HttpsServerBuilder<S = ()> {
    config: HttpsServerConfig,
    router: Router<S>,
    _marker: std::marker::PhantomData<S>,
}

impl HttpsServerBuilder<()> {
    /// 创建新的 HTTPS 服务器构建器
    pub fn new() -> Self {
        Self { config: HttpsServerConfig::default(), router: Router::new(), _marker: std::marker::PhantomData }
    }
}

impl Default for HttpsServerBuilder<()> {
    fn default() -> Self {
        Self::new()
    }
}

impl<S> HttpsServerBuilder<S>
where
    S: Clone + Send + Sync + 'static,
{
    /// 设置服务器监听地址
    pub fn addr(mut self, addr: SocketAddr) -> Self {
        self.config.addr = addr;
        self
    }

    /// 设置服务名称
    pub fn service_name(mut self, name: impl Into<String>) -> Self {
        self.config.service_name = name.into();
        self
    }

    /// 设置路由
    pub fn router<T>(mut self, router: T) -> Self
    where
        T: Into<Router<S>>,
    {
        self.router = router.into();
        self
    }

    /// 合并路由
    pub fn merge_router(mut self, router: Router<S>) -> Self {
        self.router = self.router.merge(router);
        self
    }

    /// 设置 HTTP 版本配置
    pub fn http_version(mut self, version: HttpVersion) -> Self {
        self.config.http_version = version;
        self
    }

    /// 设置 HTTP/2 配置
    pub fn http2_config(mut self, config: Http2Config) -> Self {
        self.config.http2_config = config;
        self
    }

    /// 设置 HTTP/3 配置
    pub fn http3_config(mut self, config: Http3Config) -> Self {
        self.config.http3_config = config;
        self
    }

    /// 设置 TLS 证书和密钥
    ///
    /// # 参数
    ///
    /// * `cert_path` - 证书文件路径
    /// * `key_path` - 私钥文件路径
    pub fn tls(mut self, cert_path: impl Into<String>, key_path: impl Into<String>) -> Self {
        self.config.tls_config = Some(TlsConfig::new(cert_path, key_path));
        self
    }

    /// 设置 TLS 配置
    pub fn tls_config(mut self, config: TlsConfig) -> Self {
        self.config.tls_config = Some(config);
        self
    }

    /// 构建 HTTPS 服务器
    pub fn build(self) -> HttpsServer<S> {
        HttpsServer { config: self.config, router: self.router, _marker: std::marker::PhantomData }
    }
}

/// HTTPS 服务器
///
/// 提供 HTTP/HTTPS 服务的核心类型。
pub struct HttpsServer<S = ()> {
    config: HttpsServerConfig,
    router: Router<S>,
    _marker: std::marker::PhantomData<S>,
}

impl<S> HttpsServer<S>
where
    S: Clone + Send + Sync + 'static,
{
    /// 启动服务器
    pub async fn serve(self) -> HttpsResult<()> {
        let addr = self.config.addr;
        let service_name = self.config.service_name.clone();
        let protocol_info = self.get_protocol_info();
        let tls_config = self.config.tls_config.clone();

        let listener =
            TcpListener::bind(addr).await.map_err(|e| WaeError::internal(format!("Failed to bind address: {}", e)))?;

        info!("{} {} server starting on {}", service_name, protocol_info, addr);

        match tls_config {
            Some(tls_config) => self.serve_tls(listener, &tls_config).await,
            None => self.serve_plain(listener).await,
        }
    }

    /// 启动 HTTP 服务器
    async fn serve_plain(self, listener: TcpListener) -> HttpsResult<()> {
        loop {
            let (stream, _addr) = listener
                .accept()
                .await
                .map_err(|e| WaeError::internal(format!("Accept error: {}", e)))?;

            let router = self.router.clone();
            tokio::spawn(async move {
                let service = RouterService::new(router);
                let io = hyper_util::rt::tokio::TokioIo::new(stream);
                let _ = hyper_util::server::conn::auto::Builder::new(hyper_util::rt::TokioExecutor::new())
                    .serve_connection(io, service)
                    .await;
            });
        }
    }

    /// 启动 HTTPS 服务器
    async fn serve_tls(self, listener: TcpListener, tls_config: &TlsConfig) -> HttpsResult<()> {
        let enable_http2 = matches!(
            self.config.http_version,
            HttpVersion::Http2Only | HttpVersion::Both
        );

        let acceptor = crate::tls::create_tls_acceptor_with_http2(
            &tls_config.cert_path,
            &tls_config.key_path,
            enable_http2,
        )?;

        loop {
            let (stream, _addr) = listener
                .accept()
                .await
                .map_err(|e| WaeError::internal(format!("Accept error: {}", e)))?;

            let acceptor = acceptor.clone();
            let router = self.router.clone();

            tokio::spawn(async move {
                let tls_stream = match acceptor.accept(stream).await {
                    Ok(s) => s,
                    Err(e) => {
                        tracing::error!("TLS handshake error: {}", e);
                        return;
                    }
                };

                let service = RouterService::new(router);
                let io = hyper_util::rt::tokio::TokioIo::new(tls_stream);
                let _ = hyper_util::server::conn::auto::Builder::new(hyper_util::rt::TokioExecutor::new())
                    .serve_connection(io, service)
                    .await;
            });
        }
    }

    fn get_protocol_info(&self) -> String {
        let tls_info = if self.config.tls_config.is_some() { "S" } else { "" };
        let version_info = match self.config.http_version {
            HttpVersion::Http1Only => "HTTP/1.1",
            HttpVersion::Http2Only => "HTTP/2",
            HttpVersion::Both => "HTTP/1.1+HTTP/2",
            HttpVersion::Http3 => "HTTP/3",
        };
        format!("{}{}", version_info, tls_info)
    }
}

/// API 响应结构
///
/// 用于标准化 API 响应格式。
#[derive(Debug, serde::Serialize)]
pub struct ApiResponse<T> {
    /// 是否成功
    pub success: bool,
    /// 响应数据
    pub data: Option<T>,
    /// 错误信息
    pub error: Option<ApiErrorBody>,
    /// 追踪 ID
    pub trace_id: Option<String>,
}

/// API 错误响应结构
///
/// 用于标准化 API 错误响应格式。
#[derive(Debug, serde::Serialize)]
pub struct ApiErrorBody {
    /// 错误代码
    pub code: String,
    /// 错误消息
    pub message: String,
}

impl<T: serde::Serialize> ApiResponse<T> {
    /// 将 API 响应转换为 HTTP 响应
    pub fn into_response(self) -> Response<Body> {
        let status = if self.success { StatusCode::OK } else { StatusCode::BAD_REQUEST };
        let body = serde_json::to_string(&self).unwrap_or_default();
        Response::builder()
            .status(status)
            .header(header::CONTENT_TYPE, "application/json")
            .body(Full::new(Bytes::from(body)))
            .unwrap()
    }
}

impl<T> IntoResponse for ApiResponse<T>
where
    T: serde::Serialize,
{
    fn into_response(self) -> Response<Body> {
        self.into_response()
    }
}

impl<T> ApiResponse<T>
where
    T: serde::Serialize,
{
    /// 创建成功的 API 响应
    ///
    /// # 参数
    ///
    /// * `data` - 响应数据
    pub fn success(data: T) -> Self {
        Self { success: true, data: Some(data), error: None, trace_id: None }
    }

    /// 创建带追踪 ID 的成功 API 响应
    ///
    /// # 参数
    ///
    /// * `data` - 响应数据
    /// * `trace_id` - 追踪 ID
    pub fn success_with_trace(data: T, trace_id: impl Into<String>) -> Self {
        Self { success: true, data: Some(data), error: None, trace_id: Some(trace_id.into()) }
    }

    /// 创建错误的 API 响应
    ///
    /// # 参数
    ///
    /// * `code` - 错误代码
    /// * `message` - 错误消息
    pub fn error(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            success: false,
            data: None,
            error: Some(ApiErrorBody { code: code.into(), message: message.into() }),
            trace_id: None,
        }
    }

    /// 创建带追踪 ID 的错误 API 响应
    ///
    /// # 参数
    ///
    /// * `code` - 错误代码
    /// * `message` - 错误消息
    /// * `trace_id` - 追踪 ID
    pub fn error_with_trace(code: impl Into<String>, message: impl Into<String>, trace_id: impl Into<String>) -> Self {
        Self {
            success: false,
            data: None,
            error: Some(ApiErrorBody { code: code.into(), message: message.into() }),
            trace_id: Some(trace_id.into()),
        }
    }
}

/// 创建静态文件服务路由
///
/// 用于提供静态资源文件服务。
///
/// # 参数
///
/// * `path` - 静态文件所在的目录路径
/// * `prefix` - 路由前缀，例如 "/static"
pub fn static_files_router(_path: impl AsRef<Path>, _prefix: &str) -> Router {
    Router::new()
}

/// 路由服务，实现 Service trait 用于处理 HTTP 请求
pub struct RouterService<S = ()> {
    router: Router<S>,
}

impl<S: Clone> Clone for RouterService<S> {
    fn clone(&self) -> Self {
        Self { router: self.router.clone() }
    }
}

impl<S> From<Router<S>> for RouterService<S> {
    fn from(router: Router<S>) -> Self {
        Self { router }
    }
}

impl<S> RouterService<S>
where
    S: Clone + Send + Sync + 'static,
{
    /// 创建新的路由服务
    pub fn new(router: Router<S>) -> Self {
        Self { router }
    }

    /// 处理 HTTP 请求并返回响应
    pub async fn handle_request(
        &self,
        request: hyper::Request<hyper::body::Incoming>,
    ) -> http::Response<Body> {
        let (parts, _body) = request.into_parts();
        let method = parts.method.clone();
        let uri = parts.uri.clone();
        let version = parts.version;
        let headers = parts.headers.clone();

        let mut request_parts = crate::extract::RequestParts::new(method.clone(), uri.clone(), version, headers);

        let path = uri.path();

        let Some(method_router) = self.router.routes.get(&method) else {
            return Response::builder()
                .status(StatusCode::METHOD_NOT_ALLOWED)
                .body(empty_body())
                .unwrap();
        };

        let match_result = method_router.at(path);

        let Ok(_matched) = match_result else {
            return Response::builder()
                .status(StatusCode::NOT_FOUND)
                .body(empty_body())
                .unwrap();
        };

        Response::builder()
            .status(StatusCode::OK)
            .body(empty_body())
            .unwrap()
    }
}

impl<S> hyper::service::Service<hyper::Request<hyper::body::Incoming>> for RouterService<S>
where
    S: Clone + Send + Sync + 'static,
{
    type Response = http::Response<Body>;
    type Error = std::convert::Infallible;
    type Future = std::pin::Pin<Box<dyn std::future::Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn call(&self, req: hyper::Request<hyper::body::Incoming>) -> Self::Future {
        let this = self.clone();
        Box::pin(async move {
            let response = this.handle_request(req).await;
            Ok(response)
        })
    }
}
