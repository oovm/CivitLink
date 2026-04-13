use gg_core::GResult;
use gg_render::Color;
use gg_ui::{
    EventContext, FlexAlign, FlexDirection, FontStyle, GuiEvent, LayoutStyle, Style, UiNodeData, UiNodeId, UiTree, Widget,
};

/// 编辑器树节点
///
/// 表示编辑器树形视图中的单个节点，支持图标和子节点嵌套。
pub struct EditorTreeNode {
    /// 节点名称
    pub name: String,
    /// 子节点列表
    pub children: Vec<EditorTreeNode>,
    /// 是否展开
    pub expanded: bool,
    /// 图标名称
    pub icon: String,
}

impl EditorTreeNode {
    /// 创建编辑器树节点
    ///
    /// # 参数
    ///
    /// - `name` - 节点名称
    /// - `icon` - 图标名称
    pub fn new(name: impl Into<String>, icon: impl Into<String>) -> Self {
        Self { name: name.into(), children: Vec::new(), expanded: false, icon: icon.into() }
    }

    /// 添加子节点
    pub fn add_child(mut self, child: EditorTreeNode) -> Self {
        self.children.push(child);
        self
    }

    /// 设置展开状态
    pub fn with_expanded(mut self, expanded: bool) -> Self {
        self.expanded = expanded;
        self
    }
}

/// 编辑器树形视图
///
/// 提供编辑器专用的树形结构 UI，支持节点选择和展开/折叠。
pub struct EditorTreeView {
    /// 树形数据
    pub data: Vec<EditorTreeNode>,
    /// 选中节点名称
    pub selected_name: Option<String>,
    /// 选中节点回调
    pub on_select: Option<Box<dyn FnMut(String) + Send + Sync>>,
    /// 样式
    pub style: Style,
    /// 根节点 ID
    node_id: Option<UiNodeId>,
}

impl EditorTreeView {
    /// 创建编辑器树形视图
    pub fn new() -> Self {
        let style = Style::new()
            .with_layout(LayoutStyle::new().with_direction(FlexDirection::Column).with_padding(4.0).with_gap(2.0))
            .with_background_color(Color::new(0.18, 0.18, 0.18, 1.0))
            .with_font(FontStyle::new());

        Self { data: Vec::new(), selected_name: None, on_select: None, style, node_id: None }
    }

    /// 添加根节点
    pub fn add_node(mut self, node: EditorTreeNode) -> Self {
        self.data.push(node);
        self
    }

    /// 设置选中回调
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

impl Default for EditorTreeView {
    fn default() -> Self {
        Self::new()
    }
}

impl Widget for EditorTreeView {
    fn build(&mut self, tree: &mut UiTree) -> GResult<UiNodeId> {
        let root_id = tree.create_node("EditorTreeView", self.style.clone(), UiNodeData::Container);

        for node in &self.data {
            Self::build_tree_node(tree, root_id, node, 0);
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
            if let Some(ref first) = self.data.first() {
                self.selected_name = Some(first.name.clone());
                if let Some(ref mut cb) = self.on_select {
                    cb(first.name.clone());
                }
            }
        }
    }
}

impl EditorTreeView {
    fn build_tree_node(tree: &mut UiTree, parent_id: UiNodeId, node: &EditorTreeNode, depth: usize) {
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

        let row_style = Style::new()
            .with_layout(
                LayoutStyle::new()
                    .with_direction(FlexDirection::Row)
                    .with_align_items(FlexAlign::Center)
                    .with_gap(4.0)
                    .with_padding(2.0),
            )
            .with_font(FontStyle::new().with_size(12.0));

        let row_id = tree.create_node(format!("EditorTreeNode({})", node.name), row_style, UiNodeData::Container);

        let icon_id = tree.create_node(
            format!("EditorTreeNode_Icon({})", node.name),
            Style::new().with_font(FontStyle::new().with_size(12.0)),
            UiNodeData::Text { content: format!("{}{}{}", indent, expand_indicator, node.icon) },
        );

        let label_id = tree.create_node(
            format!("EditorTreeNode_Label({})", node.name),
            Style::new().with_font(FontStyle::new().with_size(12.0)),
            UiNodeData::Text { content: node.name.clone() },
        );

        tree.add_child(row_id, icon_id);
        tree.add_child(row_id, label_id);
        tree.add_child(parent_id, row_id);

        if node.expanded {
            for child in &node.children {
                Self::build_tree_node(tree, parent_id, child, depth + 1);
            }
        }
    }
}
