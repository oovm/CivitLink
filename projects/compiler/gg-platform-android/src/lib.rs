#![warn(missing_docs)]

//! GG 引擎 Android 平台实现
//!
//! 为 Android 提供完整的平台架构实现，
//! 包括文件系统、输入、时间、窗口、线程和构建时支持。

/// Android 平台文件系统实现
pub mod fs;
/// Android 平台输入实现
pub mod input;
/// Android 平台生命周期管理
pub mod lifecycle;
/// Android 平台生命周期管理器
pub mod lifecycle_manager;
/// Android 平台构建时实现
pub mod platform;
/// Android 平台运行时实现
pub mod runtime;
/// Android 平台服务工厂
pub mod services;
/// Android 平台线程实现
pub mod thread;
/// Android 平台时间实现
pub mod time;
/// Android 平台窗口实现
pub mod window;

pub use fs::AndroidFileSystem;
pub use input::AndroidInputImpl;
pub use lifecycle::AndroidLifecycle;
pub use lifecycle_manager::{AndroidLifecycleManager, LifecycleState};
pub use platform::AndroidPlatform;
pub use runtime::AndroidRuntimePlatform;
pub use services::AndroidPlatformServices;
pub use thread::AndroidThread;
pub use time::AndroidTime;
pub use window::AndroidWindow;
