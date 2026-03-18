//! HTTP 路由模块
//!
//! 提供路由构建和管理工具。

use crate::Router;
use std::marker::PhantomData;

/// 路由构建器
pub struct RouterBuilder<S = ()> {
    /// 内部路由实例
    router: Router<S>,
    /// 状态类型标记
    _marker: PhantomData<S>,
}

impl<S> RouterBuilder<S>
where
    S: Clone + Send + Sync + 'static,
{
    /// 创建新的路由构建器
    pub fn new() -> Self
    where
        S: Default,
    {
        Self::from_router(Router::with_state(S::default()))
    }

    /// 从现有路由创建构建器
    pub fn from_router(router: Router<S>) -> Self {
        Self { router, _marker: PhantomData }
    }

    /// 嵌套路由
    pub fn nest<T>(mut self, prefix: &str, router: T) -> Self
    where
        T: Into<Router<S>>,
    {
        self.router = self.router.nest_service(prefix, router);
        self
    }

    /// 合并路由
    pub fn merge(mut self, other: Router<S>) -> Self {
        self.router = self.router.merge(other);
        self
    }

    /// 添加 GET 方法路由
    pub fn get<H, T>(mut self, path: &str, handler: H) -> Self
    where
        H: Fn(T) -> crate::Response<crate::Body> + Clone + Send + Sync + 'static,
        T: crate::extract::FromRequestParts<S, Error = crate::extract::ExtractorError> + 'static,
    {
        self.router.add_route(http::Method::GET, path, handler);
        self
    }

    /// 添加 POST 方法路由
    pub fn post<H, T>(mut self, path: &str, handler: H) -> Self
    where
        H: Fn(T) -> crate::Response<crate::Body> + Clone + Send + Sync + 'static,
        T: crate::extract::FromRequestParts<S, Error = crate::extract::ExtractorError> + 'static,
    {
        self.router.add_route(http::Method::POST, path, handler);
        self
    }

    /// 添加 PUT 方法路由
    pub fn put<H, T>(mut self, path: &str, handler: H) -> Self
    where
        H: Fn(T) -> crate::Response<crate::Body> + Clone + Send + Sync + 'static,
        T: crate::extract::FromRequestParts<S, Error = crate::extract::ExtractorError> + 'static,
    {
        self.router.add_route(http::Method::PUT, path, handler);
        self
    }

    /// 添加 DELETE 方法路由
    pub fn delete<H, T>(mut self, path: &str, handler: H) -> Self
    where
        H: Fn(T) -> crate::Response<crate::Body> + Clone + Send + Sync + 'static,
        T: crate::extract::FromRequestParts<S, Error = crate::extract::ExtractorError> + 'static,
    {
        self.router.add_route(http::Method::DELETE, path, handler);
        self
    }

    /// 添加 PATCH 方法路由
    pub fn patch<H, T>(mut self, path: &str, handler: H) -> Self
    where
        H: Fn(T) -> crate::Response<crate::Body> + Clone + Send + Sync + 'static,
        T: crate::extract::FromRequestParts<S, Error = crate::extract::ExtractorError> + 'static,
    {
        self.router.add_route(http::Method::PATCH, path, handler);
        self
    }

    /// 添加 OPTIONS 方法路由
    pub fn options<H, T>(mut self, path: &str, handler: H) -> Self
    where
        H: Fn(T) -> crate::Response<crate::Body> + Clone + Send + Sync + 'static,
        T: crate::extract::FromRequestParts<S, Error = crate::extract::ExtractorError> + 'static,
    {
        self.router.add_route(http::Method::OPTIONS, path, handler);
        self
    }

    /// 添加 HEAD 方法路由
    pub fn head<H, T>(mut self, path: &str, handler: H) -> Self
    where
        H: Fn(T) -> crate::Response<crate::Body> + Clone + Send + Sync + 'static,
        T: crate::extract::FromRequestParts<S, Error = crate::extract::ExtractorError> + 'static,
    {
        self.router.add_route(http::Method::HEAD, path, handler);
        self
    }

    /// 添加 TRACE 方法路由
    pub fn trace<H, T>(mut self, path: &str, handler: H) -> Self
    where
        H: Fn(T) -> crate::Response<crate::Body> + Clone + Send + Sync + 'static,
        T: crate::extract::FromRequestParts<S, Error = crate::extract::ExtractorError> + 'static,
    {
        self.router.add_route(http::Method::TRACE, path, handler);
        self
    }

    /// 构建路由
    pub fn build(self) -> Router<S> {
        self.router
    }

    /// 获取内部 Router 实例
    pub fn into_inner(self) -> Router<S> {
        self.build()
    }
}

impl<S> Default for RouterBuilder<S>
where
    S: Clone + Send + Sync + 'static + Default,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<S> From<Router<S>> for RouterBuilder<S>
