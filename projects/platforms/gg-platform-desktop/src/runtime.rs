#![warn(missing_docs)]

//! 桌面平台运行时实现

use gg_core::platform::{PlatformId, PlatformServices, RuntimePlatform};

use crate::services::DesktopPlatformServices;

/// 桌面平台运行时实现
///
/// 为 Windows、macOS、Linux 提供运行时平台服务。
pub struct DesktopRuntimePlatform;

impl RuntimePlatform for DesktopRuntimePlatform {
    /// 获取平台标识
    fn id(&self) -> PlatformId {
        "desktop".to_string()
    }

    /// 获取平台显示名称
    fn display_name(&self) -> &str {
        "Desktop (Windows/macOS/Linux)"
    }

    /// 创建平台服务集合
    fn create_services(&self) -> PlatformServices {
        DesktopPlatformServices::create()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gg_core::platform::RuntimePlatform;

    #[test]
    fn test_runtime_platform_id() {
        let platform = DesktopRuntimePlatform;
        assert_eq!(platform.id(), "desktop");
    }

    #[test]
    fn test_runtime_platform_display_name() {
        let platform = DesktopRuntimePlatform;
        assert_eq!(platform.display_name(), "Desktop (Windows/macOS/Linux)");
    }

    #[test]
    fn test_runtime_platform_create_services() {
        let platform = DesktopRuntimePlatform;
        let services = platform.create_services();
        assert_eq!(services.window.size(), (1280, 720));
    }
}
