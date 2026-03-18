//! GWG Engine 平台抽象层
//!
//! 提供跨平台的统一接口。

pub mod filesystem;
pub mod input;
pub mod time;
pub mod window;

/// 平台类型
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PlatformType {
    /// Windows
    Windows,
    /// macOS
    MacOS,
    /// Linux
    Linux,
    /// iOS
    iOS,
    /// Android
    Android,
    /// H5 (Web)
    H5,
    /// 微信小游戏
    WechatMiniGame,
}

impl PlatformType {
    /// 获取当前平台类型
    pub fn current() -> Self {
        #[cfg(target_os = "windows")]
        {
            Self::Windows
        }
        #[cfg(target_os = "macos")]
        {
            Self::MacOS
        }
        #[cfg(target_os = "linux")]
        {
            Self::Linux
        }
        #[cfg(target_os = "ios")]
        {
            Self::iOS
        }
        #[cfg(target_os = "android")]
        {
            Self::Android
        }
        #[cfg(target_family = "wasm")]
        {
            Self::H5
        }
    }

    /// 检查是否是桌面平台
    pub fn is_desktop(&self) -> bool {
        matches!(self, Self::Windows | Self::MacOS | Self::Linux)
    }

    /// 检查是否是移动平台
    pub fn is_mobile(&self) -> bool {
        matches!(self, Self::iOS | Self::Android)
    }

    /// 检查是否是 Web 平台
    pub fn is_web(&self) -> bool {
        matches!(self, Self::H5 | Self::WechatMiniGame)
    }
}

/// 平台抽象 trait
pub trait Platform {
    /// 获取平台类型
    fn platform_type(&self) -> PlatformType;

    /// 初始化平台
    fn initialize(&mut self) -> PlatformResult<()>;

    /// 关闭平台
    fn shutdown(&mut self);

    /// 轮询事件
    fn poll_events(&mut self);

    /// 检查是否应该退出
    fn should_exit(&self) -> bool;
}

/// 平台结果
pub type PlatformResult<T> = Result<T, PlatformError>;

/// 平台错误
#[derive(thiserror::Error, Debug)]
pub enum PlatformError {
    /// 初始化失败
    #[error("Platform initialization failed: {0}")]
    InitializationFailed(String),
    
    /// IO 错误
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    
    /// 不支持的操作
    #[error("Unsupported operation: {0}")]
    UnsupportedOperation(String),
}

pub mod prelude {
    //! 平台抽象的预导入模块

    pub use super::{
        Platform, PlatformError, PlatformResult, PlatformType,
    };
    pub use super::filesystem::Filesystem;
    pub use super::input::Input;
    pub use super::time::Time;
    pub use super::window::Window;
}
