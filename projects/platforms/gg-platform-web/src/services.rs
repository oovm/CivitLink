use gg_core::platform::PlatformServices;

use crate::{WebFileSystem, WebInput, WebTime};

/// Web 平台服务工厂
///
/// 创建 Web 平台的 `PlatformServices` 实例。
pub struct WebPlatformServices;

impl WebPlatformServices {
    /// 创建 Web 平台的平台服务
    ///
    /// # 参数
    ///
    /// - `base_url` - 资源文件的基础 URL 路径
    pub fn create(base_url: impl Into<String>) -> PlatformServices {
        PlatformServices::new(Box::new(WebFileSystem::new(base_url)), Box::new(WebInput::new()), Box::new(WebTime::new()))
    }
}
