//! HTTP 错误处理模块
//!
//! 提供统一的错误处理机制，包括：
//! - 错误响应格式化
//! - 错误扩展 trait

use http::{Response, StatusCode, header};
use serde::{Deserialize, Serialize};
use std::fmt;
use wae_types::{ErrorCategory, WaeError};

use crate::{Body, full_body};

/// HTTP 错误包装类型
///
/// 包装 WaeError 以提供 HTTP 响应转换。
#[derive(Debug, Clone)]
pub struct HttpError {
    /// 内部 WaeError
    inner: WaeError,
}

impl HttpError {
    /// 从 WaeError 创建 HttpError
    pub fn new(error: WaeError) -> Self {
        Self { inner: error }
    }

    /// 获取内部 WaeError 引用
    pub fn inner(&self) -> &WaeError {
        &self.inner
    }

    /// 获取错误分类
    pub fn category(&self) -> ErrorCategory {
        self.inner.category()
    }

    /// 获取国际化键
    pub fn i18n_key(&self) -> &'static str {
        self.inner.i18n_key()
    }

    /// 获取国际化数据
    pub fn i18n_data(&self) -> serde_json::Value {
        self.inner.i18n_data()
    }

    /// 创建无效参数错误
    pub fn invalid_params(param: impl Into<String>, reason: impl Into<String>) -> Self {
        Self::new(WaeError::invalid_params(param, reason))
    }

    /// 创建无效令牌错误
    pub fn invalid_token(reason: impl Into<String>) -> Self {
        Self::new(WaeError::invalid_token(reason))
    }

    /// 创建令牌过期错误
    pub fn token_expired() -> Self {
        Self::new(WaeError::token_expired())
    }

    /// 创建禁止访问错误
    pub fn forbidden(resource: impl Into<String>) -> Self {
        Self::new(WaeError::forbidden(resource))
    }

    /// 创建权限拒绝错误
    pub fn permission_denied(action: impl Into<String>) -> Self {
        Self::new(WaeError::permission_denied(action))
    }

    /// 创建资源未找到错误
    pub fn not_found(resource_type: impl Into<String>, identifier: impl Into<String>) -> Self {
        Self::new(WaeError::not_found(resource_type, identifier))
    }

    /// 创建内部错误
    pub fn internal(reason: impl Into<String>) -> Self {
        Self::new(WaeError::internal(reason))
    }

    /// 创建无效格式错误
    pub fn invalid_format(field: impl Into<String>, expected: impl Into<String>) -> Self {
        Self::new(WaeError::invalid_format(field, expected))
    }

    /// 转换为 HTTP 响应
    pub fn into_response(self) -> Response<Body> {
        let status = category_to_status_code(self.category());
        let error_response = ErrorResponse::from_error(&self);
        let body = serde_json::to_string(&error_response).unwrap_or_else(|_| {
            r#"{"success":false,"code":"INTERNAL_ERROR","message":"Failed to serialize error"}"#.to_string()
        });

        Response::builder().status(status).header(header::CONTENT_TYPE, "application/json").body(full_body(body)).unwrap()
    }
}

impl fmt::Display for HttpError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.inner)
    }
}

impl std::error::Error for HttpError {}

impl From<WaeError> for HttpError {
    fn from(error: WaeError) -> Self {
        Self::new(error)
    }
}

impl From<HttpError> for WaeError {
    fn from(error: HttpError) -> Self {
        error.inner
    }
}

/// HTTP 操作结果类型别名
pub type HttpResult<T> = Result<T, HttpError>;

/// 错误响应体结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorResponse {
    /// 是否成功
    pub success: bool,
    /// 错误码
    pub code: String,
    /// 错误消息
    pub message: String,
    /// 详细信息（可选）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<serde_json::Value>,
    /// 请求追踪 ID（可选）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trace_id: Option<String>,
}

impl ErrorResponse {
    /// 从 HttpError 创建错误响应
    pub fn from_error(error: &HttpError) -> Self {
        Self {
            success: false,
            code: error.i18n_key().to_string(),
            message: error.to_string(),
            details: Some(error.i18n_data()),
            trace_id: None,
        }
    }

    /// 从 WaeError 创建错误响应
    pub fn from_wae_error(error: &WaeError) -> Self {
        Self {
            success: false,
            code: error.i18n_key().to_string(),
            message: error.to_string(),
            details: Some(error.i18n_data()),
            trace_id: None,
        }
    }

    /// 从 HttpError 创建错误响应（带额外详细信息）
    pub fn from_error_with_details(error: &HttpError, details: serde_json::Value) -> Self {
        let mut base = Self::from_error(error);
        base.details = Some(details);
        base
    }

    /// 设置追踪 ID
    pub fn with_trace_id(mut self, trace_id: impl Into<String>) -> Self {
        self.trace_id = Some(trace_id.into());
        self
    }

