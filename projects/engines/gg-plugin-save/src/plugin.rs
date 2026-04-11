//! 存档系统插件模块
//! 实现 Plugin trait，负责存档系统的初始化和关闭

use gg_core::{
    GResult,
    plugin::{Plugin, PluginRegistrar},
};

use crate::{systems::SaveSystem, ui::AutoSaveIndicator};

/// 存档系统插件
///
/// 负责初始化存档系统所需的系统（SaveSystem），
/// 注册自动存档指示器资源，
/// 并在关闭时清理这些资源。
pub struct SavePlugin;

impl Plugin for SavePlugin {
    /// 返回插件名称
    fn name(&self) -> &str {
        "save"
    }

    /// 构建存档系统插件
    ///
    /// 注册存档系统和自动存档指示器资源。
    fn build(&self, registrar: &mut PluginRegistrar) {
        registrar.register_system(Box::new(SaveSystem::new()));
        registrar.insert_resource(AutoSaveIndicator::new());
    }

    /// 返回插件依赖列表
    fn dependencies(&self) -> Vec<&str> {
        Vec::new()
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
