use std::collections::HashMap;

use crate::{
    gui_event::{EventContext, GuiEvent, MouseButton},
    node::{UiNodeData, UiNodeId, UiTree},
    style::{FlexAlign, FlexDirection, FontStyle, LayoutStyle, Style},
    widget::Widget,
};
use gg_core::GResult;

/// 树节点
///
/// 表示树形结构中的单个节点，支持展开/折叠和子节点嵌套。
#[derive(Debug, Clone)]
pub struct TreeNode {
    /// 节点唯一标识
    pub id: String,
    /// 节点标签文本
    pub label: String,
    /// 节点图标
    pub icon: Option<String>,
    /// 子节点列表
    pub children: Vec<TreeNode>,
    /// 是否展开
    pub expanded: bool,
}

impl TreeNode {
    /// 创建树节点
    ///
    /// # 参数
    ///
    /// - `id` - 节点唯一标识
    /// - `label` - 节点标签文本
    pub fn new(id: impl Into<String>, label: impl Into<String>) -> Self {
        Self { id: id.into(), label: label.into(), icon: None, children: Vec::new(), expanded: false }
    }

    /// 设置图标
    pub fn with_icon(mut self, icon: impl Into<String>) -> Self {
        self.icon = Some(icon.into());
        self
    }

    /// 添加子节点
    pub fn add_child(&mut self, child: TreeNode) {
        self.children.push(child);
    }

    /// 是否为叶子节点
    pub fn is_leaf(&self) -> bool {
        self.children.is_empty()
    }
}

/// 树视图控件
///
/// 提供可展开/折叠的树形结构 UI 元素，支持节点选择和展开/折叠操作。
pub struct TreeView {
    /// 根节点列表
    pub roots: Vec<TreeNode>,
    /// 样式
    pub style: Style,
    /// 选中节点 ID
    pub selected_id: Option<String>,
    /// 节点选中回调
    pub on_select: Option<Box<dyn FnMut(&str) + Send + Sync>>,
    /// 节点展开/折叠回调
    pub on_toggle: Option<Box<dyn FnMut(&str, bool) + Send + Sync>>,
    /// 根节点 ID
    node_id: Option<UiNodeId>,
    /// 节点 ID 到 UiNodeId 的映射
    node_id_map: HashMap<String, UiNodeId>,
}

impl TreeView {
    /// 创建树视图控件
    ///
    /// # 参数
    ///
    /// - `roots` - 根节点列表
    pub fn new(roots: Vec<TreeNode>) -> Self {
        let style = Style::new()
            .with_layout(LayoutStyle::new().with_direction(FlexDirection::Column).with_padding(4.0).with_gap(2.0))
            .with_font(FontStyle::new());

        Self { roots, style, selected_id: None, on_select: None, on_toggle: None, node_id: None, node_id_map: HashMap::new() }
    }

    /// 设置样式
    pub fn with_style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }

    /// 选中指定节点
    pub fn select(&mut self, id: &str) {
        self.selected_id = Some(id.to_string());
    }

    /// 切换指定节点的展开/折叠状态
    pub fn toggle_node(&mut self, id: &str) {
        if let Some(node) = self.find_node_mut(id) {
            if !node.is_leaf() {
                node.expanded = !node.expanded;
            }
        }
    }

    /// 展开所有节点
    pub fn expand_all(&mut self) {
        for root in &mut self.roots {
            expand_all_recursive(root);
        }
    }

    /// 折叠所有节点
    pub fn collapse_all(&mut self) {
        for root in &mut self.roots {
            collapse_all_recursive(root);
        }
    }

    /// 查找指定 ID 的节点（不可变引用）
    pub fn find_node(&self, id: &str) -> Option<&TreeNode> {
        for root in &self.roots {
            if let Some(node) = find_node_recursive(root, id) {
                return Some(node);
            }
        }
        None
    }

    /// 查找指定 ID 的节点（可变引用）
    pub fn find_node_mut(&mut self, id: &str) -> Option<&mut TreeNode> {
        for root in &mut self.roots {
            if let Some(node) = find_node_mut_recursive(root, id) {
                return Some(node);
            }
        }
        None
    }
}

fn expand_all_recursive(node: &mut TreeNode) {
    if !node.is_leaf() {
        node.expanded = true;
        for child in &mut node.children {
            expand_all_recursive(child);
        }
    }
}

fn collapse_all_recursive(node: &mut TreeNode) {
    if !node.is_leaf() {
        node.expanded = false;
        for child in &mut node.children {
            collapse_all_recursive(child);
        }
    }
}

