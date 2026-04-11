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


