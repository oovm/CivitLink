//! 代数效应系统
//!
//! 提供声明式的依赖注入能力，允许在处理请求时通过类型安全的接口获取各种依赖。

#![warn(missing_docs)]

use std::{any::TypeId, collections::HashMap, sync::Arc};

use http::{Response, StatusCode, request::Parts};
use wae_types::{WaeError, WaeResult};

/// 依赖作用域
///
/// 定义依赖的生命周期和可见范围。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Scope {
    /// 单例作用域
    ///
    /// 依赖在整个应用生命周期内只创建一次，所有请求共享同一个实例。
    Singleton,
    /// 请求作用域
    ///
    /// 每个请求都会创建一个新的依赖实例，仅在该请求范围内可见。
    RequestScoped,
}

impl Default for Scope {
    fn default() -> Self {
        Scope::Singleton
    }
}

/// 带作用域的服务包装器
struct ScopedService {
    scope: Scope,
    service: Box<dyn std::any::Any + Send + Sync>,
}

/// 依赖容器
///
/// 存储所有注册的服务实例。
#[derive(Default)]
pub struct Dependencies {
    services: HashMap<String, ScopedService>,
    typed_services: HashMap<TypeId, ScopedService>,
}

impl Dependencies {
    /// 创建新的依赖容器
    pub fn new() -> Self {
        Self { services: HashMap::new(), typed_services: HashMap::new() }
    }

    /// 注册服务（按字符串键，默认单例作用域）
    pub fn register<T: Send + Sync + 'static>(&mut self, name: &str, service: T) {
        self.register_with_scope(name, service, Scope::Singleton);
    }

    /// 按作用域注册服务（按字符串键）
    pub fn register_with_scope<T: Send + Sync + 'static>(&mut self, name: &str, service: T, scope: Scope) {
        self.services.insert(name.to_string(), ScopedService { scope, service: Box::new(service) });
    }

    /// 获取服务（按字符串键）
    pub fn get<T: Clone + Send + Sync + 'static>(&self, name: &str) -> WaeResult<T> {
        self.services
            .get(name)
            .and_then(|s| s.service.downcast_ref::<T>())
            .cloned()
            .ok_or_else(|| WaeError::not_found("Dependency", name))
    }

    /// 检查服务的作用域（按字符串键）
    pub fn get_scope(&self, name: &str) -> Option<Scope> {
        self.services.get(name).map(|s| s.scope)
    }

    /// 按类型注册服务（默认单例作用域）
    pub fn register_type<T: Clone + Send + Sync + 'static>(&mut self, service: T) {
        self.register_type_with_scope(service, Scope::Singleton);
    }

    /// 按作用域注册服务（按类型）
    pub fn register_type_with_scope<T: Clone + Send + Sync + 'static>(&mut self, service: T, scope: Scope) {
        self.typed_services.insert(TypeId::of::<T>(), ScopedService { scope, service: Box::new(service) });
    }

    /// 按类型获取服务
    pub fn get_type<T: Clone + Send + Sync + 'static>(&self) -> WaeResult<T> {
        self.typed_services
            .get(&TypeId::of::<T>())
            .and_then(|s| s.service.downcast_ref::<T>())
            .cloned()
            .ok_or_else(|| WaeError::not_found("Typed dependency", std::any::type_name::<T>()))
    }

    /// 检查服务的作用域（按类型）
    pub fn get_type_scope<T: 'static>(&self) -> Option<Scope> {
        self.typed_services.get(&TypeId::of::<T>()).map(|s| s.scope)
    }
}

/// 代数效应请求上下文
///
/// 包含依赖容器和请求信息，用于在请求处理过程中获取依赖。
pub struct Effectful {
    deps: Arc<Dependencies>,
    parts: Parts,
    request_scoped_services: HashMap<String, Box<dyn std::any::Any + Send + Sync>>,
    request_scoped_typed_services: HashMap<TypeId, Box<dyn std::any::Any + Send + Sync>>,
}

impl Effectful {
    /// 创建新的 Effectful
    pub fn new(deps: Arc<Dependencies>, parts: Parts) -> Self {
        Self { deps, parts, request_scoped_services: HashMap::new(), request_scoped_typed_services: HashMap::new() }
    }

