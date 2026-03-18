//! 追踪中间件
//!
//! 为请求添加追踪日志。

use http::{Request, Response};
use pin_project_lite::pin_project;
use std::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};
use tower::{Layer, Service};
use tracing::{info, info_span};

/// 追踪中间件层
#[derive(Debug, Clone)]
pub struct TracingLayer;

impl<S> Layer<S> for TracingLayer {
    type Service = TracingService<S>;

    fn layer(&self, inner: S) -> Self::Service {
        TracingService { inner }
    }
}

/// 追踪服务
#[derive(Debug, Clone)]
pub struct TracingService<S> {
    inner: S,
}

impl<S, ReqBody, ResBody> Service<Request<ReqBody>> for TracingService<S>
where
    S: Service<Request<ReqBody>, Response = Response<ResBody>>,
    S::Future: Send + 'static,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = TracingFuture<S::Future>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, req: Request<ReqBody>) -> Self::Future {
        let method = req.method().to_string();
        let uri = req.uri().to_string();
        let span = info_span!("http_request", method = %method, uri = %uri);

        let future = self.inner.call(req);
        TracingFuture { inner: future, span }
    }
}

pin_project! {
    /// 追踪未来
    pub struct TracingFuture<F> {
        #[pin]
        inner: F,
        span: tracing::Span,
    }
}

impl<F, Res, E> Future for TracingFuture<F>
where
    F: Future<Output = Result<Res, E>>,
{
    type Output = Result<Res, E>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.project();
        let _enter = this.span.enter();
        info!("Request started");

        match this.inner.poll(cx) {
            Poll::Ready(result) => {
                info!("Request completed");
                Poll::Ready(result)
            }
            Poll::Pending => Poll::Pending,
        }
    }
}
