//! 预览调试器实现
//! 提供预览运行时的状态捕获和调试信息查询功能

use gg_core::GResult;
use gg_editor_shell::EditorContext;
use gg_galgame_schema::resources::DialogueHistory;
use gg_world::GameWorld;

/// 预览调试器
///
/// 捕获和展示游戏预览运行时的调试状态信息，
/// 包括当前对话节点、对话历史、实体数量和帧率等。
pub struct PreviewDebugger {
    /// 是否暂停
    pub is_paused: bool,
    /// 当前对话节点 ID
    pub current_node_id: Option<String>,
    /// 对话历史摘要列表
    pub dialogue_history: Vec<String>,
    /// 实体数量
    pub entity_count: usize,
    /// 当前帧率
    pub fps: f32,
    /// 累计帧数
    pub frame_count: u64,
    /// 活跃实体数量
    pub active_entities: usize,
}

impl PreviewDebugger {
    /// 创建新的预览调试器
    pub fn new() -> Self {
        Self {
            is_paused: false,
            current_node_id: None,
            dialogue_history: Vec::new(),
            entity_count: 0,
            fps: 0.0,
            frame_count: 0,
            active_entities: 0,
        }
    }

    /// 捕获当前状态
    ///
    /// 从编辑器上下文中捕获当前游戏预览的运行时状态，
    /// 包括当前节点、对话历史、实体数量和帧率等信息。
    pub fn capture_state(context: &mut EditorContext) -> GResult<Self> {
        let world = context.world_mut();
        Self::capture_from_world(world)
    }

    /// 从 World 捕获调试状态
    ///
    /// 直接从 GameWorld 中读取对话历史、实体数量等调试信息。
    pub fn capture_from_world(world: &mut GameWorld) -> GResult<Self> {
        let current_node_id = world.get_resource::<DialogueHistory>().and_then(|h| h.current_node_id.clone());

        let dialogue_history = world
            .get_resource::<DialogueHistory>()
            .map(|h| h.entries.iter().map(|e| e.text.clone()).collect())
            .unwrap_or_default();

        let entity_count = world.ecs_world.entities().len();

        Ok(Self {
            is_paused: false,
            current_node_id,
            dialogue_history,
            entity_count,
            fps: 0.0,
            frame_count: 0,
            active_entities: entity_count,
        })
    }

    /// 获取对话历史
    ///
    /// 返回对话历史摘要的切片引用。
    pub fn get_dialogue_history(&self) -> &[String] {
        &self.dialogue_history
    }

    /// 获取当前节点 ID
    ///
    /// 返回当前对话节点的 ID 引用，如果没有则返回 None。
    pub fn get_current_node(&self) -> Option<&str> {
        self.current_node_id.as_deref()
    }

    /// 更新帧率
    ///
    /// 将当前帧率更新到调试器中。
    pub fn update_fps(&mut self, fps: f32) {
        self.fps = fps;
    }
}
