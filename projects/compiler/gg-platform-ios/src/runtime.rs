use gg_core::platform::{PlatformId, PlatformServices, RuntimePlatform};

use crate::services::IOSPlatformServices;

/// iOS 平台运行时实现
///
/// 为 iOS 平台提供运行时环境的具体实现。
pub struct IOSRuntimePlatform {
    /// 资源文件的基础路径
    base_path: String,
}

impl IOSRuntimePlatform {
    /// 创建 iOS 运行时平台实例
    pub fn new() -> Self {
        Self { base_path: String::new() }
    }

    /// 使用指定基础路径创建 iOS 运行时平台实例
    pub fn with_base_path(base_path: impl Into<String>) -> Self {
        Self { base_path: base_path.into() }
    }
}

impl RuntimePlatform for IOSRuntimePlatform {
    fn id(&self) -> PlatformId {
        "ios".to_string()
    }

    fn display_name(&self) -> &str {
        "iOS"
    }

    fn create_services(&self) -> PlatformServices {
        IOSPlatformServices::create(&self.base_path)
    }
}