    /// 转换为 HTTP 响应
    pub fn into_response(self) -> Response<Body> {
        let status = StatusCode::BAD_REQUEST;
        let body = serde_json::to_string(&self).unwrap_or_else(|_| {
            r#"{"success":false,"code":"INTERNAL_ERROR","message":"Failed to serialize error"}"#.to_string()
        });

        Response::builder().status(status).header(header::CONTENT_TYPE, "application/json").body(full_body(body)).unwrap()
    }
}

/// 将 ErrorCategory 转换为 StatusCode
fn category_to_status_code(category: ErrorCategory) -> StatusCode {
    match category {
        ErrorCategory::Validation => StatusCode::BAD_REQUEST,
        ErrorCategory::Auth => StatusCode::UNAUTHORIZED,
        ErrorCategory::Permission => StatusCode::FORBIDDEN,
        ErrorCategory::NotFound => StatusCode::NOT_FOUND,
        ErrorCategory::Conflict => StatusCode::CONFLICT,
        ErrorCategory::RateLimited => StatusCode::TOO_MANY_REQUESTS,
        ErrorCategory::Network => StatusCode::BAD_GATEWAY,
        ErrorCategory::Storage => StatusCode::INTERNAL_SERVER_ERROR,
        ErrorCategory::Database => StatusCode::INTERNAL_SERVER_ERROR,
        ErrorCategory::Cache => StatusCode::INTERNAL_SERVER_ERROR,
        ErrorCategory::Config => StatusCode::INTERNAL_SERVER_ERROR,
        ErrorCategory::Timeout => StatusCode::REQUEST_TIMEOUT,
        ErrorCategory::Internal => StatusCode::INTERNAL_SERVER_ERROR,
    }
}

/// 错误扩展 trait
///
/// 为 Result 类型提供便捷的错误转换方法。
pub trait ErrorExt<T> {
    /// 将错误转换为验证错误 (400)
    fn bad_request(self) -> HttpResult<T>;

    /// 将错误转换为认证错误 (401)
    fn unauthorized(self) -> HttpResult<T>;

    /// 将错误转换为权限错误 (403)
    fn forbidden(self) -> HttpResult<T>;

    /// 将错误转换为资源未找到错误 (404)
    fn not_found(self) -> HttpResult<T>;

    /// 将错误转换为内部服务器错误 (500)
    fn internal_error(self) -> HttpResult<T>;

    /// 使用自定义 HttpError 转换
    fn with_http_error(self, error: HttpError) -> HttpResult<T>;

    /// 使用错误消息转换函数
    fn map_http_error<F>(self, f: F) -> HttpResult<T>
    where
        F: FnOnce(String) -> HttpError;
}

impl<T, E: fmt::Display> ErrorExt<T> for Result<T, E> {
    fn bad_request(self) -> HttpResult<T> {
        self.map_err(|e| HttpError::invalid_params("unknown", e.to_string()))
    }

    fn unauthorized(self) -> HttpResult<T> {
        self.map_err(|e| HttpError::invalid_token(e.to_string()))
    }

    fn forbidden(self) -> HttpResult<T> {
        self.map_err(|e| HttpError::forbidden(e.to_string()))
    }

    fn not_found(self) -> HttpResult<T> {
        self.map_err(|e| HttpError::not_found("resource", e.to_string()))
    }

    fn internal_error(self) -> HttpResult<T> {
        self.map_err(|e| HttpError::internal(e.to_string()))
    }

    fn with_http_error(self, error: HttpError) -> HttpResult<T> {
        self.map_err(|_| error)
    }

    fn map_http_error<F>(self, f: F) -> HttpResult<T>
    where
        F: FnOnce(String) -> HttpError,
    {
        self.map_err(|e| f(e.to_string()))
    }
}

/// 创建成功响应的便捷函数
pub fn success_response<T: Serialize>(data: T) -> Response<Body> {
    let body = serde_json::to_string(&serde_json::json!({
        "success": true,
        "data": data
    }))
    .unwrap_or_default();

    Response::builder().status(StatusCode::OK).header(header::CONTENT_TYPE, "application/json").body(full_body(body)).unwrap()
}

/// 创建分页响应的便捷函数
pub fn paginated_response<T: Serialize>(items: Vec<T>, total: u64, page: u32, page_size: u32) -> Response<Body> {
    let total_pages = (total as f64 / page_size as f64).ceil() as u32;

    let body = serde_json::to_string(&serde_json::json!({
        "success": true,
        "data": {
            "items": items,
            "pagination": {
                "total": total,
                "page": page,
                "page_size": page_size,
                "total_pages": total_pages
            }
        }
    }))
    .unwrap_or_default();

    Response::builder().status(StatusCode::OK).header(header::CONTENT_TYPE, "application/json").body(full_body(body)).unwrap()
}
