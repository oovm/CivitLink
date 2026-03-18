//! OpenTelemetry 分布式追踪模块
//!
//! 提供完整的 OpenTelemetry 追踪能力，包括 OTLP 导出、Trace Context 提取和中间件集成。

#[cfg(feature = "otlp")]
mod otlp;

#[cfg(feature = "otlp")]
mod propagation;

#[cfg(feature = "otlp")]
mod middleware;

#[cfg(feature = "otlp")]
pub use otlp::*;

#[cfg(feature = "otlp")]
pub use propagation::*;

#[cfg(feature = "otlp")]
pub use middleware::*;
