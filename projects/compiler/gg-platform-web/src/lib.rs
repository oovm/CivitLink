#![warn(missing_docs)]
#![doc = include_str!("readme.md")]

/// Web 平台文件系统实现
pub mod fs;
/// Web 平台输入实现
pub mod input;
/// Web 平台构建时实现
pub mod platform;
/// Web 平台渲染后端适配
pub mod render;
/// Web 平台运行时实现
pub mod runtime;
/// Web 平台服务工厂
pub mod services;
/// Web 平台线程实现
pub mod thread;
/// Web 平台时间实现
pub mod time;
/// Web 平台窗口实现
pub mod window;

pub use fs::WebFileSystem;
pub use input::WebInput;
pub use platform::WebPlatform;
pub use render::{
    RenderBackendType, RenderCapability, RenderFallbackStrategy, detect_best_backend, detect_with_fallback,
    is_webgl2_available, is_webgpu_available,
};
pub use runtime::WebRuntimePlatform;
pub use services::WebPlatformServices;
pub use thread::WebThread;
pub use time::WebTime;
pub use window::WebWindow;
