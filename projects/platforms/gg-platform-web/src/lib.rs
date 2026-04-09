#![warn(missing_docs)]

//! GG 引擎 Web 平台实现
//!
//! 为 WebAssembly/Web 环境提供平台抽象的具体实现。

/// Web 平台文件系统实现
pub mod fs;
/// Web 平台输入实现
pub mod input;
/// Web 平台服务工厂
pub mod services;
/// Web 平台时间实现
pub mod time;

pub use fs::WebFileSystem;
pub use input::WebInput;
pub use services::WebPlatformServices;
pub use time::WebTime;
