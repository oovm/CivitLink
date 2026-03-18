#![doc = include_str!("readme.md")]

mod compression;
mod cors;
mod request_id;
mod tracing;

pub use compression::{Compression, CompressionAlgorithm, CompressionBuilder, CompressionConfig};
pub use cors::{CorsConfig, cors_permissive, cors_strict};
pub use request_id::RequestIdLayer;
pub use tracing::TracingLayer;

pub use tower_http::cors::CorsLayer;

/// 重新导出 tower-http 的常用中间件
pub use tower_http::{
    catch_panic::CatchPanicLayer, normalize_path::NormalizePathLayer, request_id::MakeRequestUuid,
    set_header::SetRequestHeaderLayer, set_header::SetResponseHeaderLayer, timeout::TimeoutLayer,
};

use std::time::Duration;

/// 超时中间件构建器
pub struct TimeoutBuilder {
    timeout: Duration,
}

impl TimeoutBuilder {
    /// 创建新的超时构建器
    pub fn new() -> Self {
        Self { timeout: Duration::from_secs(30) }
    }

    /// 设置超时时间
    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// 构建超时中间件
    pub fn build(self) -> TimeoutLayer {
        TimeoutLayer::with_status_code(http::StatusCode::REQUEST_TIMEOUT, self.timeout)
    }
}

impl Default for TimeoutBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// 路径归一化配置
pub struct NormalizePathConfig;

impl NormalizePathConfig {
    /// 创建默认的路径归一化中间件
    #[allow(clippy::new_ret_no_self)]
    pub fn new() -> NormalizePathLayer {
        NormalizePathLayer::trim_trailing_slash()
    }
}
