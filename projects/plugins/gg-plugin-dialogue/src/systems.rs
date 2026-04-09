//! 对话系统和选项系统模块
//! 实现 DialogueSystem 和 ChoiceSystem，分别负责对话节点执行和选项处理

use gg_core::{GError, GErrorKind, GResult};
use gg_ecs::{System, World};
use gg_galgame_schema::components::{ChoiceState, DialogueNode};
use gg_galgame_schema::resources::{DialogueHistory, GameVariables, HistoryEntry};

use crate::commands::CommandDispatcher;

/// 对话系统
///
/// 负责执行对话节点的核心逻辑：
/// - 获取当前对话节点
/// - 处理节点中的命令列表
/// - 根据节点是否有选项设置 ChoiceState
/// - 推进到下一节点或记录对话历史
pub struct DialogueSystem {
    /// 命令分发器
    pub dispatcher: CommandDispatcher,
}

impl DialogueSystem {
    /// 创建新的对话系统
    pub fn new() -> Self {
        Self {
            dispatcher: CommandDispatcher::new(),
        }
    }
}

impl System for DialogueSystem {
    /// 返回系统名称
    fn name(&self) -> &str {
        "dialogue"
    }

    /// 执行对话系统逻辑
    ///
    /// 执行流程：
    /// 1. 获取当前 DialogueHistory 中的 current_node_id
    /// 2. 在 World 中查找对应 DialogueNode 实体
    /// 3. 处理该节点的 commands 列表（调用命令处理器）
    /// 4. 如果节点有 choices，设置 ChoiceState 为激活状态
    /// 5. 如果节点无 choices 但有 next_node_id，推进到下一节点
    /// 6. 将已完成的对话加入 DialogueHistory
    fn execute(&mut self, world: &mut World) -> GResult<()> {
        let current_node_id = {
            let history = world
                .get_component_mut::<DialogueHistory>(0)
                .ok_or_else(|| GError {
                    kind: GErrorKind::Ecs,
                    message: "DialogueHistory resource not found".to_string(),
                })?;
            history.current_node_id.clone()
        };

        let current_id = match current_node_id {
            Some(id) => id,
            None => return Ok(()),
        };

        let (speaker_id, text, commands, choices, next_node_id) = {
            let mut found_node: Option<DialogueNode> = None;
            for &entity in world.entities().iter() {
                if let Some(node) = world.get_component::<DialogueNode>(entity) {
                    if node.id == current_id {
                        found_node = Some(node.clone());
                        break;
                    }
                }
            }
            let node = found_node.ok_or_else(|| GError {
                kind: GErrorKind::Ecs,
                message: format!("DialogueNode '{}' not found", current_id),
            })?;
            (
                node.speaker_id.clone(),
                node.text.clone(),
                node.commands.clone(),
                node.choices.clone(),
                node.next_node_id.clone(),
            )
        };

        for command in &commands {
            self.dispatcher.dispatch(command, world)?;
        }

        if !choices.is_empty() {
            let choice_state = world
                .get_component_mut::<ChoiceState>(0)
                .ok_or_else(|| GError {
                    kind: GErrorKind::Ecs,
                    message: "ChoiceState component not found".to_string(),
                })?;
            choice_state.choices = choices;
            choice_state.selected_index = None;
            choice_state.is_active = true;
        } else if let Some(next_id) = next_node_id {
            let history = world
                .get_component_mut::<DialogueHistory>(0)
                .ok_or_else(|| GError {
                    kind: GErrorKind::Ecs,
                    message: "DialogueHistory resource not found".to_string(),
                })?;
            history.current_node_id = Some(next_id);
        }

        let history = world
            .get_component_mut::<DialogueHistory>(0)
            .ok_or_else(|| GError {
                kind: GErrorKind::Ecs,
                message: "DialogueHistory resource not found".to_string(),
            })?;
        let speaker_name = speaker_id;
        history.entries.push(HistoryEntry {
            speaker_name,
            text,
            timestamp: 0.0,
        });

        Ok(())
    }
}

/// 选项系统
///
/// 负责处理玩家选择选项的逻辑：
/// - 查找所有 ChoiceState 组件
/// - 如果 selected_index 有值，获取对应 Choice
/// - 评估 Choice 的 condition（通过 GameVariables.evaluate_condition）
/// - 推进到 next_node_id
pub struct ChoiceSystem;

impl ChoiceSystem {
    /// 创建新的选项系统
    pub fn new() -> Self {
        Self
    }
}

impl System for ChoiceSystem {
    /// 返回系统名称
    fn name(&self) -> &str {
        "choice"
    }

    /// 执行选项系统逻辑
    ///
    /// 执行流程：
    /// 1. 查找所有 ChoiceState 组件
    /// 2. 如果 selected_index 有值，获取对应 Choice
    /// 3. 评估 Choice 的 condition（通过 GameVariables.evaluate_condition）
    /// 4. 推进到 next_node_id
    fn execute(&mut self, world: &mut World) -> GResult<()> {
        let (selected_index, choices, _is_active) = {
            let choice_state = match world.get_component_mut::<ChoiceState>(0) {
                Some(cs) => cs,
                None => return Ok(()),
            };
            if !choice_state.is_active {
                return Ok(());
            }
            let idx = match choice_state.selected_index {
                Some(i) => i,
                None => return Ok(()),
            };
            (idx, choice_state.choices.clone(), choice_state.is_active)
        };

        if selected_index >= choices.len() {
            return Err(GError {
                kind: GErrorKind::Ecs,
                message: format!(
                    "Choice index {} out of bounds (len: {})",
                    selected_index,
                    choices.len()
                ),
            });
        }

        let choice = &choices[selected_index];

        if let Some(ref condition) = choice.condition {
            let variables = world
                .get_component_mut::<GameVariables>(0)
                .ok_or_else(|| GError {
                    kind: GErrorKind::Ecs,
                    message: "GameVariables resource not found".to_string(),
                })?;
            if !variables.evaluate_condition(condition) {
                return Err(GError {
                    kind: GErrorKind::Plugin,
                    message: format!("Condition '{}' not satisfied", condition),
                });
            }
        }

        let next_node_id = choice.next_node_id.clone();
        {
            let choice_state = world
                .get_component_mut::<ChoiceState>(0)
                .ok_or_else(|| GError {
                    kind: GErrorKind::Ecs,
                    message: "ChoiceState component not found".to_string(),
                })?;
            choice_state.is_active = false;
            choice_state.selected_index = None;
            choice_state.choices.clear();
        }

        let history = world
            .get_component_mut::<DialogueHistory>(0)
            .ok_or_else(|| GError {
                kind: GErrorKind::Ecs,
                message: "DialogueHistory resource not found".to_string(),
            })?;
        history.current_node_id = Some(next_node_id);

        Ok(())
    }
}
