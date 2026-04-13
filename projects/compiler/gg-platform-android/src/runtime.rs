use gg_core::platform::{PlatformId, PlatformServices, RuntimePlatform};

use crate::services::AndroidPlatformServices;

/// Android 平台运行时实现
///
/// 为 Android 平台提供运行时环境的具体实现。
pub struct AndroidRuntimePlatform {
    /// 资源文件的基础路径
    base_path: String,
}

impl AndroidRuntimePlatform {
    /// 创建 Android 运行时平台实例
    pub fn new() -> Self {
        Self { base_path: String::new() }
    }

    /// 使用指定基础路径创建 Android 运行时平台实例
    pub fn with_base_path(base_path: impl Into<String>) -> Self {
        Self { base_path: base_path.into() }
    }
}

impl RuntimePlatform for AndroidRuntimePlatform {
    fn id(&self) -> PlatformId {
        "android".to_string()
    }

    fn display_name(&self) -> &str {
        "Android"
    }

    fn create_services(&self) -> PlatformServices {
        AndroidPlatformServices::create(&self.base_path)
    }
}
