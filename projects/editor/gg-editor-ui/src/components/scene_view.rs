use gg_core::GResult;
use gg_render::Color;
use gg_ui::{EventContext, FontStyle, GuiEvent, LayoutStyle, Style, UiNodeData, UiNodeId, UiTree, Widget};

/// 场景编辑视图
///
/// 提供场景渲染和交互的视口区域。
pub struct SceneView {
    /// 场景名称
    pub scene_name: String,
    /// 样式
    pub style: Style,
    /// 根节点 ID
    node_id: Option<UiNodeId>,
}

impl SceneView {
    /// 创建场景视图
    ///
    /// # 参数
    ///
    /// - `scene_name` - 场景名称
    pub fn new(scene_name: impl Into<String>) -> Self {
        let style = Style::new().with_background_color(Color::new(0.1, 0.1, 0.1, 1.0)).with_font(FontStyle::new());

        Self { scene_name: scene_name.into(), style, node_id: None }
    }

    /// 设置样式
    pub fn with_style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }
}

impl Widget for SceneView {
    fn build(&mut self, tree: &mut UiTree) -> GResult<UiNodeId> {
        let root_id = tree.create_node("SceneView", self.style.clone(), UiNodeData::Container);

        let label_style = Style::new().with_font(FontStyle::new().with_size(12.0));

        let label_id = tree.create_node(
            "SceneView_Label",
            label_style,
            UiNodeData::Text { content: format!("Scene: {}", self.scene_name) },
        );

        tree.add_child(root_id, label_id);

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
