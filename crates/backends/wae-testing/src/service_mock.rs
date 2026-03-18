//! 外部服务 Mock 工具模块
//!
//! 提供 MockExternalService 结构体，用于模拟外部 HTTP 服务，支持请求期望配置、调用记录和验证。

use crate::mock::{Mock, MockCall};
use bytes::Bytes;
use http::{HeaderMap, Method, StatusCode};
use parking_lot::RwLock;
use std::{collections::HashMap, sync::Arc};
use wae_types::{WaeError, WaeErrorKind, WaeResult as TestingResult};

/// 外部服务请求记录
#[derive(Debug, Clone)]
pub struct ServiceRequest {
    /// HTTP 方法
    pub method: Method,
    /// 请求路径
    pub path: String,
    /// 请求头
    pub headers: HeaderMap,
    /// 请求体 (可选)
    pub body: Option<Bytes>,
    /// 请求时间戳
    pub timestamp: std::time::Instant,
}

/// 外部服务响应
#[derive(Debug, Clone)]
pub struct ServiceResponse {
    /// 状态码
    pub status: StatusCode,
    /// 响应头
    pub headers: HeaderMap,
    /// 响应体
    pub body: Bytes,
}

impl ServiceResponse {
    /// 创建成功响应
    pub fn ok(body: impl Into<Bytes>) -> Self {
        Self { status: StatusCode::OK, headers: HeaderMap::new(), body: body.into() }
    }

    /// 创建错误响应
    pub fn error(status: StatusCode, body: impl Into<Bytes>) -> Self {
        Self { status, headers: HeaderMap::new(), body: body.into() }
    }

    /// 添加响应头
    pub fn header<K, V>(mut self, key: K, value: V) -> Self
    where
        K: TryInto<http::HeaderName>,
        V: TryInto<http::HeaderValue>,
    {
        if let (Ok(key), Ok(value)) = (key.try_into(), value.try_into()) {
            self.headers.append(key, value);
        }
        self
    }
}

/// 服务请求匹配规则
#[derive(Clone)]
pub enum ServiceMatchRule {
    /// 匹配指定路径
    Path(String),
    /// 匹配路径和方法
    PathAndMethod(String, Method),
    /// 自定义匹配函数
    Custom(Arc<dyn Fn(&ServiceRequest) -> bool + Send + Sync>),
}

impl std::fmt::Debug for ServiceMatchRule {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ServiceMatchRule::Path(path) => f.debug_tuple("Path").field(path).finish(),
            ServiceMatchRule::PathAndMethod(path, method) => f.debug_tuple("PathAndMethod").field(path).field(method).finish(),
            ServiceMatchRule::Custom(_) => f.debug_tuple("Custom").field(&"<function>").finish(),
        }
    }
}

/// 服务响应配置
#[derive(Debug, Clone)]
pub struct ServiceResponseConfig {
    /// 匹配规则
    pub match_rule: ServiceMatchRule,
    /// 响应内容
    pub response: ServiceResponse,
}

/// 外部服务期望配置
#[derive(Debug, Default)]
pub struct ServiceExpectation {
    /// 期望的请求次数映射 (路径 -> 期望次数)
    pub expected_requests: HashMap<String, usize>,
    /// 描述信息
    pub description: Option<String>,
}

impl ServiceExpectation {
    /// 创建新的服务期望配置
    pub fn new() -> Self {
        Self::default()
    }

    /// 设置期望请求次数
    pub fn expect_request(mut self, path: impl Into<String>, count: usize) -> Self {
        self.expected_requests.insert(path.into(), count);
        self
    }

    /// 设置描述
    pub fn description(mut self, desc: impl Into<String>) -> Self {
        self.description = Some(desc.into());
        self
    }
}

/// Mock 外部服务构建器
pub struct MockExternalServiceBuilder {
    responses: Vec<ServiceResponseConfig>,
    expectation: ServiceExpectation,
    requests: Arc<RwLock<Vec<ServiceRequest>>>,
}

impl MockExternalServiceBuilder {
    /// 创建新的 Mock 外部服务构建器
    pub fn new() -> Self {
        Self { responses: Vec::new(), expectation: ServiceExpectation::default(), requests: Arc::new(RwLock::new(Vec::new())) }
    }

