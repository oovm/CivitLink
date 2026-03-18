//! 追踪中间件模块
//!
//! 提供追踪中间件，用于自动提取和注入 trace context。

use http::{Request, Response};
use pin_project_lite::pin_project;
use std::{
    future::Future,
    pin::Pin,
    task::{Context as TaskContext, Poll},
};
use tower::{Layer, Service};

use crate::tracing::{TraceContext, extract_context_from_headers, inject_context_to_headers, set_global_text_map_propagator};

/// 追踪中间件 Layer
#[derive(Clone, Debug, Default)]
pub struct OpenTelemetryLayer {
    _private: (),
}

impl OpenTelemetryLayer {
    /// 创建新的 OpenTelemetryLayer
    pub fn new() -> Self {
        set_global_text_map_propagator();
        Self { _private: () }
    }
}

impl<S> Layer<S> for OpenTelemetryLayer {
    type Service = OpenTelemetryService<S>;

    fn layer(&self, inner: S) -> Self::Service {
        OpenTelemetryService { inner }
    }
}

/// 追踪中间件 Service
#[derive(Clone, Debug)]
pub struct OpenTelemetryService<S> {
    inner: S,
}

impl<S, ReqBody, ResBody> Service<Request<ReqBody>> for OpenTelemetryService<S>
where
    S: Service<Request<ReqBody>, Response = Response<ResBody>>,
    ResBody: Default,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = OpenTelemetryFuture<S::Future>;

    fn poll_ready(&mut self, cx: &mut TaskContext<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, req: Request<ReqBody>) -> Self::Future {
        let context = extract_context_from_headers(req.headers()).unwrap_or_default();
        let child_context = context.child();

        OpenTelemetryFuture { inner: self.inner.call(req), context: child_context }
    }
}

pin_project! {
    /// 追踪中间件 Future
    pub struct OpenTelemetryFuture<F> {
        #[pin]
        inner: F,
        context: TraceContext,
    }
}

impl<F, ResBody, E> Future for OpenTelemetryFuture<F>
where
    F: Future<Output = Result<Response<ResBody>, E>>,
    ResBody: Default,
{
    type Output = F::Output;

    fn poll(self: Pin<&mut Self>, cx: &mut TaskContext<'_>) -> Poll<Self::Output> {
        let this = self.project();
        let result = futures_util::ready!(this.inner.poll(cx));
        match result {
            Ok(mut res) => {
                inject_context_to_headers(this.context, res.headers_mut());
                Poll::Ready(Ok(res))
            }
            Err(e) => Poll::Ready(Err(e)),
        }
    }
}
