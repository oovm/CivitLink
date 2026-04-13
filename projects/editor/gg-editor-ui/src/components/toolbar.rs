use gg_core::GResult;
use gg_render::Color;
use gg_ui::{
    EventContext, FlexAlign, FlexDirection, FontStyle, GuiEvent, LayoutStyle, Style, UiNodeData, UiNodeId, UiTree, Widget,
};

/// 工具按钮
///
/// 表示工具栏中的单个工具项，包含标签、可选图标和点击回调。
pub struct EditorTool {
    /// 工具标签
    pub label: String,
    /// 图标名称
    pub icon: Option<String>,
    /// 点击回调
    pub on_click: Option<Box<dyn FnMut() + Send + Sync>>,
}

impl EditorTool {
    /// 创建工具按钮
    ///
    /// # 参数
    ///
    /// - `label` - 工具标签文本
    pub fn new(label: impl Into<String>) -> Self {
        Self { label: label.into(), icon: None, on_click: None }
    }

    /// 设置图标名称
    pub fn with_icon(mut self, icon: impl Into<String>) -> Self {
        self.icon = Some(icon.into());
        self
    }

    /// 设置点击回调
    pub fn with_on_click(mut self, on_click: Box<dyn FnMut() + Send + Sync>) -> Self {
        self.on_click = Some(on_click);
        self
    }
}

/// 编辑器工具栏组件
///
/// 提供水平排列的工具按钮栏，用于放置编辑器常用操作工具。
pub struct EditorToolbar {
    /// 工具按钮列表
    pub tools: Vec<EditorTool>,
    /// 样式
    pub style: Style,
    /// 根节点 ID
    node_id: Option<UiNodeId>,
}

impl EditorToolbar {
    /// 创建空工具栏
    pub fn new() -> Self {
        let style = Style::new()
            .with_layout(
                LayoutStyle::new()
                    .with_direction(FlexDirection::Row)
                    .with_align_items(FlexAlign::Center)
                    .with_gap(4.0)
                    .with_padding(4.0),
            )
            .with_background_color(Color::new(0.2, 0.2, 0.2, 1.0));

        Self { tools: Vec::new(), style, node_id: None }
    }

    /// 添加工具按钮
    pub fn add_tool(mut self, label: impl Into<String>, on_click: Box<dyn FnMut() + Send + Sync>) -> Self {
        self.tools.push(EditorTool::new(label).with_on_click(on_click));
        self
    }

    /// 设置样式
    pub fn with_style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }
}

impl Default for EditorToolbar {
    fn default() -> Self {
        Self::new()
    }
}

impl Widget for EditorToolbar {
    fn build(&mut self, tree: &mut UiTree) -> GResult<UiNodeId> {
        let root_id = tree.create_node("EditorToolbar", self.style.clone(), UiNodeData::Container);

        for tool in &self.tools {
            let tool_style = Style::new()
                .with_layout(
                    LayoutStyle::new()
                        .with_direction(FlexDirection::Row)
                        .with_align_items(FlexAlign::Center)
                        .with_gap(4.0)
                        .with_padding(6.0),
                )
                .with_background_color(Color::new(0.3, 0.3, 0.3, 1.0))
                .with_corner_radius(4.0)
                .with_font(self.style.font.clone().unwrap_or_default());

            let tool_id = tree.create_node(format!("EditorToolbar_Tool({})", tool.label), tool_style, UiNodeData::Container);

            let label_style = Style::new().with_font(FontStyle::new());

            let label_id = tree.create_node(
                format!("EditorToolbar_ToolLabel({})", tool.label),
                label_style,
                UiNodeData::Text { content: tool.label.clone() },
            );

            tree.add_child(tool_id, label_id);
            tree.add_child(root_id, tool_id);
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
