use gg_core::platform::PlatformServices;

use crate::{fs::IOSFileSystem, input::IOSInputImpl, thread::IOSThread, time::IOSTime, window::IOSWindow};

/// iOS 平台服务工厂
///
/// 创建 iOS 平台的 PlatformServices 实例。
pub struct IOSPlatformServices;

impl IOSPlatformServices {
    /// 创建 iOS 平台的平台服务
    pub fn create(_base_path: &str) -> PlatformServices {
        PlatformServices::new(
            Box::new(IOSFileSystem::new()),
            Box::new(IOSInputImpl::new()),
            Box::new(IOSTime::new()),
            Box::new(IOSWindow::default()),
            Box::new(IOSThread::new()),
        )
    }
}
