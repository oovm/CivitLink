use gg_core::platform::{PlatformServices, WindowConfig};

use crate::{MobileFileSystem, MobileInputImpl, MobileThread, MobileTime, MobileWindow};

/// 移动平台服务工厂
///
/// 创建移动平台的 `PlatformServices` 实例。
pub struct MobilePlatformServices;

impl MobilePlatformServices {
    /// 创建移动平台的平台服务
    pub fn create() -> PlatformServices {
        Self::create_with_config(WindowConfig::default())
    }

    /// 使用指定窗口配置创建移动平台的平台服务
    pub fn create_with_config(window_config: WindowConfig) -> PlatformServices {
        PlatformServices::new(
            Box::new(MobileFileSystem::new()),
            Box::new(MobileInputImpl::new()),
            Box::new(MobileTime::new()),
            Box::new(MobileWindow::new(window_config)),
            Box::new(MobileThread::new()),
        )
    }
}
