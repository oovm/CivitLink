use std::sync::Arc;

use gg_core::platform::PlatformServices;

use crate::{DesktopFileSystem, DesktopInput, DesktopThread, DesktopTime, DesktopWindow};
use gg_core::platform::WindowConfig;

/// 桌面平台服务工厂
///
/// 创建桌面平台的 `PlatformServices` 实例。
pub struct DesktopPlatformServices;

impl DesktopPlatformServices {
    /// 创建桌面平台的平台服务
    pub fn create() -> PlatformServices {
        Self::create_with_config(WindowConfig::default())
    }

    /// 使用指定窗口配置创建桌面平台的平台服务
    pub fn create_with_config(window_config: WindowConfig) -> PlatformServices {
        PlatformServices::new(
            Box::new(DesktopFileSystem::new()),
            Box::new(DesktopInput::new()),
            Box::new(DesktopTime::new()),
            Box::new(DesktopWindow::new(window_config)),
            Box::new(DesktopThread::new()),
        )
    }

    /// 使用 winit 窗口创建桌面平台的平台服务
    pub fn create_with_winit_window(
        window: Arc<winit::window::Window>,
        window_config: WindowConfig,
    ) -> PlatformServices {
        PlatformServices::new(
            Box::new(DesktopFileSystem::new()),
            Box::new(DesktopInput::new()),
            Box::new(DesktopTime::new()),
            Box::new(DesktopWindow::from_winit_window(window, window_config)),
            Box::new(DesktopThread::new()),
        )
    }
}