    /// 添加响应配置 (匹配路径)
    pub fn respond_to_path(mut self, path: impl Into<String>, response: ServiceResponse) -> Self {
        self.responses.push(ServiceResponseConfig { match_rule: ServiceMatchRule::Path(path.into()), response });
        self
    }

    /// 添加响应配置 (匹配路径和方法)
    pub fn respond_to_path_and_method(mut self, path: impl Into<String>, method: Method, response: ServiceResponse) -> Self {
        self.responses
            .push(ServiceResponseConfig { match_rule: ServiceMatchRule::PathAndMethod(path.into(), method), response });
        self
    }

    /// 添加自定义响应配置
    pub fn respond_with<F>(mut self, matcher: F, response: ServiceResponse) -> Self
    where
        F: Fn(&ServiceRequest) -> bool + Send + Sync + 'static,
    {
        self.responses.push(ServiceResponseConfig { match_rule: ServiceMatchRule::Custom(Arc::new(matcher)), response });
        self
    }

    /// 设置期望
    pub fn expect(mut self, expectation: ServiceExpectation) -> Self {
        self.expectation = expectation;
        self
    }

    /// 构建 Mock 外部服务
    pub fn build(self) -> MockExternalService {
        MockExternalService { responses: self.responses, expectation: self.expectation, requests: self.requests }
    }
}

impl Default for MockExternalServiceBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Mock 外部服务
///
/// 用于模拟外部 HTTP 服务，支持请求期望配置、调用记录和验证。
pub struct MockExternalService {
    responses: Vec<ServiceResponseConfig>,
    expectation: ServiceExpectation,
    requests: Arc<RwLock<Vec<ServiceRequest>>>,
}

impl MockExternalService {
    /// 处理外部服务请求
    pub fn handle_request(
        &self,
        method: Method,
        path: String,
        headers: HeaderMap,
        body: Option<Bytes>,
    ) -> TestingResult<ServiceResponse> {
        let request = ServiceRequest {
            method: method.clone(),
            path: path.clone(),
            headers: headers.clone(),
            body: body.clone(),
            timestamp: std::time::Instant::now(),
        };

        {
            let mut requests = self.requests.write();
            requests.push(request.clone());
        }

        for config in &self.responses {
            if Self::matches(&request, &config.match_rule) {
                return Ok(config.response.clone());
            }
        }

        Err(WaeError::new(WaeErrorKind::MockError { reason: format!("No mock response configured for {} {}", method, path) }))
    }

    /// 异步处理外部服务请求
    pub async fn handle_request_async(
        &self,
        method: Method,
        path: String,
        headers: HeaderMap,
        body: Option<Bytes>,
    ) -> TestingResult<ServiceResponse> {
        self.handle_request(method, path, headers, body)
    }

    fn matches(request: &ServiceRequest, rule: &ServiceMatchRule) -> bool {
        match rule {
            ServiceMatchRule::Path(path) => request.path == *path,
            ServiceMatchRule::PathAndMethod(path, method) => request.path == *path && request.method == *method,
            ServiceMatchRule::Custom(matcher) => matcher(request),
        }
    }

    /// 获取请求记录
    pub fn requests(&self) -> Vec<ServiceRequest> {
        self.requests.read().clone()
    }

    /// 获取请求次数
    pub fn request_count(&self) -> usize {
        self.requests.read().len()
    }

    /// 获取指定路径的请求次数
    pub fn request_count_by_path(&self, path: &str) -> usize {
        self.requests.read().iter().filter(|r| r.path == path).count()
    }
}

impl Mock for MockExternalService {
    fn calls(&self) -> Vec<MockCall> {
        self.requests
            .read()
            .iter()
            .map(|r| MockCall { args: vec![r.method.to_string(), r.path.clone()], timestamp: r.timestamp })
            .collect()
    }

    fn call_count(&self) -> usize {
        self.request_count()
    }

    fn verify(&self) -> TestingResult<()> {
        for (path, expected) in &self.expectation.expected_requests {
            let actual = self.request_count_by_path(path);
            if actual != *expected {
                return Err(WaeError::new(WaeErrorKind::AssertionFailed {
                    message: format!("Expected {} requests for path '{}', but got {}", expected, path, actual),
                }));
            }
        }

        Ok(())
    }

    fn reset(&self) {
        let mut requests = self.requests.write();
        requests.clear();
    }
}
