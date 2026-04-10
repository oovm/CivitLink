#![warn(missing_docs)]

//! 移动平台运行时实现

use gg_core::platform::{PlatformId, PlatformServices, RuntimePlatform};

use crate::services::MobilePlatformServices;

/// 移动平台运行时实现
///
/// 为 iOS/Android 提供运行时平台服务。
pub struct MobileRuntimePlatform;

impl RuntimePlatform for MobileRuntimePlatform {
    /// 获取平台标识
    fn id(&self) -> PlatformId {
        "mobile".to_string()
    }

    /// 获取平台显示名称
    fn display_name(&self) -> &str {
        "Mobile (iOS/Android)"
    }

    /// 创建平台服务集合
    fn create_services(&self) -> PlatformServices {
        MobilePlatformServices::create()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gg_core::platform::RuntimePlatform;

    #[test]
    fn test_runtime_platform_id() {
        let platform = MobileRuntimePlatform;
        assert_eq!(platform.id(), "mobile");
    }

    #[test]
    fn test_runtime_platform_display_name() {
        let platform = MobileRuntimePlatform;
        assert_eq!(platform.display_name(), "Mobile (iOS/Android)");
    }

    #[test]
    fn test_runtime_platform_create_services() {
        let platform = MobileRuntimePlatform;
        let services = platform.create_services();
        assert_eq!(services.window.size(), (1280, 720));
    }
}
