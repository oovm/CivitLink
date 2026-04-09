//! 存档系统插件模块
//! 实现 Plugin trait，负责存档系统的初始化和关闭

use gg_core::plugin::Plugin;
use gg_core::GResult;

/// 存档系统插件
///
/// 负责初始化存档系统所需的资源，
/// 并在关闭时清理这些资源。
pub struct SavePlugin;

impl Plugin for SavePlugin {
    /// 返回插件名称
    fn name(&self) -> &str {
        "save"
    }

    /// 初始化存档系统资源
    fn initialize(&self) -> GResult<()> {
        Ok(())
    }

    /// 关闭存档系统，清理资源
    fn shutdown(&self) -> GResult<()> {
        Ok(())
    }
}
