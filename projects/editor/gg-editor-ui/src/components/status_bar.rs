use gg_core::GResult;
use gg_render::Color;
use gg_ui::{
    EventContext, FlexAlign, FlexDirection, FontStyle, GuiEvent, LayoutStyle, Style, UiNodeData, UiNodeId, UiTree, Widget,
};

/// 状态区段
///
/// 表示状态栏中的一个区段，包含显示内容和可选固定宽度。
pub struct StatusSection {
    /// 区段内容
    pub content: String,
    /// 固定宽度
    pub width: Option<f32>,
}

impl StatusSection {
    /// 创建状态区段
    ///
    /// # 参数
    ///
    /// - `content` - 区段显示内容
    pub fn new(content: impl Into<String>) -> Self {
        Self { content: content.into(), width: None }
    }

    /// 设置固定宽度
    pub fn with_width(mut self, width: f32) -> Self {
        self.width = Some(width);
        self
    }
}

/// 编辑器状态栏组件
///
/// 提供水平排列的状态信息栏，用于显示编辑器底部状态信息（如行列号、编码格式等）。
pub struct EditorStatusBar {
    /// 状态区段列表
    pub sections: Vec<StatusSection>,
    /// 样式
    pub style: Style,
    /// 根节点 ID
    node_id: Option<UiNodeId>,
}

impl EditorStatusBar {
    /// 创建空状态栏
    pub fn new() -> Self {
        let style = Style::new()
            .with_layout(
                LayoutStyle::new()
                    .with_direction(FlexDirection::Row)
                    .with_align_items(FlexAlign::Center)
                    .with_gap(8.0)
                    .with_padding(4.0),
            )
            .with_background_color(Color::new(0.15, 0.15, 0.15, 1.0))
            .with_font(FontStyle::new().with_size(12.0));

        Self { sections: Vec::new(), style, node_id: None }
    }

    /// 添加状态区段
    pub fn add_section(mut self, content: impl Into<String>, width: Option<f32>) -> Self {
        let mut section = StatusSection::new(content);
        if let Some(w) = width {
            section = section.with_width(w);
        }
        self.sections.push(section);
        self
    }

    /// 设置样式
    pub fn with_style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }
}

impl Default for EditorStatusBar {
    fn default() -> Self {
        Self::new()
    }
}

impl Widget for EditorStatusBar {
    fn build(&mut self, tree: &mut UiTree) -> GResult<UiNodeId> {
        let root_id = tree.create_node("EditorStatusBar", self.style.clone(), UiNodeData::Container);

        for section in &self.sections {
            let mut section_layout =
                LayoutStyle::new().with_direction(FlexDirection::Row).with_align_items(FlexAlign::Center).with_padding(4.0);

            if let Some(width) = section.width {
                section_layout = section_layout.with_width(gg_ui::style::SizeValue::Px(width));
            }

            let section_style = Style::new().with_layout(section_layout).with_font(self.style.font.clone().unwrap_or_default());

            let section_id =
                tree.create_node(format!("EditorStatusBar_Section({})", section.content), section_style, UiNodeData::Container);

            let label_style = Style::new().with_font(FontStyle::new());

            let label_id = tree.create_node(
                format!("EditorStatusBar_SectionLabel({})", section.content),
                label_style,
                UiNodeData::Text { content: section.content.clone() },
            );

            tree.add_child(section_id, label_id);
            tree.add_child(root_id, section_id);
        }

        self.node_id = Some(root_id);
        Ok(root_id)
    }

    fn update(&self, _tree: &mut UiTree) {}

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
