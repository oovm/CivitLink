use crate::{
    gui_event::{EventContext, GuiEvent, MouseButton},
    node::{UiNodeData, UiNodeId, UiTree},
    style::{FlexDirection, FontStyle, LayoutStyle, Style},
    widget::Widget,
};
use gg_core::GResult;

/// 提示位置
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TooltipPosition {
    /// 上方
    Top,
    /// 下方
    Bottom,
    /// 左方
    Left,
    /// 右方
    Right,
    /// 自动
    Auto,
}

impl Default for TooltipPosition {
    fn default() -> Self {
        Self::Auto
    }
}

/// 工具提示组件
///
/// 提供悬停提示 UI 元素，在目标组件上方或下方显示提示文本。
pub struct Tooltip {
    /// 提示文本
    pub content: String,
    /// 提示位置
    pub position: TooltipPosition,
    /// 显示延迟（毫秒）
    pub delay_ms: u64,
    /// 目标子组件节点 ID
    pub child: Option<UiNodeId>,
    /// 样式
    pub style: Style,
    /// 根节点 ID
    node_id: Option<UiNodeId>,
    /// 是否可见
    is_visible: bool,
    /// 提示文本节点 ID
    tooltip_text_id: Option<UiNodeId>,
}

impl Tooltip {
    /// 创建工具提示
    ///
    /// # 参数
    ///
    /// - `content` - 提示文本内容
    pub fn new(content: impl Into<String>) -> Self {
        let style = Style::new()
            .with_background_color(gg_render::Color::new(0.15, 0.15, 0.15, 0.95))
            .with_corner_radius(4.0)
            .with_layout(LayoutStyle::new().with_direction(FlexDirection::Column).with_padding(6.0))
            .with_font(FontStyle::new().with_size(12.0));

        Self {
            content: content.into(),
            position: TooltipPosition::default(),
            delay_ms: 500,
            child: None,
            style,
            node_id: None,
            is_visible: false,
            tooltip_text_id: None,
        }
    }

    /// 设置提示位置
    pub fn with_position(mut self, position: TooltipPosition) -> Self {
        self.position = position;
        self
    }

    /// 设置显示延迟
    pub fn with_delay(mut self, delay_ms: u64) -> Self {
        self.delay_ms = delay_ms;
        self
    }

    /// 设置样式
    pub fn with_style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }

    /// 显示提示
    pub fn show(&mut self) {
        self.is_visible = true;
    }

    /// 隐藏提示
    pub fn hide(&mut self) {
        self.is_visible = false;
    }
}

impl Widget for Tooltip {
    fn build(&mut self, tree: &mut UiTree) -> GResult<UiNodeId> {
        let root_style = Style::new().with_layout(LayoutStyle::new().with_direction(FlexDirection::Column));

        let root_id = tree.create_node("Tooltip", root_style, UiNodeData::Container);

        let child_area_style = Style::new().with_layout(LayoutStyle::new().with_direction(FlexDirection::Column));

        let child_area_id = tree.create_node("Tooltip_ChildArea", child_area_style, UiNodeData::Container);

        tree.add_child(root_id, child_area_id);
        self.child = Some(child_area_id);

        let tooltip_text_style = Style::new()
            .with_background_color(self.style.background_color.unwrap_or(gg_render::Color::new(0.15, 0.15, 0.15, 0.95)))
            .with_corner_radius(self.style.corner_radius)
            .with_layout(LayoutStyle::new().with_direction(FlexDirection::Column).with_padding(6.0))
            .with_font(self.style.font.clone().unwrap_or_else(|| FontStyle::new().with_size(12.0)));

        let tooltip_text_id =
            tree.create_node("Tooltip_Text", tooltip_text_style, UiNodeData::Text { content: self.content.clone() });

        tree.add_child(root_id, tooltip_text_id);

        if let Some(node) = tree.get_mut(tooltip_text_id) {
            node.visible = self.is_visible;
        }

        self.node_id = Some(root_id);
        self.tooltip_text_id = Some(tooltip_text_id);

        Ok(root_id)
    }

    fn update(&self, tree: &mut UiTree) {
        if let Some(tooltip_text_id) = self.tooltip_text_id {
            if let Some(node) = tree.get_mut(tooltip_text_id) {
                node.visible = self.is_visible;
                if let UiNodeData::Text { ref mut content } = node.data {
                    *content = self.content.clone();
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
        match event {
            GuiEvent::MouseMove { .. } => {
                if !self.is_visible {
                    self.show();
                }
            }
            GuiEvent::MouseClick { .. } => {
                if self.is_visible {
                    self.hide();
                }
            }
            _ => {}
        }
    }
}
