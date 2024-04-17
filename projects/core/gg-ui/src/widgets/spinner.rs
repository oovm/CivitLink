use crate::{
    gui_event::{EventContext, GuiEvent},
    node::{UiNodeData, UiNodeId, UiTree},
    reactive::Signal,
    style::{FlexDirection, LayoutStyle, SizeValue, Style},
    widget::Widget,
};
use gg_core::GResult;
use gg_render::Color;

/// 加载指示器组件
///
/// 提供旋转加载动画 UI 元素，用于表示异步操作进行中。
pub struct Spinner {
    /// 指示器尺寸
    pub size: f32,
    /// 是否旋转
    pub is_spinning: Signal<bool>,
    /// 自定义颜色
    pub color: Option<Color>,
    /// 样式
    pub style: Style,
    /// 根节点 ID
    node_id: Option<UiNodeId>,
    /// 指示器节点 ID
    indicator_node_id: Option<UiNodeId>,
}

impl Spinner {
    /// 创建加载指示器
    ///
    /// 默认尺寸为 24.0。
    pub fn new() -> Self {
        let style = Style::new().with_layout(
            LayoutStyle::new()
                .with_direction(FlexDirection::Column)
                .with_align_items(crate::style::FlexAlign::Center)
                .with_justify_content(crate::style::FlexAlign::Center),
        );

        Self { size: 24.0, is_spinning: Signal::new(true), color: None, style, node_id: None, indicator_node_id: None }
    }

    /// 设置指示器尺寸
    pub fn with_size(mut self, size: f32) -> Self {
        self.size = size;
        self
    }

    /// 设置自定义颜色
    pub fn with_color(mut self, color: Color) -> Self {
        self.color = Some(color);
        self
    }

    /// 设置样式
    pub fn with_style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }

    /// 开始旋转
    pub fn start(&mut self) {
        self.is_spinning.set(true);
    }

    /// 停止旋转
    pub fn stop(&mut self) {
        self.is_spinning.set(false);
    }
}

impl Default for Spinner {
    fn default() -> Self {
        Self::new()
    }
}

impl Widget for Spinner {
    fn build(&mut self, tree: &mut UiTree) -> GResult<UiNodeId> {
        let root_id = tree.create_node("Spinner", self.style.clone(), UiNodeData::Container);

        let indicator_color = self.color.unwrap_or(gg_render::Color::new(0.5, 0.7, 1.0, 1.0));

        let indicator_style = Style::new()
            .with_border_color(indicator_color)
            .with_border_width(2.0)
            .with_corner_radius(self.size / 2.0)
            .with_layout(
                LayoutStyle::new()
                    .with_direction(FlexDirection::Column)
                    .with_width(SizeValue::Px(self.size))
                    .with_height(SizeValue::Px(self.size)),
            );

        let indicator_id = tree.create_node("Spinner_Indicator", indicator_style, UiNodeData::Container);

        tree.add_child(root_id, indicator_id);

        if !self.is_spinning.get() {
            if let Some(node) = tree.get_mut(root_id) {
                node.visible = false;
            }
        }

        self.node_id = Some(root_id);
        self.indicator_node_id = Some(indicator_id);

        Ok(root_id)
    }

    fn update(&self, tree: &mut UiTree) {
        if let Some(node_id) = self.node_id {
            if let Some(node) = tree.get_mut(node_id) {
                node.visible = *self.is_spinning.get();
            }
        }

        if let Some(indicator_node_id) = self.indicator_node_id {
            let indicator_color = self.color.unwrap_or(gg_render::Color::new(0.5, 0.7, 1.0, 1.0));
            if let Some(node) = tree.get_mut(indicator_node_id) {
                node.style.border_color = Some(indicator_color);
                node.style.corner_radius = self.size / 2.0;
                node.style.layout.width = SizeValue::Px(self.size);
                node.style.layout.height = SizeValue::Px(self.size);
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
