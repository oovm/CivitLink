//! HTTP 响应模块
//!
//! 提供统一的响应处理工具。

use http::{Response, StatusCode, header};
use serde::Serialize;
use std::path::Path;

use crate::{ApiResponse, Body, full_body};

/// HTML 响应类型
pub struct Html<T>(pub T);

/// 重定向响应类型
pub struct Redirect;

/// 将类型转换为 HTTP 响应的 trait
pub trait IntoResponse {
    /// 将类型转换为 HTTP 响应
    fn into_response(self) -> Response<Body>;
}

/// JSON 响应构建器
pub struct JsonResponse;

impl JsonResponse {
    /// 返回成功响应
    pub fn success<T: Serialize>(data: T) -> Response<Body> {
        let api_response = ApiResponse::success(data);
        let status = if api_response.success { StatusCode::OK } else { StatusCode::BAD_REQUEST };
        let body = serde_json::to_string(&api_response).unwrap_or_default();
        Response::builder().status(status).header(header::CONTENT_TYPE, "application/json").body(full_body(body)).unwrap()
    }

    /// 返回错误响应
    pub fn error(code: impl Into<String>, message: impl Into<String>) -> Response<Body> {
        let api_response = ApiResponse::<()>::error(code, message);
        let status = if api_response.success { StatusCode::OK } else { StatusCode::BAD_REQUEST };
        let body = serde_json::to_string(&api_response).unwrap_or_default();
        Response::builder().status(status).header(header::CONTENT_TYPE, "application/json").body(full_body(body)).unwrap()
    }

    /// 返回 404 错误
    pub fn not_found(message: impl Into<String>) -> Response<Body> {
        let body = ApiResponse::<()>::error("NOT_FOUND", message);
        Response::builder()
            .status(StatusCode::NOT_FOUND)
            .header(header::CONTENT_TYPE, "application/json")
            .body(full_body(serde_json::to_string(&body).unwrap_or_default()))
            .unwrap()
    }

    /// 返回 500 错误
    pub fn internal_error(message: impl Into<String>) -> Response<Body> {
        let body = ApiResponse::<()>::error("INTERNAL_ERROR", message);
        Response::builder()
            .status(StatusCode::INTERNAL_SERVER_ERROR)
            .header(header::CONTENT_TYPE, "application/json")
            .body(full_body(serde_json::to_string(&body).unwrap_or_default()))
            .unwrap()
    }
}

/// 附件响应
///
/// 用于创建文件下载响应。
pub struct Attachment {
    filename: String,
    content_type: String,
    data: Vec<u8>,
}

impl Attachment {
    /// 创建新的附件响应
    pub fn new(filename: impl Into<String>, content_type: impl Into<String>, data: Vec<u8>) -> Self {
        Self { filename: filename.into(), content_type: content_type.into(), data }
    }

    /// 从文件路径创建附件响应
    pub async fn from_path(path: impl AsRef<Path>, filename: Option<impl Into<String>>) -> std::io::Result<Self> {
        let path = path.as_ref();
        let data = tokio::fs::read(path).await?;
        let filename = filename
            .map(|f| f.into())
            .unwrap_or_else(|| path.file_name().and_then(|n| n.to_str()).unwrap_or("download").to_string());
        let content_type = mime_guess::from_path(path).first_or_octet_stream().to_string();
        Ok(Self::new(filename, content_type, data))
    }
}

impl IntoResponse for Attachment {
    fn into_response(self) -> Response<Body> {
        let disposition = format!("attachment; filename=\"{}\"", self.filename);
        Response::builder()
            .status(StatusCode::OK)
            .header(header::CONTENT_TYPE, self.content_type)
            .header(header::CONTENT_DISPOSITION, disposition)
            .body(full_body(self.data))
            .unwrap()
    }
}

/// 流式响应
///
/// 用于创建流式数据响应。
pub struct StreamResponse {
    body: Body,
    content_type: String,
}

impl StreamResponse {
    /// 创建新的流式响应
    pub fn new(body: Body, content_type: impl Into<String>) -> Self {
        Self { body, content_type: content_type.into() }
    }
}

impl IntoResponse for StreamResponse {
    fn into_response(self) -> Response<Body> {
        Response::builder().status(StatusCode::OK).header(header::CONTENT_TYPE, self.content_type).body(self.body).unwrap()
    }
}
