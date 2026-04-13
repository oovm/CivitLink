#![warn(missing_docs)]

//! Pleroma 通用游戏引擎
//!
//! 提供基础的游戏引擎功能，不包含任何特化的游戏类型逻辑。

/// 引擎配置模块
pub mod config;

/// 引擎核心模块
pub mod engine;

/// 通用资源模块
pub mod resources;

/// 引擎初始化和启动
pub use engine::Engine;

/// 通用资源类型
pub use resources::*;