    /// 获取依赖（按字符串键）
    ///
    /// 如果依赖是 RequestScoped 作用域，将从当前请求的独立存储中获取或初始化。
    pub fn get<T: Clone + Send + Sync + 'static>(&self, name: &str) -> WaeResult<T> {
        if let Some(scope) = self.deps.get_scope(name) {
            match scope {
                Scope::Singleton => self.deps.get(name),
                Scope::RequestScoped => {
                    if let Some(service) = self.request_scoped_services.get(name) {
                        service
                            .downcast_ref::<T>()
                            .cloned()
                            .ok_or_else(|| WaeError::not_found("Request-scoped dependency", name))
                    }
                    else {
                        self.deps.get(name)
                    }
                }
            }
        }
        else {
            self.deps.get(name)
        }
    }

    /// 设置请求作用域的依赖（按字符串键）
    ///
    /// 仅对 RequestScoped 作用域的依赖有效。
    pub fn set<T: Send + Sync + 'static>(&mut self, name: &str, service: T) -> WaeResult<()> {
        if let Some(Scope::RequestScoped) = self.deps.get_scope(name) {
            self.request_scoped_services.insert(name.to_string(), Box::new(service));
            Ok(())
        }
        else {
            Err(WaeError::invalid_params("dependency", "Can only set RequestScoped dependencies"))
        }
    }

    /// 按类型获取依赖
    ///
    /// 如果依赖是 RequestScoped 作用域，将从当前请求的独立存储中获取或初始化。
    pub fn get_type<T: Clone + Send + Sync + 'static>(&self) -> WaeResult<T> {
        if let Some(scope) = self.deps.get_type_scope::<T>() {
            match scope {
                Scope::Singleton => self.deps.get_type(),
                Scope::RequestScoped => {
                    if let Some(service) = self.request_scoped_typed_services.get(&TypeId::of::<T>()) {
                        service
                            .downcast_ref::<T>()
                            .cloned()
                            .ok_or_else(|| WaeError::not_found("Typed request-scoped dependency", std::any::type_name::<T>()))
                    }
                    else {
                        self.deps.get_type()
                    }
                }
            }
        }
        else {
            self.deps.get_type()
        }
    }

    /// 设置请求作用域的依赖（按类型）
    ///
    /// 仅对 RequestScoped 作用域的依赖有效。
    pub fn set_type<T: Clone + Send + Sync + 'static>(&mut self, service: T) -> WaeResult<()> {
        if let Some(Scope::RequestScoped) = self.deps.get_type_scope::<T>() {
            self.request_scoped_typed_services.insert(TypeId::of::<T>(), Box::new(service));
            Ok(())
        }
        else {
            Err(WaeError::invalid_params("dependency", "Can only set RequestScoped dependencies"))
        }
    }

    /// 获取请求头
    pub fn header(&self, name: &str) -> Option<&str> {
        self.parts.headers.get(name).and_then(|v| v.to_str().ok())
    }

    /// 获取请求 Parts 的引用
    pub fn parts(&self) -> &Parts {
        &self.parts
    }

    /// 按类型获取依赖（便捷方法）
    pub fn use_type<T: Clone + Send + Sync + 'static>(&self) -> WaeResult<T> {
        self.get_type()
    }

    /// 获取配置（按类型）
    pub fn use_config<T: Clone + Send + Sync + 'static>(&self) -> WaeResult<T> {
        self.get_type()
    }

    /// 获取认证服务（按类型）
    pub fn use_auth<T: Clone + Send + Sync + 'static>(&self) -> WaeResult<T> {
        self.get_type()
    }
}

/// 代数效应构建器
///
/// 用于构建依赖容器并注册各种依赖。
pub struct AlgebraicEffect {
    deps: Dependencies,
}

impl Default for AlgebraicEffect {
    fn default() -> Self {
        Self::new()
    }
}

impl AlgebraicEffect {
    /// 创建新的代数效应构建器
    pub fn new() -> Self {
        Self { deps: Dependencies::new() }
    }

    /// 注册服务（按字符串键，默认单例作用域）
    pub fn with<T: Send + Sync + 'static>(mut self, name: &str, service: T) -> Self {
        self.deps.register(name, service);
        self
    }

    /// 按作用域注册服务（按字符串键）
    pub fn with_scope<T: Send + Sync + 'static>(mut self, name: &str, service: T, scope: Scope) -> Self {
        self.deps.register_with_scope(name, service, scope);
        self
    }

    /// 按类型注册服务（默认单例作用域）
    pub fn with_type<T: Clone + Send + Sync + 'static>(mut self, service: T) -> Self {
        self.deps.register_type(service);
        self
    }

    /// 按作用域注册服务（按类型）
    pub fn with_type_scope<T: Clone + Send + Sync + 'static>(mut self, service: T, scope: Scope) -> Self {
        self.deps.register_type_with_scope(service, scope);
        self
    }

    /// 注册配置服务（按类型）
    pub fn with_config<T: Clone + Send + Sync + 'static>(mut self, config: T) -> Self {
        self.deps.register_type(config);
        self
    }

    /// 注册配置服务（按类型和作用域）
    pub fn with_config_scope<T: Clone + Send + Sync + 'static>(mut self, config: T, scope: Scope) -> Self {
        self.deps.register_type_with_scope(config, scope);
        self
    }

    /// 注册认证服务（按类型）
    pub fn with_auth<T: Clone + Send + Sync + 'static>(mut self, auth: T) -> Self {
        self.deps.register_type(auth);
        self
    }

    /// 注册认证服务（按类型和作用域）
    pub fn with_auth_scope<T: Clone + Send + Sync + 'static>(mut self, auth: T, scope: Scope) -> Self {
        self.deps.register_type_with_scope(auth, scope);
        self
    }

    /// 构建依赖容器
    pub fn build(self) -> Arc<Dependencies> {
        Arc::new(self.deps)
    }
}

/// WaeError 的包装类型，用于解决 orphan rule 问题
pub struct WaeErrorResponse(pub WaeError);

impl WaeErrorResponse {
    /// 将 WaeError 转换为 http::Response
    pub fn into_response<B>(self) -> Response<B>
    where
        B: From<String>,
    {
        let status = self.0.http_status();
        let body = B::from(self.0.to_string());
        let status_code = StatusCode::from_u16(status).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);
        Response::builder().status(status_code).body(body).unwrap()
    }
}
