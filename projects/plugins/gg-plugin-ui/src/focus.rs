//! UI 输入焦点管理模块
//! 提供焦点获取/释放和键盘输入路由功能

use gg_ui::UiNodeId;

/// 焦点管理器
///
/// 管理当前拥有输入焦点的 UI 节点，
/// 确保同一时间只有一个控件接收键盘输入。
#[derive(Debug, Clone, Default)]
pub struct FocusManager {
    /// 当前拥有焦点的节点 ID
    pub focused_node_id: Option<UiNodeId>,
}

impl FocusManager {
    /// 创建新的焦点管理器
    pub fn new() -> Self {
        Self::default()
    }

    /// 获取焦点
    ///
    /// 将输入焦点转移到指定节点，前一个焦点节点自动失去焦点。
    pub fn acquire_focus(&mut self, node_id: UiNodeId) {
        self.focused_node_id = Some(node_id);
    }

    /// 释放焦点
    ///
    /// 如果指定节点当前拥有焦点，则释放之。
    pub fn release_focus(&mut self, node_id: UiNodeId) {
        if self.focused_node_id == Some(node_id) {
            self.focused_node_id = None;
        }
    }

    /// 检查指定节点是否拥有焦点
    pub fn is_focused(&self, node_id: UiNodeId) -> bool {
        self.focused_node_id == Some(node_id)
    }

    /// 获取当前焦点节点 ID
    pub fn focused(&self) -> Option<UiNodeId> {
        self.focused_node_id
    }
}
