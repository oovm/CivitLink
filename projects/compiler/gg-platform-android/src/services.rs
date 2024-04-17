use gg_core::platform::PlatformServices;

use crate::{fs::AndroidFileSystem, input::AndroidInputImpl, thread::AndroidThread, time::AndroidTime, window::AndroidWindow};

/// Android 平台服务工厂
///
/// 创建 Android 平台的 PlatformServices 实例。
pub struct AndroidPlatformServices;

impl AndroidPlatformServices {
    /// 创建 Android 平台的平台服务
    pub fn create(_base_path: &str) -> PlatformServices {
        PlatformServices::new(
            Box::new(AndroidFileSystem::new()),
            Box::new(AndroidInputImpl::new()),
            Box::new(AndroidTime::new()),
            Box::new(AndroidWindow::default()),
            Box::new(AndroidThread::new()),
        )
    }
}
