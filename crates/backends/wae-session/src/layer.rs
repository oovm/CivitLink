//! Session 中间件层
//!
//! 提供 Session 中间件实现，用于在请求处理过程中管理 Session。

use crate::{Session, SessionConfig, SessionStore};
use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};
use http::{Request, Response};
use http_body::Body;
use rand::Rng;
use std::{marker::PhantomData, sync::Arc};
use tower::{Layer, Service};

/// Session 中间件层
///
/// 用于在 tower 应用中添加 Session 支持。
#[derive(Debug, Clone)]
pub struct SessionLayer<S, ReqBody, ResBody>
where
    S: SessionStore,
    ReqBody: Body + Send + 'static,
    ResBody: Body + Send + 'static,
{
    /// Session 存储
    store: S,
    /// Session 配置
    config: SessionConfig,
    /// 用于标记请求体类型
    _phantom: PhantomData<(ReqBody, ResBody)>,
}

impl<S, ReqBody, ResBody> SessionLayer<S, ReqBody, ResBody>
where
    S: SessionStore,
    ReqBody: Body + Send + 'static,
    ResBody: Body + Send + 'static,
{
    /// 创建新的 Session 中间件层
    pub fn new(store: S, config: SessionConfig) -> Self {
        Self { store, config, _phantom: PhantomData }
    }

    /// 使用默认配置创建 Session 中间件层
    pub fn with_store(store: S) -> Self {
        Self::new(store, SessionConfig::default())
    }
}

impl<S, T, ReqBody, ResBody> Layer<T> for SessionLayer<S, ReqBody, ResBody>
where
    S: SessionStore,
    T: Service<Request<ReqBody>, Response = Response<ResBody>> + Clone + Send + 'static,
    T::Future: Send,
    ReqBody: Body + Send + 'static,
    ResBody: Body + Send + 'static,
{
    type Service = SessionService<T, S, ReqBody, ResBody>;

    fn layer(&self, inner: T) -> Self::Service {
        SessionService { inner, store: self.store.clone(), config: self.config.clone(), _phantom: PhantomData }
    }
}

/// Session 服务
///
/// 用于处理 Session 的服务包装器。
#[derive(Debug, Clone)]
pub struct SessionService<T, S, ReqBody, ResBody>
where
    S: SessionStore,
    ReqBody: Body + Send + 'static,
    ResBody: Body + Send + 'static,
{
    /// 内部服务
    inner: T,
    /// Session 存储
    store: S,
    /// Session 配置
    config: SessionConfig,
    /// 用于标记请求体类型
    _phantom: PhantomData<(ReqBody, ResBody)>,
}

impl<T, S, ReqBody, ResBody> SessionService<T, S, ReqBody, ResBody>
where
    S: SessionStore,
    ReqBody: Body + Send + 'static,
    ResBody: Body + Send + 'static,
{
    /// 从请求中提取 Session ID
    fn extract_session_id<B>(&self, request: &Request<B>) -> Option<String> {
        let cookie_header = request.headers().get("cookie")?.to_str().ok()?;

        for cookie in cookie_header.split(';') {
            let cookie = cookie.trim();
            if let Some(value) = cookie.strip_prefix(&format!("{}=", self.config.cookie_name)) {
                return Some(value.to_string());
            }
        }

        None
    }
}

impl<T, S, ReqBody, ResBody> Service<Request<ReqBody>> for SessionService<T, S, ReqBody, ResBody>
where
    S: SessionStore,
    T: Service<Request<ReqBody>, Response = Response<ResBody>> + Clone + Send + 'static,
    T::Future: Send,
    ReqBody: Body + Send + 'static,
    ResBody: Body + Send + 'static,
{
    type Response = Response<ResBody>;
    type Error = T::Error;
    type Future = std::pin::Pin<Box<dyn std::future::Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, cx: &mut std::task::Context<'_>) -> std::task::Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, mut request: Request<ReqBody>) -> Self::Future {
        let mut inner = self.inner.clone();
        let session_id = self.extract_session_id(&request);
        let config = self.config.clone();
        let store = self.store.clone();

        Box::pin(async move {
            let generate_session_id = || -> String {
                let mut bytes = vec![0u8; config.id_length];
                rand::rng().fill_bytes(&mut bytes);
                URL_SAFE_NO_PAD.encode(&bytes)
            };

            let session = if let Some(id) = session_id {
                if let Some(data) = store.get(&id).await {
                    if let Ok(data_map) = serde_json::from_str(&data) {
                        Session::from_data(id.to_string(), data_map).await
                    }
                    else {
                        Session::new(generate_session_id())
                    }
                }
                else {
                    Session::new(generate_session_id())
                }
            }
            else {
                Session::new(generate_session_id())
            };

            let session = Arc::new(session);
            request.extensions_mut().insert(session.clone());

            let mut response = inner.call(request).await?;

            if session.is_dirty().await || session.is_new() {
                let data = session.to_json().await;
                store.set(session.id(), &data, config.ttl).await;

                let cookie_value = config.build_cookie_header(session.id());
                if let Ok(header_value) = cookie_value.parse() {
                    response.headers_mut().append(http::header::SET_COOKIE, header_value);
                }
            }

            Ok(response)
        })
    }
}
