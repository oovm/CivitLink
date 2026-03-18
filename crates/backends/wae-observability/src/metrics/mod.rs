//! Prometheus 指标收集与导出模块
//!
//! 提供完整的 Prometheus 指标收集、管理和导出功能，包括 HTTP 指标中间件和 /metrics 端点。

#[cfg(feature = "metrics")]
mod registry;

#[cfg(feature = "metrics")]
pub use registry::*;

#[cfg(feature = "metrics")]
mod middleware;

#[cfg(feature = "metrics")]
pub use middleware::*;

#[cfg(feature = "metrics")]
mod endpoint;

#[cfg(feature = "metrics")]
pub use endpoint::*;