fn find_node_recursive<'a>(node: &'a TreeNode, id: &str) -> Option<&'a TreeNode> {
    if node.id == id {
        return Some(node);
    }
    for child in &node.children {
        if let Some(found) = find_node_recursive(child, id) {
            return Some(found);
        }
    }
    None
}

fn find_node_mut_recursive<'a>(node: &'a mut TreeNode, id: &str) -> Option<&'a mut TreeNode> {
    if node.id == id {
        return Some(node);
    }
    for child in &mut node.children {
        if let Some(found) = find_node_mut_recursive(child, id) {
            return Some(found);
        }
    }
    None
}

fn build_tree_node(
    tree: &mut UiTree,
    node: &TreeNode,
    selected_id: &Option<String>,
    depth: usize,
    node_id_map: &mut HashMap<String, UiNodeId>,
) -> UiNodeId {
    let is_selected = selected_id.as_deref() == Some(node.id.as_str());

    let row_bg =
        if is_selected { gg_render::Color::new(0.26, 0.52, 0.96, 0.3) } else { gg_render::Color::new(0.0, 0.0, 0.0, 0.0) };

    let indent = depth as f32 * 16.0;

    let row_style = Style::new().with_background_color(row_bg).with_corner_radius(2.0).with_layout(
        LayoutStyle::new()
            .with_direction(FlexDirection::Row)
            .with_align_items(FlexAlign::Center)
            .with_gap(4.0)
            .with_padding(4.0)
            .with_margin_left(indent),
    );

    let row_id = tree.create_node(format!("TreeNode_Row({})", node.id), row_style, UiNodeData::Container);

    let indicator_text = if node.is_leaf() {
        "  ".to_string()
    }
    else if node.expanded {
        "▼".to_string()
    }
    else {
        "▶".to_string()
    };

    let indicator_style = Style::new().with_font(FontStyle::new().with_size(10.0));

    let indicator_id = tree.create_node(
        format!("TreeNode_Indicator({})", node.id),
        indicator_style,
        UiNodeData::Text { content: indicator_text },
    );

    tree.add_child(row_id, indicator_id);

    if let Some(ref icon) = node.icon {
        let icon_style = Style::new().with_font(FontStyle::new().with_size(14.0));

        let icon_id =
            tree.create_node(format!("TreeNode_Icon({})", node.id), icon_style, UiNodeData::Text { content: icon.clone() });

        tree.add_child(row_id, icon_id);
    }

    let label_style = Style::new().with_font(FontStyle::new());

    let label_id =
        tree.create_node(format!("TreeNode_Label({})", node.id), label_style, UiNodeData::Text { content: node.label.clone() });

    tree.add_child(row_id, label_id);

    node_id_map.insert(node.id.clone(), row_id);

    if node.expanded && !node.is_leaf() {
        for child in &node.children {
            let child_row_id = build_tree_node(tree, child, selected_id, depth + 1, node_id_map);
            tree.add_child(row_id, child_row_id);
        }
    }

    row_id
}

impl Widget for TreeView {
    fn build(&mut self, tree: &mut UiTree) -> GResult<UiNodeId> {
        let root_id = tree.create_node("TreeView", self.style.clone(), UiNodeData::Container);

        let mut node_id_map = HashMap::new();

        for root_node in &self.roots {
            let child_row_id = build_tree_node(tree, root_node, &self.selected_id, 0, &mut node_id_map);
            tree.add_child(root_id, child_row_id);
        }

        self.node_id = Some(root_id);
        self.node_id_map = node_id_map;

        Ok(root_id)
    }

    fn update(&self, tree: &mut UiTree) {
        for (node_id_str, &ui_node_id) in &self.node_id_map {
            let is_selected = self.selected_id.as_deref() == Some(node_id_str.as_str());

            let bg = if is_selected {
                gg_render::Color::new(0.26, 0.52, 0.96, 0.3)
            }
            else {
                gg_render::Color::new(0.0, 0.0, 0.0, 0.0)
            };

            if let Some(node) = tree.get_mut(ui_node_id) {
                node.style.background_color = Some(bg);
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
            if let Some(ref first_root) = self.roots.first() {
                let id = first_root.id.clone();
                if first_root.expanded {
                    self.selected_id = Some(id.clone());
                    if let Some(ref mut cb) = self.on_select {
                        cb(&id);
                    }
                } else {
                    self.selected_id = Some(id.clone());
                    if let Some(ref mut cb) = self.on_toggle {
                        cb(&id, true);
                    }
                }
            }
        }
    }
}
