#![warn(missing_docs)]

//! GG 引擎资产管辖系统模块
//! 提供资产注册表、依赖图、增量构建器、资产缓存和统一管线，
//! 统一管理所有资源格式的编译和依赖

pub mod cache;
pub mod compilers;
pub mod dependency_graph;
pub mod error;
pub mod incremental;
pub mod pipeline;
pub mod prelude;
pub mod registry;
pub mod transformer;
pub mod types;
