use gg_core::platform::PlatformServices;

use crate::{DesktopFileSystem, DesktopInput, DesktopTime};

/// 桌面平台服务工厂
///
/// 创建桌面平台的 `PlatformServices` 实例。
pub struct DesktopPlatformServices;

impl DesktopPlatformServices {
    /// 创建桌面平台的平台服务
    pub fn create() -> PlatformServices {
        PlatformServices::new(Box::new(DesktopFileSystem::new()), Box::new(DesktopInput::new()), Box::new(DesktopTime::new()))
    }
}
