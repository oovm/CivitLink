//! 对话系统、选项系统、打字机效果系统和等待系统模块
//! 实现 DialogueSystem、ChoiceSystem、TypewriterSystem 和 WaitSystem

use crate::schema::{ChoiceState, DialogueHistory, DialogueNode, GameVariables, HistoryEntry, WaitTimer};
use gg_core::{GError, GErrorKind, GResult};
use gg_ecs::{System, World};

use crate::{commands::CommandDispatcher, typewriter::TypewriterState};

/// 默认打字机显示速度（字符/秒）
const DEFAULT_TYPEWRITER_SPEED: f32 = 30.0;

/// 对话系统
///
/// 负责执行对话节点的核心逻辑：
/// - 获取当前对话节点
/// - 检查打字机效果是否完成，未完成则等待
/// - 检查等待计时器是否完成，未完成则等待
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
        Self { dispatcher: CommandDispatcher::new() }
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
    /// 1. 检查是否有活跃的打字机效果，未完成则提前返回
    /// 2. 检查是否有活跃的等待计时器，未完成则提前返回
    /// 3. 获取当前 DialogueHistory 中的 current_node_id
    /// 4. 在 World 中查找对应 DialogueNode 实体
    /// 5. 处理该节点的 commands 列表（调用命令处理器）
    /// 6. 如果节点有文本，创建 TypewriterState 资源
    /// 7. 如果节点有 choices，设置 ChoiceState 为激活状态
    /// 8. 如果节点无 choices 但有 next_node_id，推进到下一节点
    /// 9. 将已完成的对话加入 DialogueHistory
    fn execute(&mut self, world: &mut World) -> GResult<()> {
        if let Some(state) = world.get_resource::<TypewriterState>() {
            if !state.is_complete() {
                return Ok(());
            }
        }
        world.remove_resource::<TypewriterState>();

        if world.get_resource::<WaitTimer>().is_some() {
            return Ok(());
        }

        let current_node_id = {
            let history = world
                .get_resource::<DialogueHistory>()
                .ok_or_else(|| GError { kind: GErrorKind::Ecs, message: "DialogueHistory resource not found".to_string() })?;
            history.current_node_id.clone()
        };

        let current_id = match current_node_id {
            Some(id) => id,
            None => return Ok(()),
        };

        let (speaker_id, text, commands, choices, next_node_id) = {
            let mut found_node: Option<DialogueNode> = None;
            for entity in world.entities() {
                if let Some(node) = world.get_component::<DialogueNode>(entity) {
                    if node.id == current_id {
                        found_node = Some(node.clone());
                        break;
                    }
                }
            }
            let node = found_node
                .ok_or_else(|| GError { kind: GErrorKind::Ecs, message: format!("DialogueNode '{}' not found", current_id) })?;
            (node.speaker_id.clone(), node.text.clone(), node.commands.clone(), node.choices.clone(), node.next_node_id.clone())
        };

        for command in &commands {
            self.dispatcher.dispatch(command, world)?;
        }

        if !text.is_empty() {
            world.insert_resource(TypewriterState::new(text.clone(), DEFAULT_TYPEWRITER_SPEED));
        }

        if !choices.is_empty() {
            let choice_state = world
                .get_resource_mut::<ChoiceState>()
                .ok_or_else(|| GError { kind: GErrorKind::Ecs, message: "ChoiceState resource not found".to_string() })?;
            choice_state.choices = choices;
            choice_state.selected_index = None;
            choice_state.is_active = true;
        }
        else if let Some(next_id) = next_node_id {
            let history = world
                .get_resource_mut::<DialogueHistory>()
                .ok_or_else(|| GError { kind: GErrorKind::Ecs, message: "DialogueHistory resource not found".to_string() })?;
            history.current_node_id = Some(next_id);
        }

        let history = world
            .get_resource_mut::<DialogueHistory>()
            .ok_or_else(|| GError { kind: GErrorKind::Ecs, message: "DialogueHistory resource not found".to_string() })?;
        let speaker_name = speaker_id;
        history.entries.push(HistoryEntry { speaker_name, text, timestamp: 0.0 });

        Ok(())
    }
}

