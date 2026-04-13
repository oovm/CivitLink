//! 对话存档模块
//! 提供对话状态的快照和恢复功能

use std::collections::{HashMap, HashSet};

use serde::{Deserialize, Serialize};

use crate::schema::{AudioControl, DialogueHistory, GameState, GameVariables, PortraitState, SceneBackground, VariableValue};

/// 游戏状态快照
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameStateSnapshot {
    /// 游戏变量
    pub variables: HashMap<String, VariableValue>,
    /// 布尔标志集合
    pub flags: HashSet<String>,
    /// 当前天数
    pub day: i32,
    /// 当前时间
    pub time: f32,
    /// 游戏是否结束
    pub game_over: bool,
}

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
    /// 游戏状态快照
    pub game_state: Option<GameStateSnapshot>,
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

        let game_state = world.get_resource::<GameState>().map(|gs| GameStateSnapshot {
            variables: gs.variables.clone(),
            flags: gs.flags.clone(),
            day: gs.day,
            time: gs.time,
            game_over: gs.game_over,
        });

        Self { current_node_id, variables, portrait_states, background_path, bgm_path, game_state }
    }

    /// 将存档状态恢复到 World
    ///
    /// 恢复对话历史节点 ID、游戏变量和游戏状态。
    /// 立绘、背景和音频的恢复需要更高层级的实体管理支持。
    pub fn restore(self, world: &mut gg_ecs::World) {
        if let Some(history) = world.get_resource_mut::<DialogueHistory>() {
            history.current_node_id = self.current_node_id;
        }

        if let Some(variables) = world.get_resource_mut::<GameVariables>() {
            variables.variables = self.variables;
        }

        if let Some(snapshot) = self.game_state {
            if let Some(gs) = world.get_resource_mut::<GameState>() {
                gs.variables = snapshot.variables;
                gs.flags = snapshot.flags;
                gs.day = snapshot.day;
                gs.time = snapshot.time;
                gs.game_over = snapshot.game_over;
            }
        }
    }
}
