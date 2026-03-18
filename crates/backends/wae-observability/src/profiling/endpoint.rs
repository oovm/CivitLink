//! Profiling HTTP 端点
//!
//! tokio-console 提供了自己的独立服务，此模块提供信息端点。

use http::{Response, StatusCode, header};
use http_body_util::Full;
use hyper::body::Bytes;
use serde::Serialize;

/// Profiling 服务信息响应
#[derive(Debug, Clone, Serialize)]
pub struct ProfilingInfo {
    /// 服务状态
    pub status: &'static str,
    /// 控制台地址
    pub console_addr: String,
    /// 使用说明
    pub instructions: &'static str,
}

/// Profiling 信息端点处理函数
///
/// 返回 tokio-console 服务的信息和使用说明。
///
/// # Arguments
///
/// * `console_addr` - tokio-console 服务地址
pub async fn profiling_info_handler(console_addr: String) -> Response<Full<Bytes>> {
    let info = ProfilingInfo {
        status: "running",
        console_addr,
        instructions: "Use `tokio-console <addr>` to connect to the console service",
    };

    let body = serde_json::to_vec(&info).unwrap_or_default();

    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "application/json")
        .body(Full::new(Bytes::from(body)))
        .unwrap()
}