where
    S: Clone + Send + Sync + 'static,
{
    fn from(router: Router<S>) -> Self {
        Self::from_router(router)
    }
}

impl<S> From<RouterBuilder<S>> for Router<S>
where
    S: Clone + Send + Sync + 'static,
{
    fn from(builder: RouterBuilder<S>) -> Self {
        builder.build()
    }
}

/// 方法路由类型
pub struct MethodRouter<S = ()> {
    /// 状态类型标记
    _marker: PhantomData<S>,
}

impl<S> Clone for MethodRouter<S> {
    fn clone(&self) -> Self {
        Self { _marker: PhantomData }
    }
}

impl<S> Default for MethodRouter<S> {
    fn default() -> Self {
        Self { _marker: PhantomData }
    }
}

/// GET 方法路由
pub fn get<H, T, S>(handler: H) -> impl FnOnce(&mut Router<S>, &str)
where
    H: Fn(T) -> crate::Response<crate::Body> + Clone + Send + Sync + 'static,
    T: crate::extract::FromRequestParts<S, Error = crate::extract::ExtractorError> + 'static,
    S: Clone + Send + Sync + 'static,
{
    move |router, path| {
        router.add_route(http::Method::GET, path, handler);
    }
}

/// POST 方法路由
pub fn post<H, T, S>(handler: H) -> impl FnOnce(&mut Router<S>, &str)
where
    H: Fn(T) -> crate::Response<crate::Body> + Clone + Send + Sync + 'static,
    T: crate::extract::FromRequestParts<S, Error = crate::extract::ExtractorError> + 'static,
    S: Clone + Send + Sync + 'static,
{
    move |router, path| {
        router.add_route(http::Method::POST, path, handler);
    }
}

/// PUT 方法路由
pub fn put<H, T, S>(handler: H) -> impl FnOnce(&mut Router<S>, &str)
where
    H: Fn(T) -> crate::Response<crate::Body> + Clone + Send + Sync + 'static,
    T: crate::extract::FromRequestParts<S, Error = crate::extract::ExtractorError> + 'static,
    S: Clone + Send + Sync + 'static,
{
    move |router, path| {
        router.add_route(http::Method::PUT, path, handler);
    }
}

/// DELETE 方法路由
pub fn delete<H, T, S>(handler: H) -> impl FnOnce(&mut Router<S>, &str)
where
    H: Fn(T) -> crate::Response<crate::Body> + Clone + Send + Sync + 'static,
    T: crate::extract::FromRequestParts<S, Error = crate::extract::ExtractorError> + 'static,
    S: Clone + Send + Sync + 'static,
{
    move |router, path| {
        router.add_route(http::Method::DELETE, path, handler);
    }
}

/// PATCH 方法路由
pub fn patch<H, T, S>(handler: H) -> impl FnOnce(&mut Router<S>, &str)
where
    H: Fn(T) -> crate::Response<crate::Body> + Clone + Send + Sync + 'static,
    T: crate::extract::FromRequestParts<S, Error = crate::extract::ExtractorError> + 'static,
    S: Clone + Send + Sync + 'static,
{
    move |router, path| {
        router.add_route(http::Method::PATCH, path, handler);
    }
}

/// OPTIONS 方法路由
pub fn options<H, T, S>(handler: H) -> impl FnOnce(&mut Router<S>, &str)
where
    H: Fn(T) -> crate::Response<crate::Body> + Clone + Send + Sync + 'static,
    T: crate::extract::FromRequestParts<S, Error = crate::extract::ExtractorError> + 'static,
    S: Clone + Send + Sync + 'static,
{
    move |router, path| {
        router.add_route(http::Method::OPTIONS, path, handler);
    }
}

/// HEAD 方法路由
pub fn head<H, T, S>(handler: H) -> impl FnOnce(&mut Router<S>, &str)
where
    H: Fn(T) -> crate::Response<crate::Body> + Clone + Send + Sync + 'static,
    T: crate::extract::FromRequestParts<S, Error = crate::extract::ExtractorError> + 'static,
    S: Clone + Send + Sync + 'static,
{
    move |router, path| {
        router.add_route(http::Method::HEAD, path, handler);
    }
}

/// TRACE 方法路由
pub fn trace<H, T, S>(handler: H) -> impl FnOnce(&mut Router<S>, &str)
where
    H: Fn(T) -> crate::Response<crate::Body> + Clone + Send + Sync + 'static,
    T: crate::extract::FromRequestParts<S, Error = crate::extract::ExtractorError> + 'static,
    S: Clone + Send + Sync + 'static,
{
    move |router, path| {
        router.add_route(http::Method::TRACE, path, handler);
    }
}

/// 健康检查路由
pub fn health_check_router() -> Router {
    Router::new()
}
