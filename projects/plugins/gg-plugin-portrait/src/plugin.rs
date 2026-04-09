//! 立绘系统插件模块
//! 实现 Plugin trait，负责立绘系统的初始化和关闭

use gg_core::plugin::Plugin;
use gg_core::GResult;

/// 立绘系统插件
///
/// 负责初始化立绘系统所需的资源，
/// 并在关闭时清理这些资源。
pub struct PortraitPlugin;

impl Plugin for PortraitPlugin {
    /// 返回插件名称
    fn name(&self) -> &str {
        "portrait"
    }

    /// 初始化立绘系统资源
    fn initialize(&self) -> GResult<()> {
        Ok(())
    }

    /// 关闭立绘系统，清理资源
    fn shutdown(&self) -> GResult<()> {
        Ok(())
    }
}
