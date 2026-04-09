#![warn(missing_docs)]

//! GG 引擎桌面平台实现
//!
//! 为 Windows、macOS、Linux 提供平台抽象的具体实现。

/// 桌面平台文件系统实现
pub mod fs;
/// 桌面平台输入实现
pub mod input;
/// 桌面平台服务工厂
pub mod services;
/// 桌面平台时间实现
pub mod time;

pub use fs::DesktopFileSystem;
pub use input::DesktopInput;
pub use services::DesktopPlatformServices;
pub use time::DesktopTime;