/// 选项系统
///
/// 负责处理玩家选择选项的逻辑：
/// - 查找 ChoiceState 资源
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
    /// 1. 查找 ChoiceState 资源
    /// 2. 如果 selected_index 有值，获取对应 Choice
    /// 3. 评估 Choice 的 condition（通过 GameVariables.evaluate_condition）
    /// 4. 推进到 next_node_id
    fn execute(&mut self, world: &mut World) -> GResult<()> {
        let (selected_index, choices, _is_active) = {
            let choice_state = match world.get_resource_mut::<ChoiceState>() {
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
                message: format!("Choice index {} out of bounds (len: {})", selected_index, choices.len()),
            });
        }

        let choice = &choices[selected_index];

        if let Some(ref condition) = choice.condition {
            let variables = world
                .get_resource::<GameVariables>()
                .ok_or_else(|| GError { kind: GErrorKind::Ecs, message: "GameVariables resource not found".to_string() })?;
            if !ExpressionEvaluator::evaluate(condition, &variables) {
                return Err(GError { kind: GErrorKind::Plugin, message: format!("Condition '{}' not satisfied", condition) });
            }
        }

        let next_node_id = choice.next_node_id.clone();
        {
            let choice_state = world
                .get_resource_mut::<ChoiceState>()
                .ok_or_else(|| GError { kind: GErrorKind::Ecs, message: "ChoiceState resource not found".to_string() })?;
            choice_state.is_active = false;
            choice_state.selected_index = None;
            choice_state.choices.clear();
        }

        let history = world
            .get_resource_mut::<DialogueHistory>()
            .ok_or_else(|| GError { kind: GErrorKind::Ecs, message: "DialogueHistory resource not found".to_string() })?;
        history.current_node_id = Some(next_node_id);

        Ok(())
    }
}

/// 打字机效果系统
///
/// 每帧更新 TypewriterState 资源，推进当前显示位置。
/// 使用 World 中的 DeltaTime 资源获取真实帧间隔。
pub struct TypewriterSystem;

impl TypewriterSystem {
    /// 创建新的打字机效果系统
    pub fn new() -> Self {
        Self
    }
}

impl System for TypewriterSystem {
    /// 返回系统名称
    fn name(&self) -> &str {
        "typewriter"
    }

    /// 执行打字机效果系统逻辑
    ///
    /// 如果 World 中存在 TypewriterState 资源，使用真实 delta time 更新其显示进度。
    fn execute(&mut self, world: &mut World) -> GResult<()> {
        let delta = world.get_resource::<DeltaTime>().map(|d| d.secs).unwrap_or(1.0 / 60.0);
        if let Some(state) = world.get_resource_mut::<TypewriterState>() {
            state.update(delta);
        }
        Ok(())
    }
}

/// 等待系统
///
/// 每帧更新 WaitTimer 资源倒计时，完成后移除 WaitTimer 资源。
/// 使用 World 中的 DeltaTime 资源获取真实帧间隔。
pub struct WaitSystem;

impl WaitSystem {
    /// 创建新的等待系统
    pub fn new() -> Self {
        Self
    }
}

impl System for WaitSystem {
    /// 返回系统名称
    fn name(&self) -> &str {
        "wait"
    }

    /// 执行等待系统逻辑
    ///
    /// 如果 World 中存在 WaitTimer 资源，使用真实 delta time 减少剩余时间。
    /// 当剩余时间小于等于零时，移除 WaitTimer 资源。
    fn execute(&mut self, world: &mut World) -> GResult<()> {
        let delta = world.get_resource::<DeltaTime>().map(|d| d.secs).unwrap_or(1.0 / 60.0);

        let should_remove = if let Some(timer) = world.get_resource_mut::<WaitTimer>() {
            timer.remaining_secs -= delta;
            timer.remaining_secs <= 0.0
        }
        else {
            false
        };

        if should_remove {
            world.remove_resource::<WaitTimer>();
        }

        Ok(())
    }
}
