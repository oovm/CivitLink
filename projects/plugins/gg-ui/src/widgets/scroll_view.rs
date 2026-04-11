use crate::{
    node::{UiNodeData, UiNodeId, UiTree},
    style::{LayoutStyle, Overflow, SizeValue, Style},
    widget::Widget,
};
use gg_core::GResult;

/// 滚动视图控件
///
/// 提供可滚动的内容区域，通过裁剪溢出内容和偏移内容位置实现滚动效果。
pub struct ScrollView {
    /// 可视区域尺寸 `[width, height]`
    pub view_size: [f32; 2],
    /// 内容尺寸 `[width, height]`
    pub content_size: [f32; 2],
    /// 滚动偏移 `[x, y]`
    pub scroll_offset: [f32; 2],
    /// 根节点 ID（裁剪容器）
    node_id: Option<UiNodeId>,
    /// 内容容器节点 ID
    content_node_id: Option<UiNodeId>,
}

impl ScrollView {
    /// 创建滚动视图控件
    ///
    /// # 参数
    ///
    /// - `view_size` - 可视区域尺寸 `[width, height]`
    /// - `content_size` - 内容尺寸 `[width, height]`
    pub fn new(view_size: [f32; 2], content_size: [f32; 2]) -> Self {
        Self {
            view_size,
            content_size,
            scroll_offset: [0.0, 0.0],
            node_id: None,
            content_node_id: None,
        }
    }

    /// 设置滚动偏移，自动限制在内容边界内
    ///
    /// # 参数
    ///
    /// - `offset` - 目标滚动偏移 `[x, y]`
    pub fn set_scroll_offset(&mut self, offset: [f32; 2]) {
        let max_x = (self.content_size[0] - self.view_size[0]).max(0.0);
        let max_y = (self.content_size[1] - self.view_size[1]).max(0.0);
        self.scroll_offset[0] = offset[0].clamp(0.0, max_x);
        self.scroll_offset[1] = offset[1].clamp(0.0, max_y);
    }

    /// 获取当前滚动偏移
    pub fn scroll_offset(&self) -> [f32; 2] {
        self.scroll_offset
    }

    /// 处理垂直滚动事件
    ///
    /// 根据滚动增量调整垂直方向的滚动偏移，自动限制在内容边界内。
    ///
    /// # 参数
    ///
    /// - `delta_y` - 垂直方向滚动增量
    pub fn handle_scroll(&mut self, delta_y: f32) {
        let new_y = self.scroll_offset[1] + delta_y;
        let max_y = (self.content_size[1] - self.view_size[1]).max(0.0);
        self.scroll_offset[1] = new_y.clamp(0.0, max_y);
    }
}

impl Widget for ScrollView {
    fn build(&mut self, ui_tree: &mut UiTree) -> GResult<UiNodeId> {
        let root_style = Style::new()
            .with_overflow(Overflow::Clip)
            .with_layout(
                LayoutStyle::new()
                    .with_width(SizeValue::Px(self.view_size[0]))
                    .with_height(SizeValue::Px(self.view_size[1])),
            );

        let root_id = ui_tree.create_node("ScrollView", root_style, UiNodeData::Container);

        let content_style = Style::new().with_layout(
            LayoutStyle::new()
                .with_width(SizeValue::Px(self.content_size[0]))
                .with_height(SizeValue::Px(self.content_size[1]))
                .with_margin_left(-self.scroll_offset[0])
                .with_margin_top(-self.scroll_offset[1]),
        );

        let content_id = ui_tree.create_node("ScrollView_Content", content_style, UiNodeData::Container);
        ui_tree.add_child(root_id, content_id);

        self.node_id = Some(root_id);
        self.content_node_id = Some(content_id);
        Ok(root_id)
    }

    fn update(&self, ui_tree: &mut UiTree) {
        if let Some(content_id) = self.content_node_id {
            if let Some(node) = ui_tree.get_mut(content_id) {
                node.style.layout.margin_left = -self.scroll_offset[0];
                node.style.layout.margin_top = -self.scroll_offset[1];
            }
        }
    }

    fn node_id(&self) -> Option<UiNodeId> {
        self.node_id
    }
}
