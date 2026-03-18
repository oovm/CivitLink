//! HTTP 端点实现
//!
//! 提供健康检查的 HTTP 端点，包括存活检查和就绪检查

use crate::{HealthCheck, HealthStatus, health::HealthRegistry};
use http::{Response, StatusCode, header};
use http_body_util::Full;
use hyper::body::Bytes;
use serde::Serialize;
use std::sync::Arc;

/// 存活检查响应
#[derive(Debug, Clone, Serialize)]
pub struct LiveResponse {
    /// 整体健康状态
    pub overall: HealthStatus,
}

/// 就绪检查响应
#[derive(Debug, Clone, Serialize)]
pub struct ReadyResponse {
    /// 整体健康状态
    pub overall: HealthStatus,
    /// 各个检查项
    pub checks: Vec<HealthCheck>,
}

/// 存活检查端点处理器
///
/// 返回服务存活状态，仅返回基本状态
pub async fn live_handler(registry: Arc<HealthRegistry>) -> Response<Full<Bytes>> {
    let report = registry.check_liveness().await;
    let response = LiveResponse { overall: report.overall };
    let body = serde_json::to_vec(&response).unwrap_or_default();

    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "application/json")
        .body(Full::new(Bytes::from(body)))
        .unwrap()
}

/// 就绪检查端点处理器
///
/// 返回服务就绪状态，执行所有注册的检查
pub async fn ready_handler(registry: Arc<HealthRegistry>) -> Response<Full<Bytes>> {
    let report = registry.check_readiness().await;
    let response = ReadyResponse { overall: report.overall, checks: report.checks };
    let body = serde_json::to_vec(&response).unwrap_or_default();

    let status =
        if matches!(report.overall, HealthStatus::Critical) { StatusCode::SERVICE_UNAVAILABLE } else { StatusCode::OK };

    Response::builder()
        .status(status)
        .header(header::CONTENT_TYPE, "application/json")
        .body(Full::new(Bytes::from(body)))
        .unwrap()
}
