//! 对话脚本加载器模块
//! 提供从 JSON 数据加载对话脚本到 World 的功能

use crate::schema::{DialogueHistory, DialogueScript, GameVariables};
use gg_core::{GError, GErrorKind, GResult};
use gg_ecs::World;

/// 对话脚本加载器
///
/// 从 JSON 数据加载对话脚本，创建 World 中的实体和资源。
pub struct DialogueScriptLoader;

impl DialogueScriptLoader {
    /// 从 JSON 字符串加载对话脚本到 World
    ///
    /// 解析 JSON 数据，创建 DialogueNode 和 CharacterDef 实体，
    /// 设置 GameVariables 初始值和 DialogueHistory 起始节点。
    ///
    /// # 参数
    ///
    /// - `world` - 游戏世界
    /// - `json` - JSON 格式的对话脚本数据
    pub fn load_from_json(world: &mut World, json: &str) -> GResult<()> {
        let script = DialogueScript::from_json(json)
            .map_err(|e| GError { kind: GErrorKind::Asset, message: format!("Failed to parse dialogue script: {}", e) })?;

        for node in script.nodes {
            let entity = world.spawn().id();
            world.add_component(entity, node)?;
        }

        for character in script.characters {
            let entity = world.spawn().id();
            world.add_component(entity, character)?;
        }

        if let Some(variables) = world.get_resource_mut::<GameVariables>() {
            for (name, value) in script.variables {
                variables.set_variable(name, value);
            }
        }

        if let Some(history) = world.get_resource_mut::<DialogueHistory>() {
            if history.current_node_id.is_none() {
                history.current_node_id = Some("start".to_string());
            }
        }

        Ok(())
    }
}
