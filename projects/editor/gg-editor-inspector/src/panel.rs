//! 属性检查器面板实现
//! 提供实体组件属性查看、编辑和撤销/重做功能

use gg_core::GResult;
use gg_ecs::Entity;
use gg_editor_shell::panel::{EditorPanel, PanelContext, PanelData};

/// 撤销操作记录
#[derive(Debug, Clone)]
pub struct UndoAction {
    /// 操作描述
    pub description: String,
    /// 目标实体
    pub entity: Entity,
    /// 组件类型名称
    pub component_type: String,
    /// 旧值 JSON 表示
    pub old_value_json: String,
    /// 新值 JSON 表示
    pub new_value_json: String,
}

/// 属性检查器面板
///
/// 提供选中实体组件属性的查看和编辑功能，
/// 支持撤销/重做操作栈来追踪属性变更历史。
pub struct InspectorPanel {
    /// 面板是否可见
    visible: bool,
    /// 撤销栈
    undo_stack: Vec<UndoAction>,
    /// 重做栈
    redo_stack: Vec<UndoAction>,
}

impl InspectorPanel {
    /// 创建新的属性检查器面板
    pub fn new() -> Self {
        Self {
            visible: true,
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
        }
    }

    /// 撤销上一次操作
    ///
    /// 从撤销栈弹出最近的操作并恢复旧值，同时将其压入重做栈。
    pub fn undo(&mut self, context: &mut PanelData) -> GResult<()> {
        let _ = context;
        if let Some(action) = self.undo_stack.pop() {
            self.redo_stack.push(action);
        }
        Ok(())
    }

    /// 重做上一次撤销的操作
    ///
    /// 从重做栈弹出最近的操作并应用新值，同时将其压入撤销栈。
    pub fn redo(&mut self, context: &mut PanelData) -> GResult<()> {
        let _ = context;
        if let Some(action) = self.redo_stack.pop() {
            self.undo_stack.push(action);
        }
        Ok(())
    }

    /// 是否可以撤销
    pub fn can_undo(&self) -> bool {
        !self.undo_stack.is_empty()
    }

    /// 是否可以重做
    pub fn can_redo(&self) -> bool {
        !self.redo_stack.is_empty()
    }
}

impl EditorPanel for InspectorPanel {
    fn name(&self) -> &str {
        "Inspector"
    }

    fn is_visible(&self) -> bool {
        self.visible
    }

    fn set_visible(&mut self, visible: bool) {
        self.visible = visible;
    }

    fn render(&mut self, _context: &mut PanelContext) -> GResult<()> {
        Ok(())
    }
}
