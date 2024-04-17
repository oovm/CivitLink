use crate::{
    gui_event::{EventContext, GuiEvent, MouseButton},
    node::{UiNodeData, UiNodeId, UiTree},
    style::{FlexAlign, FlexDirection, FontStyle, LayoutStyle, SizeValue, Style},
    widget::Widget,
};
use gg_core::GResult;

/// 复选框控件
///
/// 提供可切换选中状态的 UI 元素，包含一个勾选框和文本标签。
pub struct Checkbox {
    /// 文本标签
    pub label: String,
    /// 是否选中
    pub checked: bool,
    /// 状态变更回调
    pub on_change: Option<Box<dyn FnMut(bool) + Send + Sync>>,
    /// 根节点 ID
    node_id: Option<UiNodeId>,
    /// 勾选框节点 ID
    check_box_node_id: Option<UiNodeId>,
    /// 标签节点 ID
    label_node_id: Option<UiNodeId>,
}

impl Checkbox {
    /// 创建复选框控件
    ///
    /// # 参数
    ///
    /// - `label` - 文本标签
    /// - `checked` - 初始选中状态
    pub fn new(label: String, checked: bool) -> Self {
        Self { label, checked, on_change: None, node_id: None, check_box_node_id: None, label_node_id: None }
    }

    /// 设置选中状态
    pub fn set_checked(&mut self, checked: bool) {
        self.checked = checked;
    }

    /// 获取选中状态
    pub fn is_checked(&self) -> bool {
        self.checked
    }

    /// 切换选中状态并触发回调
    pub fn toggle(&mut self) {
        self.checked = !self.checked;
        if let Some(ref mut callback) = self.on_change {
            callback(self.checked);
        }
    }

    /// 设置状态变更回调
    pub fn set_on_change(&mut self, callback: Box<dyn FnMut(bool) + Send + Sync>) {
        self.on_change = Some(callback);
    }
}

impl Widget for Checkbox {
    fn build(&mut self, tree: &mut UiTree) -> GResult<UiNodeId> {
        let root_style = Style::new().with_layout(
            LayoutStyle::new().with_direction(FlexDirection::Row).with_align_items(FlexAlign::Center).with_gap(8.0),
        );

        let root_id = tree.create_node(format!("Checkbox({})", self.label), root_style, UiNodeData::Container);

        let check_bg =
            if self.checked { gg_render::Color::new(0.26, 0.52, 0.96, 1.0) } else { gg_render::Color::new(0.0, 0.0, 0.0, 0.0) };

        let check_box_style = Style::new()
            .with_background_color(check_bg)
            .with_border_color(gg_render::Color::new(0.5, 0.5, 0.5, 1.0))
            .with_border_width(2.0)
            .with_corner_radius(3.0)
            .with_layout(LayoutStyle::new().with_width(SizeValue::Px(20.0)).with_height(SizeValue::Px(20.0)));

        let check_box_id = tree.create_node(format!("Checkbox_Box({})", self.label), check_box_style, UiNodeData::Container);

        let label_style = Style::new().with_font(FontStyle::new());

        let label_id = tree.create_node(
            format!("Checkbox_Label({})", self.label),
            label_style,
            UiNodeData::Text { content: self.label.clone() },
        );

        tree.add_child(root_id, check_box_id);
        tree.add_child(root_id, label_id);

        self.node_id = Some(root_id);
        self.check_box_node_id = Some(check_box_id);
        self.label_node_id = Some(label_id);

        Ok(root_id)
    }

    fn update(&self, tree: &mut UiTree) {
        if let Some(check_box_id) = self.check_box_node_id {
            let bg = if self.checked {
                gg_render::Color::new(0.26, 0.52, 0.96, 1.0)
            }
            else {
                gg_render::Color::new(0.0, 0.0, 0.0, 0.0)
            };

            if let Some(node) = tree.get_mut(check_box_id) {
                node.style.background_color = Some(bg);
            }
        }

        if let Some(label_id) = self.label_node_id {
            if let Some(node) = tree.get_mut(label_id) {
                if let UiNodeData::Text { ref mut content } = node.data {
                    *content = self.label.clone();
                }
            }
        }
    }

    fn node_id(&self) -> Option<UiNodeId> {
        self.node_id
    }

    fn render_template(&self) -> oak_voc::TemplateNode {
        oak_voc::TemplateNode::text(String::new())
    }

    fn script_setup(&mut self) {}

    fn get_id(&self) -> &str {
        ""
    }

    fn handle_event(&mut self, event: &GuiEvent, _ctx: &mut EventContext) {
        if let GuiEvent::MouseClick { button: MouseButton::Left, .. } = event {
            self.checked = !self.checked;
            if let Some(ref mut cb) = self.on_change {
                cb(self.checked);
            }
        }
    }
}
