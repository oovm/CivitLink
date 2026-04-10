#![warn(missing_docs)]

//! Web 平台运行时实现

use gg_core::platform::{PlatformId, PlatformServices, RuntimePlatform};

use crate::services::WebPlatformServices;

/// Web 平台运行时实现
///
/// 为 WebAssembly/Web 环境提供运行时平台服务。
pub struct WebRuntimePlatform {
    /// 资源文件的基础 URL 路径
    base_url: String,
}

impl WebRuntimePlatform {
    /// 创建新的 Web 运行时平台实例
    pub fn new(base_url: impl Into<String>) -> Self {
        Self { base_url: base_url.into() }
    }
}

impl RuntimePlatform for WebRuntimePlatform {
    fn id(&self) -> PlatformId {
        "web".to_string()
    }

    fn display_name(&self) -> &str {
        "Web (WebAssembly)"
    }

    fn create_services(&self) -> PlatformServices {
        WebPlatformServices::create(&self.base_url)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gg_core::platform::RuntimePlatform;

    #[test]
    fn test_runtime_platform_id() {
        let platform = WebRuntimePlatform::new("/assets");
        assert_eq!(platform.id(), "web");
    }

    #[test]
    fn test_runtime_platform_display_name() {
        let platform = WebRuntimePlatform::new("/assets");
        assert_eq!(platform.display_name(), "Web (WebAssembly)");
    }

    #[test]
    fn test_runtime_platform_create_services() {
        let platform = WebRuntimePlatform::new("/assets");
        let services = platform.create_services();
        assert_eq!(services.window.size(), (1280, 720));
    }
}
