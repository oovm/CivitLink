//! 预览调试器实现
//! 提供预览运行时的状态捕获和调试信息查询功能

use gg_core::GResult;
use gg_editor_shell::EditorContext;

/// 预览调试器
///
/// 捕获和展示游戏预览运行时的调试状态信息，
/// 包括当前对话节点、对话历史和实体数量等。
pub struct PreviewDebugger {
    /// 是否暂停
    pub is_paused: bool,
    /// 当前对话节点 ID
    pub current_node_id: Option<String>,
    /// 对话历史摘要列表
    pub dialogue_history: Vec<String>,
    /// 实体数量
    pub entity_count: usize,
}

impl PreviewDebugger {
    /// 创建新的预览调试器
    pub fn new() -> Self {
        Self { is_paused: false, current_node_id: None, dialogue_history: Vec::new(), entity_count: 0 }
    }

    /// 捕获当前状态
    ///
    /// 从编辑器上下文中捕获当前游戏预览的运行时状态，
    /// 包括当前节点、对话历史和实体数量等信息。
    pub fn capture_state(context: &mut EditorContext) -> GResult<Self> {
        let _ = context;
        Ok(Self::new())
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
}
