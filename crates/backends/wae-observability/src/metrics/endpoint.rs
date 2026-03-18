//! 指标导出端点
//!
//! 提供 /metrics HTTP 端点，用于导出 Prometheus 格式的指标数据。

use crate::metrics::MetricsRegistry;
use http::{Response, StatusCode, header};
use http_body_util::Full;
use hyper::body::Bytes;
use std::sync::Arc;

/// Metrics 端点处理函数
///
/// 导出 Prometheus 格式的指标数据。
///
/// # Arguments
///
/// * `registry` - 指标注册器
pub async fn metrics_handler(registry: Arc<MetricsRegistry>) -> Response<Full<Bytes>> {
    let buffer = registry.export().into_bytes();

    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "text/plain; version=0.0.4")
        .body(Full::new(Bytes::from(buffer)))
        .unwrap()
}
