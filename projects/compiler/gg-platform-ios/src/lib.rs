#![warn(missing_docs)]

//! GG 引擎 iOS 平台实现
//!
//! 为 iOS 提供完整的平台架构实现，
//! 包括文件系统、输入、时间、窗口、线程和构建时支持。

/// iOS 平台文件系统实现
pub mod fs;
/// iOS 平台输入实现
pub mod input;
/// iOS 平台生命周期管理
pub mod lifecycle;
/// iOS 平台生命周期管理器
pub mod lifecycle_manager;
/// iOS 平台构建时实现
pub mod platform;
/// iOS 平台运行时实现
pub mod runtime;
/// iOS 平台服务工厂
pub mod services;
/// iOS 平台线程实现
pub mod thread;
/// iOS 平台时间实现
pub mod time;
/// iOS 平台窗口实现
pub mod window;

pub use fs::IOSFileSystem;
pub use input::IOSInputImpl;
pub use lifecycle::IOSLifecycle;
pub use lifecycle_manager::{IOSLifecycleManager, LifecycleState};
pub use platform::IOSPlatform;
pub use runtime::IOSRuntimePlatform;
pub use services::IOSPlatformServices;
pub use thread::IOSThread;
pub use time::IOSTime;
pub use window::IOSWindow;
