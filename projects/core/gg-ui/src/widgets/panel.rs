use crate::{
    node::{UiNodeData, UiNodeId, UiTree},
    style::Style,
    widget::Widget,
};
use gg_core::GResult;

/// 面板控件
///
/// 提供容器功能的 UI 元素，用于组织和布局子控件。
pub struct Panel {
    /// 样式
    pub style: Style,
    /// 子控件节点 ID 列表
    pub children: Vec<UiNodeId>,
    /// 根节点 ID
    node_id: Option<UiNodeId>,
}

impl Panel {
    /// 创建面板控件
    pub fn new() -> Self {
        Self { style: Style::new(), children: Vec::new(), node_id: None }
    }

    /// 设置样式
    pub fn with_style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }

    /// 添加子节点 ID
    pub fn add_child(&mut self, child_id: UiNodeId) {
        self.children.push(child_id);
    }
}

impl Default for Panel {
    fn default() -> Self {
        Self::new()
    }
}

impl Widget for Panel {
    fn build(&mut self, tree: &mut UiTree) -> GResult<UiNodeId> {
        let id = tree.create_node("Panel", self.style.clone(), UiNodeData::Container);

        for &child_id in &self.children {
            tree.add_child(id, child_id);
        }

        self.node_id = Some(id);
        Ok(id)
    }

    fn update(&self, _tree: &mut UiTree) {}

    fn node_id(&self) -> Option<UiNodeId> {
        self.node_id
    }
}
