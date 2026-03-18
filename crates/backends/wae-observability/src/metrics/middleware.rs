//! HTTP 指标收集中间件
//!
//! 提供 Tower Layer 和 Service 实现，用于自动收集 HTTP 请求指标。

use crate::metrics::MetricsRegistry;
use futures_util::future::BoxFuture;
use http::{Request, Response, StatusCode};
use std::{
    sync::Arc,
    task::{Context, Poll},
};
use tower::{Layer, Service};

/// 指标收集 Layer
///
/// Tower Layer 实现，用于为 HTTP 服务添加指标收集功能。
#[derive(Clone)]
pub struct MetricsLayer {
    registry: Arc<MetricsRegistry>,
}

impl MetricsLayer {
    /// 创建新的指标收集 Layer
    ///
    /// # Arguments
    ///
    /// * `registry` - 指标注册器
    pub fn new(registry: Arc<MetricsRegistry>) -> Self {
        Self { registry }
    }
}

impl<S> Layer<S> for MetricsLayer {
    type Service = MetricsService<S>;

    fn layer(&self, inner: S) -> Self::Service {
        MetricsService { inner, registry: Arc::clone(&self.registry) }
    }
}

/// 指标收集 Service
///
/// Tower Service 实现，用于自动收集 HTTP 请求指标。
#[derive(Clone)]
pub struct MetricsService<S> {
    inner: S,
    registry: Arc<MetricsRegistry>,
}

impl<S, ReqBody, ResBody> Service<Request<ReqBody>> for MetricsService<S>
where
    S: Service<Request<ReqBody>, Response = Response<ResBody>>,
    S::Future: Send + 'static,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = BoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, req: Request<ReqBody>) -> Self::Future {
        let registry = Arc::clone(&self.registry);
        let method = req.method().to_string();
        let path = req.uri().path().to_string();
        let start = std::time::Instant::now();

        let future = self.inner.call(req);

        Box::pin(async move {
            let result = future.await;
            let duration = start.elapsed().as_secs_f64();

            let status = match &result {
                Ok(response) => status_code_to_category(response.status()),
                Err(_) => "5xx",
            };

            registry.record_http_request(&method, &path, status, duration);

            result
        })
    }
}

/// 将 HTTP 状态码转换为分类标签
///
/// # Arguments
///
/// * `status` - HTTP 状态码
///
/// # Returns
///
/// 状态码分类: "2xx", "4xx", "5xx"
fn status_code_to_category(status: StatusCode) -> &'static str {
    match status.as_u16() {
        200..=299 => "2xx",
        400..=499 => "4xx",
        500..=599 => "5xx",
        _ => "other",
    }
}
