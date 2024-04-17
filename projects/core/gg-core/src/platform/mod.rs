#![warn(missing_docs)]

//! 平台抽象层
//! 提供跨平台的文件系统、输入、时间、窗口、线程和构建服务接口

pub mod build;
pub mod fs;
pub mod input;
pub mod runtime;
pub mod services;
pub mod thread;
pub mod time;
pub mod window;
pub mod window_manager;

pub use build::{
    BuildConfig, BuildProfile, DeviceInfo, EnvironmentReport, GenerateContext, PackageContext, Platform, RunContext, ToolStatus,
};
pub use fs::{DirEntry, FileMetadata, FileSystem, FileType};
pub use input::{GamepadAxis, GamepadButton, GamepadId, Input, InputEvent, KeyCode, KeyState, PointerAction, PointerButton};
pub use runtime::RuntimePlatform;
pub use services::PlatformServices;
pub use thread::RuntimeThread;
pub use time::Time;
pub use window::{Window, WindowConfig, WindowEvent, WindowId, WindowManager, WindowManagerEvent};
pub use window_manager::SingleWindowManager;

/// 平台标识
pub type PlatformId = String;
