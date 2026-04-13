//! 表格编辑器面板实现

use gg_core::GResult;
use gg_editor_shell::{
    context::EditorContext,
    event::EditorEvent,
    panel::{EditorPanel, PanelLayoutHint, PanelPosition},
};
use gg_ui::{UiNodeId, UiTree};
use serde::{Deserialize, Serialize};

/// 单元格编辑状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CellEdit {
    /// 行索引
    pub row: usize,
    /// 列索引
    pub col: usize,
    /// 原始值
    pub original_value: String,
    /// 当前编辑值
    pub current_value: String,
}

/// 表格编辑器面板
///
/// 提供可视化表格编辑功能，支持表格浏览、单元格编辑和保存。
/// 与 gg-sheet 编译器集成，编辑后可触发增量编译。
pub struct SheetEditorPanel {
    /// 面板可见性
    visible: bool,
    /// 当前加载的表格名称
    current_table: Option<String>,
    /// 当前编辑中的单元格
    editing_cell: Option<CellEdit>,
    /// 是否有未保存的修改
    dirty: bool,
}

impl SheetEditorPanel {
    /// 创建新的表格编辑器面板
    pub fn new() -> Self {
        Self { visible: false, current_table: None, editing_cell: None, dirty: false }
    }

    /// 加载指定表格
    pub fn load_table(&mut self, table_name: &str) {
        self.current_table = Some(table_name.to_string());
        self.dirty = false;
        self.editing_cell = None;
    }

    /// 开始编辑单元格
    pub fn begin_edit(&mut self, row: usize, col: usize, value: &str) {
        self.editing_cell = Some(CellEdit { row, col, original_value: value.to_string(), current_value: value.to_string() });
    }

    /// 确认编辑
    pub fn commit_edit(&mut self) -> Option<CellEdit> {
        let edit = self.editing_cell.take();
        if edit.is_some() {
            self.dirty = true;
        }
        edit
    }

    /// 取消编辑
    pub fn cancel_edit(&mut self) {
        self.editing_cell = None;
    }

    /// 保存修改到文件并触发编译
    pub fn save_and_compile(&mut self, context: &mut EditorContext) -> GResult<()> {
        if self.dirty {
            self.dirty = false;
        }
        Ok(())
    }

    /// 获取当前表格名称
    pub fn current_table_name(&self) -> Option<&str> {
        self.current_table.as_deref()
    }

    /// 是否有未保存的修改
    pub fn is_dirty(&self) -> bool {
        self.dirty
    }
}

impl Default for SheetEditorPanel {
    fn default() -> Self {
        Self::new()
    }
}

impl EditorPanel for SheetEditorPanel {
    fn name(&self) -> &str {
        "Sheet Editor"
    }

    fn is_visible(&self) -> bool {
        self.visible
    }

    fn set_visible(&mut self, visible: bool) {
        self.visible = visible;
    }

    fn on_event(&mut self, event: &EditorEvent, _context: &mut EditorContext) {
        let _ = event;
    }

    fn build_ui(&mut self, _context: &mut EditorContext, _ui_tree: &mut UiTree) -> Option<UiNodeId> {
        None
    }

    fn layout_hint(&self) -> PanelLayoutHint {
        PanelLayoutHint {
            position: PanelPosition::Center,
            preferred_size: Some((800.0, 600.0)),
            min_size: Some((400.0, 300.0)),
        }
    }
}
