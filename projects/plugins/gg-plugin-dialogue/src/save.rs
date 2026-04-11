//! 对话存档模块
//! 提供对话状态的快照和恢复功能

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::schema::{AudioControl, DialogueHistory, GameVariables, PortraitState, SceneBackground, VariableValue};

/// 对话存档状态
///
/// 保存对话系统的完整状态，可用于存档和读档。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DialogueSaveState {
    /// 当前对话节点 ID
    pub current_node_id: Option<String>,
    /// 游戏变量快照
    pub variables: HashMap<String, VariableValue>,
    /// 立绘状态快照
    pub portrait_states: Vec<PortraitState>,
    /// 当前背景路径
    pub background_path: Option<String>,
    /// 当前 BGM 路径
    pub bgm_path: Option<String>,
}

impl DialogueSaveState {
    /// 从 World 创建对话存档快照
    ///
    /// 提取当前对话历史节点 ID、游戏变量、立绘状态、背景和 BGM 信息。
    pub fn snapshot(world: &gg_ecs::World) -> Self {
        let current_node_id = world.get_resource::<DialogueHistory>().and_then(|h| h.current_node_id.clone());

        let variables = world.get_resource::<GameVariables>().map(|v| v.variables.clone()).unwrap_or_default();

        let portrait_states: Vec<PortraitState> = {
            let mut portraits = Vec::new();
            for entity in world.entities() {
                if let Some(portrait) = world.get_component::<PortraitState>(entity) {
                    portraits.push(portrait.clone());
                }
            }
            portraits
        };

        let background_path = world
            .entities()
            .iter()
            .find_map(|&entity| world.get_component::<SceneBackground>(entity).and_then(|bg| bg.asset_path.clone()));

        let bgm_path = world
            .entities()
            .iter()
            .find_map(|&entity| world.get_component::<AudioControl>(entity).and_then(|ac| ac.bgm_path.clone()));

        Self { current_node_id, variables, portrait_states, background_path, bgm_path }
    }

    /// 将存档状态恢复到 World
    ///
    /// 恢复对话历史节点 ID 和游戏变量。
    /// 立绘、背景和音频的恢复需要更高层级的实体管理支持。
    pub fn restore(self, world: &mut gg_ecs::World) {
        if let Some(history) = world.get_resource_mut::<DialogueHistory>() {
            history.current_node_id = self.current_node_id;
        }

        if let Some(variables) = world.get_resource_mut::<GameVariables>() {
            variables.variables = self.variables;
        }
    }
}
