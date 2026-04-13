use crate::{
    gui_event::{EventContext, GuiEvent},
    node::{UiNodeData, UiNodeId, UiTree},
    reactive::Signal,
    style::{FlexAlign, FlexDirection, FontStyle, LayoutStyle, SizeValue, Style},
    widget::Widget,
};
use gg_core::GResult;

/// 进度条组件
///
/// 提供水平进度条 UI 元素，支持设置进度值、标签和百分比显示。
pub struct ProgressBar {
    /// 当前进度（0.0 - 1.0）
    pub value: Signal<f32>,
    /// 标签文本
    pub label: Option<String>,
    /// 是否显示百分比
    pub show_percentage: bool,
    /// 样式
    pub style: Style,
    /// 根节点 ID
    node_id: Option<UiNodeId>,
    /// 填充区域节点 ID
    fill_node_id: Option<UiNodeId>,
    /// 标签节点 ID
    label_node_id: Option<UiNodeId>,
    /// 百分比节点 ID
    percentage_node_id: Option<UiNodeId>,
}

impl ProgressBar {
    /// 创建进度条
    pub fn new() -> Self {
        let style = Style::new()
            .with_background_color(gg_render::Color::new(0.2, 0.2, 0.2, 1.0))
            .with_corner_radius(4.0)
            .with_layout(LayoutStyle::new().with_direction(FlexDirection::Column).with_padding(4.0).with_gap(4.0))
            .with_font(FontStyle::new());

        Self {
            value: Signal::new(0.0),
            label: None,
            show_percentage: false,
            style,
            node_id: None,
            fill_node_id: None,
            label_node_id: None,
            percentage_node_id: None,
        }
    }

    /// 设置初始进度值
    pub fn with_value(mut self, value: f32) -> Self {
        self.value.set(value.clamp(0.0, 1.0));
        self
    }

    /// 设置标签文本
    pub fn with_label(mut self, label: impl Into<String>) -> Self {
        self.label = Some(label.into());
        self
    }

    /// 设置是否显示百分比
    pub fn with_show_percentage(mut self, show: bool) -> Self {
        self.show_percentage = show;
        self
    }

    /// 设置样式
    pub fn with_style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }

    /// 设置进度值
    pub fn set_value(&mut self, value: f32) {
        self.value.set(value.clamp(0.0, 1.0));
    }
}

impl Default for ProgressBar {
    fn default() -> Self {
        Self::new()
    }
}

impl Widget for ProgressBar {
    fn build(&mut self, tree: &mut UiTree) -> GResult<UiNodeId> {
        let root_id = tree.create_node("ProgressBar", self.style.clone(), UiNodeData::Container);

        let mut label_node_id = None;

        if let Some(ref label) = self.label {
            let label_style = Style::new()
                .with_font(self.style.font.clone().unwrap_or_default())
                .with_layout(LayoutStyle::new().with_direction(FlexDirection::Row));

            let lid = tree.create_node("ProgressBar_Label", label_style, UiNodeData::Text { content: label.clone() });

            tree.add_child(root_id, lid);
            label_node_id = Some(lid);
        }

        let track_style =
            Style::new().with_background_color(gg_render::Color::new(0.3, 0.3, 0.3, 1.0)).with_corner_radius(4.0).with_layout(
                LayoutStyle::new()
                    .with_direction(FlexDirection::Row)
                    .with_width(SizeValue::Percent(1.0))
                    .with_height(SizeValue::Px(8.0)),
            );

        let track_id = tree.create_node("ProgressBar_Track", track_style, UiNodeData::Container);

        let value = self.value.get().clamp(0.0, 1.0);
        let fill_style = Style::new()
            .with_background_color(gg_render::Color::new(0.26, 0.52, 0.96, 1.0))
            .with_corner_radius(4.0)
            .with_layout(
                LayoutStyle::new()
                    .with_direction(FlexDirection::Row)
                    .with_width(SizeValue::Percent(value))
                    .with_height(SizeValue::Px(8.0)),
            );

        let fill_id = tree.create_node("ProgressBar_Fill", fill_style, UiNodeData::Container);

        tree.add_child(track_id, fill_id);
        tree.add_child(root_id, track_id);

        let mut percentage_node_id = None;

        if self.show_percentage {
            let pct_text = format!("{:.0}%", value * 100.0);
            let pct_style = Style::new()
                .with_font(self.style.font.clone().unwrap_or_default())
                .with_layout(LayoutStyle::new().with_direction(FlexDirection::Row).with_align_items(FlexAlign::End));

            let pid = tree.create_node("ProgressBar_Percentage", pct_style, UiNodeData::Text { content: pct_text });

            tree.add_child(root_id, pid);
            percentage_node_id = Some(pid);
        }

        self.node_id = Some(root_id);
        self.fill_node_id = Some(fill_id);
        self.label_node_id = label_node_id;
        self.percentage_node_id = percentage_node_id;

        Ok(root_id)
    }

    fn update(&self, tree: &mut UiTree) {
        let value = *self.value.get();
        let clamped = value.clamp(0.0, 1.0);

        if let Some(fill_id) = self.fill_node_id {
            if let Some(node) = tree.get_mut(fill_id) {
                node.style.layout.width = SizeValue::Percent(clamped);
            }
        }

        if let Some(label_node_id) = self.label_node_id {
            if let Some(ref label) = self.label {
                if let Some(node) = tree.get_mut(label_node_id) {
                    if let UiNodeData::Text { ref mut content } = node.data {
                        *content = label.clone();
                    }
                }
            }
        }

        if let Some(percentage_node_id) = self.percentage_node_id {
            let pct_text = format!("{:.0}%", value * 100.0);
            if let Some(node) = tree.get_mut(percentage_node_id) {
                if let UiNodeData::Text { ref mut content } = node.data {
                    *content = pct_text;
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

    fn handle_event(&mut self, _event: &GuiEvent, _ctx: &mut EventContext) {}
}
