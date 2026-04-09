//! 对话系统插件模块
//! 实现 Plugin trait，负责对话系统的初始化和关闭

use gg_core::plugin::{Plugin, PluginRegistrar};
use gg_core::GResult;
use gg_galgame_schema::resources::{DialogueHistory, GameVariables};

use crate::systems::{ChoiceSystem, DialogueSystem, TypewriterSystem, WaitSystem};

/// 对话系统插件
///
/// 负责初始化对话系统所需的资源（DialogueHistory、GameVariables），
/// 注册对话相关系统（DialogueSystem、ChoiceSystem、TypewriterSystem、WaitSystem），
/// 并在关闭时清理这些资源。
pub struct DialoguePlugin;

impl Plugin for DialoguePlugin {
    /// 返回插件名称
    fn name(&self) -> &str {
        "dialogue"
    }

    /// 构建对话系统插件
    ///
    /// 注册对话系统所需的全局资源和系统：
    /// - 资源：DialogueHistory、GameVariables
    /// - 系统：DialogueSystem、ChoiceSystem、TypewriterSystem、WaitSystem
    fn build(&self, registrar: &mut PluginRegistrar) {
        registrar.insert_resource(DialogueHistory {
            entries: Vec::new(),
            current_node_id: None,
        });
        registrar.insert_resource(GameVariables {
            variables: std::collections::HashMap::new(),
        });
        registrar.register_system(Box::new(DialogueSystem::new()));
        registrar.register_system(Box::new(ChoiceSystem::new()));
        registrar.register_system(Box::new(TypewriterSystem::new(1.0 / 60.0)));
        registrar.register_system(Box::new(WaitSystem::new(1.0 / 60.0)));
    }

    /// 返回插件依赖列表
    fn dependencies(&self) -> Vec<&str> {
        Vec::new()
    }

    /// 初始化对话系统资源
    fn initialize(&self) -> GResult<()> {
        Ok(())
    }

    /// 关闭对话系统，清理资源
    fn shutdown(&self) -> GResult<()> {
        Ok(())
    }
}
