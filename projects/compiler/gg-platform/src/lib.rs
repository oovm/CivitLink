#![warn(missing_docs)]

//! GG 引擎平台基础库
//!
//! 提供跨平台的通用接口和工具，为具体平台实现提供基础支持。

/// 平台通用构建配置管理
pub mod build_profile;
/// 跨平台构建协调器
pub mod coordinator;
/// 平台通用文件系统接口
pub mod fs;
/// 平台通用输入接口
pub mod input;
/// 平台通用构建时接口
pub mod platform;
/// 平台通用运行时接口
pub mod runtime;
/// 平台通用服务工厂
pub mod services;
/// 平台通用线程接口
pub mod thread;
/// 平台通用时间接口
pub mod time;
/// 平台通用工具函数
pub mod utils;
/// 平台通用窗口接口
pub mod window;

pub use fs::PlatformFileSystem;
pub use input::PlatformInput;
pub use platform::Platform;
pub use runtime::PlatformRuntime;
pub use services::PlatformServices;
pub use thread::PlatformThread;
pub use time::PlatformTime;
pub use window::PlatformWindow;
