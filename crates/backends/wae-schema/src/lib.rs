//! WAE Schema - Schema 定义与验证模块
//!
//! 提供统一的 Schema 定义能力，支持：
//! - 数据结构 Schema 定义
//! - Schema 验证
//! - OpenAPI 文档生成
#![warn(missing_docs)]

/// OpenAPI 相关功能
pub mod openapi;
/// Schema 相关功能
pub mod schema;
/// Swagger UI 相关功能
pub mod swagger_ui;

// 重新导出所有公共类型
pub use openapi::*;
pub use schema::*;
