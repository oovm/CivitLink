use gg_core::GResult;
use gg_render::Color;
use gg_ui::{EventContext, FlexDirection, FontStyle, GuiEvent, LayoutStyle, Style, UiNodeData, UiNodeId, UiTree, Widget};

/// 层级节点数据
///
/// 表示层级视图中的单个节点，支持子节点嵌套和展开/折叠。
pub struct HierarchyNode {
    /// 节点名称
    pub name: String,
    /// 子节点列表
    pub children: Vec<HierarchyNode>,
    /// 是否展开
    pub expanded: bool,
}

impl HierarchyNode {
    /// 创建层级节点
    ///
    /// # 参数
    ///
    /// - `name` - 节点名称
    pub fn new(name: impl Into<String>) -> Self {
        Self { name: name.into(), children: Vec::new(), expanded: false }
    }

    /// 添加子节点
    pub fn add_child(mut self, child: HierarchyNode) -> Self {
        self.children.push(child);
        self
    }

    /// 设置展开状态
    pub fn with_expanded(mut self, expanded: bool) -> Self {
        self.expanded = expanded;
        self
    }
}

/// 层级视图面板
///
/// 提供场景层级结构的树形显示，支持节点选择和展开/折叠。
pub struct HierarchyView {
    /// 根节点数据
    pub root: Option<HierarchyNode>,
    /// 选中节点名称
    pub selected_name: Option<String>,
    /// 节点选中回调
    pub on_select: Option<Box<dyn FnMut(String) + Send + Sync>>,
    /// 样式
    pub style: Style,
    /// 根节点 ID
    node_id: Option<UiNodeId>,
}

impl HierarchyView {
    /// 创建层级视图面板
    pub fn new() -> Self {
        let style = Style::new()
            .with_layout(LayoutStyle::new().with_direction(FlexDirection::Column).with_padding(4.0).with_gap(2.0))
            .with_background_color(Color::new(0.18, 0.18, 0.18, 1.0))
            .with_font(FontStyle::new());

        Self { root: None, selected_name: None, on_select: None, style, node_id: None }
    }

    /// 设置根节点数据
    pub fn with_root(mut self, root: HierarchyNode) -> Self {
        self.root = Some(root);
        self
    }

    /// 设置节点选中回调
    pub fn with_on_select(mut self, on_select: Box<dyn FnMut(String) + Send + Sync>) -> Self {
        self.on_select = Some(on_select);
        self
    }

    /// 设置样式
    pub fn with_style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }
}

impl Default for HierarchyView {
    fn default() -> Self {
        Self::new()
    }
}

impl Widget for HierarchyView {
    fn build(&mut self, tree: &mut UiTree) -> GResult<UiNodeId> {
        let root_id = tree.create_node("HierarchyView", self.style.clone(), UiNodeData::Container);

        let header_style = Style::new().with_font(FontStyle::new().with_size(14.0));

        let header_id =
            tree.create_node("HierarchyView_Header", header_style, UiNodeData::Text { content: "Hierarchy".to_string() });

        tree.add_child(root_id, header_id);

        if let Some(ref node) = self.root {
            Self::build_hierarchy_node(tree, root_id, node, 0);
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

    fn handle_event(&mut self, event: &GuiEvent, _ctx: &mut EventContext) {
        if let GuiEvent::MouseClick { button: gg_ui::MouseButton::Left, .. } = event {
            if let Some(ref node) = self.root {
                self.selected_name = Some(node.name.clone());
                if let Some(ref mut cb) = self.on_select {
                    cb(node.name.clone());
                }
            }
        }
    }
}

impl HierarchyView {
    fn build_hierarchy_node(tree: &mut UiTree, parent_id: UiNodeId, node: &HierarchyNode, depth: usize) {
        let indent = "  ".repeat(depth);
        let expand_indicator = if node.children.is_empty() {
            "  ".to_string()
        }
        else if node.expanded {
            "▼ ".to_string()
        }
        else {
            "▶ ".to_string()
        };

        let label_style = Style::new().with_font(FontStyle::new().with_size(12.0));

        let label_id = tree.create_node(
            format!("HierarchyView_Node({})", node.name),
            label_style,
            UiNodeData::Text { content: format!("{}{}{}", indent, expand_indicator, node.name) },
        );

        tree.add_child(parent_id, label_id);

        if node.expanded {
            for child in &node.children {
                Self::build_hierarchy_node(tree, parent_id, child, depth + 1);
            }
        }
    }
}
