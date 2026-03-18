//! 健康检查模块
//!
//! 提供完整的健康检查功能，包括：
//! - 存活检查（liveness）
//! - 就绪检查（readiness）
//! - 健康检查器注册管理
//! - HTTP 端点集成

mod endpoint;
mod registry;

pub use endpoint::{LiveResponse, ReadyResponse, live_handler, ready_handler};
pub use registry::{CheckCategory, HealthRegistry};
