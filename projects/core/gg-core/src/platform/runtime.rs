#![warn(missing_docs)]

//! 运行时平台统一接口
//! 提供运行时获取平台服务的统一入口

use super::{PlatformId, PlatformServices};

/// 运行时平台 trait
///
/// 作为运行时获取平台服务的统一入口。
/// 各平台实现此 trait 以提供平台特定的服务集合。
pub trait RuntimePlatform: Send + Sync + 'static {
    /// 获取平台标识
    fn id(&self) -> PlatformId;

    /// 获取平台显示名称
    fn display_name(&self) -> &str;

    /// 创建平台服务集合
    ///
    /// 根据当前平台创建并返回包含所有平台服务的 `PlatformServices` 实例。
    fn create_services(&self) -> PlatformServices;
}
