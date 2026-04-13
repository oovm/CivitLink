use gg_core::GResult;
use gg_render::Color;
use gg_ui::{EventContext, FlexDirection, FontStyle, GuiEvent, LayoutStyle, Style, UiNodeData, UiNodeId, UiTree, Widget};

/// 分割方向
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SplitOrientation {
    /// 水平分割（左右布局）
    Horizontal,
    /// 垂直分割（上下布局）
    Vertical,
}

/// 分割视图
///
/// 提供两个子视图的分割容器，支持拖拽调整分割比例。
pub struct SplitView {
    /// 分割方向
    pub orientation: SplitOrientation,
    /// 分割比例（0.0~1.0）
    pub ratio: f32,
    /// 样式
    pub style: Style,
    /// 根节点 ID
    node_id: Option<UiNodeId>,
    /// 是否正在拖拽分割线
    is_dragging: bool,
}

impl SplitView {
    /// 创建分割视图
    ///
    /// # 参数
    ///
    /// - `orientation` - 分割方向
    pub fn new(orientation: SplitOrientation) -> Self {
        let direction = match orientation {
            SplitOrientation::Horizontal => FlexDirection::Row,
            SplitOrientation::Vertical => FlexDirection::Column,
        };

        let style =
            Style::new().with_layout(LayoutStyle::new().with_direction(direction).with_gap(0.0)).with_font(FontStyle::new());

        Self { orientation, ratio: 0.5, style, node_id: None, is_dragging: false }
    }

    /// 设置分割比例
    pub fn with_ratio(mut self, ratio: f32) -> Self {
        self.ratio = ratio.clamp(0.1, 0.9);
        self
    }

    /// 设置样式
    pub fn with_style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }
}

impl Widget for SplitView {
    fn build(&mut self, tree: &mut UiTree) -> GResult<UiNodeId> {
        let root_id = tree.create_node("SplitView", self.style.clone(), UiNodeData::Container);

        let first_style = Style::new()
            .with_background_color(Color::new(0.2, 0.2, 0.2, 1.0))
            .with_font(self.style.font.clone().unwrap_or_default());

        let first_id = tree.create_node("SplitView_First", first_style, UiNodeData::Container);

        let divider_style = Style::new().with_background_color(Color::new(0.4, 0.4, 0.4, 1.0));

        let divider_id = tree.create_node("SplitView_Divider", divider_style, UiNodeData::Container);

        let second_style = Style::new()
            .with_background_color(Color::new(0.2, 0.2, 0.2, 1.0))
            .with_font(self.style.font.clone().unwrap_or_default());

        let second_id = tree.create_node("SplitView_Second", second_style, UiNodeData::Container);

        tree.add_child(root_id, first_id);
        tree.add_child(root_id, divider_id);
        tree.add_child(root_id, second_id);

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

    fn handle_event(&mut self, event: &GuiEvent, _ctx: &mut EventContext) {
        match event {
            GuiEvent::MouseClick { button: gg_ui::MouseButton::Left, .. } => {
                self.is_dragging = true;
            }
            GuiEvent::MouseMove { x, y } => {
                if self.is_dragging {
                    let pos = match self.orientation {
                        SplitOrientation::Horizontal => *x,
                        SplitOrientation::Vertical => *y,
                    };
                    self.ratio = pos.clamp(0.1, 0.9);
                }
            }
            _ => {}
        }
    }
}
