#![warn(missing_docs)]

//! GG 引擎移动平台实现
//!
//! 为 iOS/Android 提供移动平台的完整架构实现，
//! 包括文件系统、输入、时间、窗口、线程和构建时支持。

/// 移动平台文件系统实现
pub mod fs;
/// 移动平台输入实现
pub mod input;
/// 移动平台生命周期管理
pub mod lifecycle;
/// 移动平台生命周期管理器
pub mod lifecycle_manager;
/// 移动平台输入扩展
pub mod mobile_input;
/// 移动平台构建时实现
pub mod platform;
/// 移动平台运行时实现
pub mod runtime;
/// 移动平台服务工厂
pub mod services;
/// 移动平台线程实现
pub mod thread;
/// 移动平台时间实现
pub mod time;
/// 移动平台窗口实现
pub mod window;

pub use fs::MobileFileSystem;
pub use input::MobileInputImpl;
pub use lifecycle::MobileLifecycle;
pub use lifecycle_manager::{LifecycleState, MobileLifecycleManager};
pub use mobile_input::MobileInput;
pub use platform::{MobilePlatform, MobileTarget};
pub use runtime::MobileRuntimePlatform;
pub use services::MobilePlatformServices;
pub use thread::MobileThread;
pub use time::MobileTime;
pub use window::MobileWindow;
