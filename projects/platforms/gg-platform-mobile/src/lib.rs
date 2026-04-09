#![warn(missing_docs)]

//! GG 引擎移动平台实现
//!
//! 为 iOS/Android 提供移动平台特有的 trait 定义和架构设计。
//! 本模块为占位模块，仅定义接口，不包含具体实现。

/// 移动平台生命周期管理
pub mod lifecycle;
/// 移动平台输入扩展
pub mod mobile_input;

pub use lifecycle::MobileLifecycle;
pub use mobile_input::MobileInput;
