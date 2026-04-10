#![warn(missing_docs)]

//! 平台抽象层
//! 提供跨平台的文件系统、输入、时间和构建服务接口

pub mod build;
pub mod fs;
pub mod input;
pub mod services;
pub mod time;

pub use build::{BuildConfig, GenerateContext, PackageContext, Platform, RunContext};
pub use fs::{DirEntry, FileMetadata, FileSystem, FileType};
pub use input::{Input, InputEvent, KeyCode, KeyState, PointerAction, PointerButton};
pub use services::PlatformServices;
pub use time::Time;

/// 平台标识
pub type PlatformId = String;
