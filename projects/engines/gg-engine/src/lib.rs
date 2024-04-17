#![warn(missing_docs)]

//! GG 通用引擎库
//! 提供基础的游戏引擎功能，不包含任何特化的游戏类型

/// 引擎配置模块
pub mod config;

/// 引擎核心模块
pub mod engine;

/// 角色模块
pub mod character;

/// 角色组件模块
pub use character::components;

/// 角色资源模块
pub use character::resources;

/// 引擎初始化和启动
pub use engine::Engine;

/// 角色相关组件和资源
pub use character::*;
