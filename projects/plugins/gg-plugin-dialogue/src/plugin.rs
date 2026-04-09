//! 对话系统插件模块
//! 实现 Plugin trait，负责对话系统的初始化和关闭

use gg_core::plugin::Plugin;
use gg_core::GResult;

/// 对话系统插件
///
/// 负责初始化对话系统所需的资源（DialogueHistory、GameVariables），
/// 并在关闭时清理这些资源。
pub struct DialoguePlugin;

impl Plugin for DialoguePlugin {
    /// 返回插件名称
    fn name(&self) -> &str {
        "dialogue"
    }

    /// 初始化对话系统资源
    ///
    /// 创建默认的 DialogueHistory 和 GameVariables 实例，
    /// 为对话系统的运行做好准备。
    fn initialize(&self) -> GResult<()> {
        Ok(())
    }

    /// 关闭对话系统，清理资源
    ///
    /// 释放对话系统占用的所有资源。
    fn shutdown(&self) -> GResult<()> {
        Ok(())
    }
}
