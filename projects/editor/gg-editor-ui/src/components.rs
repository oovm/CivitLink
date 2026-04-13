//! 基础UI组件实现
//!
//! 提供编辑器UI系统的基础组件，如Layout、Button、Text等
//! 所有组件实现从 gg-ui 重新导出，并扩展编辑器专用组件

pub use gg_ui::widgets_builtin::{Button, Image, Input, Layout, Panel, ScrollView, Stack, Text};

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

/// 菜单项（简化版，不同于 context_menu 的 MenuItem）
///
/// 表示菜单组中的单个可点击项，包含标签、可选快捷键和点击回调。
pub struct MenuItem {
    /// 菜单项标签
    pub label: String,
    /// 快捷键
    pub shortcut: Option<String>,
    /// 点击回调
    pub on_click: Option<Box<dyn FnMut() + Send + Sync>>,
}

impl MenuItem {
    /// 创建菜单项
    ///
    /// # 参数
    ///
    /// - `label` - 菜单项标签文本
    pub fn new(label: impl Into<String>) -> Self {
        Self { label: label.into(), shortcut: None, on_click: None }
    }

    /// 设置快捷键
    pub fn with_shortcut(mut self, shortcut: impl Into<String>) -> Self {
        self.shortcut = Some(shortcut.into());
        self
    }

    /// 设置点击回调
    pub fn with_on_click(mut self, on_click: Box<dyn FnMut() + Send + Sync>) -> Self {
        self.on_click = Some(on_click);
        self
    }
}

/// 菜单组
///
/// 表示菜单栏中的一组相关菜单项，包含组标签和菜单项列表。
pub struct MenuGroup {
    /// 菜单组标签
    pub label: String,
    /// 菜单项列表
    pub items: Vec<MenuItem>,
}

impl MenuGroup {
    /// 创建菜单组
    ///
    /// # 参数
    ///
    /// - `label` - 菜单组标签文本
    pub fn new(label: impl Into<String>) -> Self {
        Self { label: label.into(), items: Vec::new() }
    }

    /// 添加菜单项
    pub fn add_item(mut self, item: MenuItem) -> Self {
        self.items.push(item);
        self
    }
}

/// 编辑器菜单栏组件
///
/// 提供水平排列的菜单组栏，用于放置编辑器顶部菜单（如文件、编辑、视图等）。
pub struct EditorMenuBar {
    /// 菜单组列表
    pub menus: Vec<MenuGroup>,
    /// 样式
    pub style: Style,
    /// 根节点 ID
    node_id: Option<UiNodeId>,
}

impl EditorMenuBar {
    /// 创建空菜单栏
    pub fn new() -> Self {
        let style = Style::new()
            .with_layout(
                LayoutStyle::new()
                    .with_direction(FlexDirection::Row)
                    .with_align_items(FlexAlign::Center)
                    .with_gap(0.0)
                    .with_padding(2.0),
            )
            .with_background_color(Color::new(0.15, 0.15, 0.15, 1.0));

        Self { menus: Vec::new(), style, node_id: None }
    }

    /// 添加菜单组
    pub fn add_menu(mut self, label: impl Into<String>, items: Vec<MenuItem>) -> Self {
        let mut group = MenuGroup::new(label);
        for item in items {
            group.items.push(item);
        }
        self.menus.push(group);
        self
    }

    /// 设置样式
    pub fn with_style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }
}

impl Default for EditorMenuBar {
    fn default() -> Self {
        Self::new()
    }
}

impl Widget for EditorMenuBar {
    fn build(&mut self, tree: &mut UiTree) -> GResult<UiNodeId> {
        let root_id = tree.create_node("EditorMenuBar", self.style.clone(), UiNodeData::Container);

        for menu in &self.menus {
            let menu_style = Style::new()
                .with_layout(
                    LayoutStyle::new().with_direction(FlexDirection::Row).with_align_items(FlexAlign::Center).with_padding(4.0),
                )
                .with_font(self.style.font.clone().unwrap_or_default());

            let menu_id = tree.create_node(format!("EditorMenuBar_Menu({})", menu.label), menu_style, UiNodeData::Container);

            let label_style = Style::new().with_font(FontStyle::new());

            let label_id = tree.create_node(
                format!("EditorMenuBar_MenuLabel({})", menu.label),
                label_style,
                UiNodeData::Text { content: menu.label.clone() },
            );

            tree.add_child(menu_id, label_id);
            tree.add_child(root_id, menu_id);
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

/// 属性条目
///
/// 表示属性检查器中的单个属性，包含标签、类型、值和变更回调。
pub struct PropertyEntry {
    /// 属性标签
    pub label: String,
    /// 属性类型名称
    pub prop_type: String,
    /// 属性当前值
    pub value: String,
    /// 值变更回调
    pub on_change: Option<Box<dyn FnMut(String) + Send + Sync>>,
}

impl PropertyEntry {
    /// 创建属性条目
    ///
    /// # 参数
    ///
    /// - `label` - 属性标签文本
    /// - `prop_type` - 属性类型名称
    /// - `value` - 属性当前值
    pub fn new(label: impl Into<String>, prop_type: impl Into<String>, value: impl Into<String>) -> Self {
        Self { label: label.into(), prop_type: prop_type.into(), value: value.into(), on_change: None }
    }

    /// 设置值变更回调
    pub fn with_on_change(mut self, on_change: Box<dyn FnMut(String) + Send + Sync>) -> Self {
        self.on_change = Some(on_change);
        self
    }
}

/// 属性检查器面板
///
/// 提供选中对象的属性编辑 UI，以标签-值对的形式显示和修改属性。
pub struct InspectorPanel {
    /// 目标对象名称
    pub target_name: String,
    /// 属性列表
    pub properties: Vec<PropertyEntry>,
    /// 样式
    pub style: Style,
    /// 根节点 ID
    node_id: Option<UiNodeId>,
}

impl InspectorPanel {
    /// 创建属性检查器面板
    ///
    /// # 参数
    ///
    /// - `target_name` - 目标对象名称
    pub fn new(target_name: impl Into<String>) -> Self {
        let style = Style::new()
            .with_layout(LayoutStyle::new().with_direction(FlexDirection::Column).with_padding(8.0).with_gap(4.0))
            .with_background_color(Color::new(0.18, 0.18, 0.18, 1.0))
            .with_font(FontStyle::new());

        Self { target_name: target_name.into(), properties: Vec::new(), style, node_id: None }
    }

    /// 添加属性条目
    pub fn add_property(mut self, entry: PropertyEntry) -> Self {
        self.properties.push(entry);
        self
    }

    /// 设置样式
    pub fn with_style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }
}

impl Widget for InspectorPanel {
    fn build(&mut self, tree: &mut UiTree) -> GResult<UiNodeId> {
        let root_id = tree.create_node("InspectorPanel", self.style.clone(), UiNodeData::Container);

        let header_style = Style::new()
            .with_layout(LayoutStyle::new().with_direction(FlexDirection::Row).with_padding(4.0))
            .with_font(FontStyle::new().with_size(14.0));

        let header_id = tree.create_node(
            "InspectorPanel_Header",
            header_style,
            UiNodeData::Text { content: format!("Inspector: {}", self.target_name) },
        );

        tree.add_child(root_id, header_id);

        for prop in &self.properties {
            let row_style = Style::new()
                .with_layout(
                    LayoutStyle::new()
                        .with_direction(FlexDirection::Row)
                        .with_align_items(FlexAlign::Center)
                        .with_gap(8.0)
                        .with_padding(4.0),
                )
                .with_font(self.style.font.clone().unwrap_or_default());

            let row_id = tree.create_node(format!("InspectorPanel_Row({})", prop.label), row_style, UiNodeData::Container);

            let label_style = Style::new().with_font(FontStyle::new().with_size(12.0));

            let label_id = tree.create_node(
                format!("InspectorPanel_Label({})", prop.label),
                label_style,
                UiNodeData::Text { content: format!("{}:", prop.label) },
            );

            let value_style = Style::new()
                .with_background_color(Color::new(0.25, 0.25, 0.25, 1.0))
                .with_border_color(Color::new(0.4, 0.4, 0.4, 1.0))
                .with_border_width(1.0)
                .with_corner_radius(2.0)
                .with_font(FontStyle::new().with_size(12.0));

            let value_id = tree.create_node(
                format!("InspectorPanel_Value({})", prop.label),
                value_style,
                UiNodeData::Text { content: prop.value.clone() },
            );

            tree.add_child(row_id, label_id);
            tree.add_child(row_id, value_id);
            tree.add_child(root_id, row_id);
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

/// 资源条目
///
/// 表示资源浏览器中的单个资源，包含名称、类型图标和路径。
pub struct AssetEntry {
    /// 资源名称
    pub name: String,
    /// 资源类型图标
    pub icon: String,
    /// 资源路径
    pub path: String,
}

impl AssetEntry {
    /// 创建资源条目
    ///
    /// # 参数
    ///
    /// - `name` - 资源名称
    /// - `icon` - 资源类型图标名称
    /// - `path` - 资源路径
    pub fn new(name: impl Into<String>, icon: impl Into<String>, path: impl Into<String>) -> Self {
        Self { name: name.into(), icon: icon.into(), path: path.into() }
    }
}

/// 资源浏览器面板
///
/// 提供项目资源的浏览、搜索和管理功能。
pub struct AssetBrowser {
    /// 当前路径
    pub path: String,
    /// 资源列表
    pub assets: Vec<AssetEntry>,
    /// 搜索查询
    pub search_query: String,
    /// 样式
    pub style: Style,
    /// 根节点 ID
    node_id: Option<UiNodeId>,
}

impl AssetBrowser {
    /// 创建资源浏览器面板
    ///
    /// # 参数
    ///
    /// - `path` - 初始浏览路径
    pub fn new(path: impl Into<String>) -> Self {
        let style = Style::new()
            .with_layout(LayoutStyle::new().with_direction(FlexDirection::Column).with_padding(4.0).with_gap(4.0))
            .with_background_color(Color::new(0.18, 0.18, 0.18, 1.0))
            .with_font(FontStyle::new());

        Self { path: path.into(), assets: Vec::new(), search_query: String::new(), style, node_id: None }
    }

    /// 添加资源条目
    pub fn add_asset(mut self, entry: AssetEntry) -> Self {
        self.assets.push(entry);
        self
    }

    /// 设置样式
    pub fn with_style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }
}

impl Widget for AssetBrowser {
    fn build(&mut self, tree: &mut UiTree) -> GResult<UiNodeId> {
        let root_id = tree.create_node("AssetBrowser", self.style.clone(), UiNodeData::Container);

        let header_style = Style::new()
            .with_layout(LayoutStyle::new().with_direction(FlexDirection::Row).with_padding(4.0))
            .with_font(FontStyle::new().with_size(14.0));

        let header_id = tree.create_node(
            "AssetBrowser_Header",
            header_style,
            UiNodeData::Text { content: format!("Assets: {}", self.path) },
        );

        tree.add_child(root_id, header_id);

        let search_style = Style::new()
            .with_background_color(Color::new(0.25, 0.25, 0.25, 1.0))
            .with_border_color(Color::new(0.4, 0.4, 0.4, 1.0))
            .with_border_width(1.0)
            .with_corner_radius(2.0)
            .with_font(FontStyle::new().with_size(12.0));

        let search_id = tree.create_node(
            "AssetBrowser_Search",
            search_style,
            UiNodeData::Text {
                content: if self.search_query.is_empty() { "Search...".to_string() } else { self.search_query.clone() },
            },
        );

        tree.add_child(root_id, search_id);

        for asset in &self.assets {
            let row_style = Style::new()
                .with_layout(
                    LayoutStyle::new()
                        .with_direction(FlexDirection::Row)
                        .with_align_items(FlexAlign::Center)
                        .with_gap(4.0)
                        .with_padding(2.0),
                )
                .with_font(FontStyle::new().with_size(12.0));

            let row_id = tree.create_node(format!("AssetBrowser_Item({})", asset.name), row_style, UiNodeData::Container);

            let icon_id = tree.create_node(
                format!("AssetBrowser_Icon({})", asset.name),
                Style::new().with_font(FontStyle::new().with_size(12.0)),
                UiNodeData::Text { content: asset.icon.clone() },
            );

            let name_id = tree.create_node(
                format!("AssetBrowser_Name({})", asset.name),
                Style::new().with_font(FontStyle::new().with_size(12.0)),
                UiNodeData::Text { content: asset.name.clone() },
            );

            tree.add_child(row_id, icon_id);
            tree.add_child(row_id, name_id);
            tree.add_child(root_id, row_id);
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

/// 标签页条目
///
/// 表示标签页容器中的单个标签页，包含标题和是否可关闭。
pub struct TabEntry {
    /// 标签页标题
    pub title: String,
    /// 是否可关闭
    pub closable: bool,
}

impl TabEntry {
    /// 创建标签页条目
    ///
    /// # 参数
    ///
    /// - `title` - 标签页标题
    pub fn new(title: impl Into<String>) -> Self {
        Self { title: title.into(), closable: false }
    }

    /// 设置是否可关闭
    pub fn with_closable(mut self, closable: bool) -> Self {
        self.closable = closable;
        self
    }
}

/// 标签页容器
///
/// 提供标签栏和内容区域的组合容器，支持标签页切换。
pub struct TabContainer {
    /// 标签页列表
    pub tabs: Vec<TabEntry>,
    /// 活跃标签索引
    pub active_tab: usize,
    /// 标签切换回调
    pub on_tab_change: Option<Box<dyn FnMut(usize) + Send + Sync>>,
    /// 样式
    pub style: Style,
    /// 根节点 ID
    node_id: Option<UiNodeId>,
}

impl TabContainer {
    /// 创建标签页容器
    pub fn new() -> Self {
        let style = Style::new()
            .with_layout(LayoutStyle::new().with_direction(FlexDirection::Column).with_gap(0.0))
            .with_font(FontStyle::new());

        Self { tabs: Vec::new(), active_tab: 0, on_tab_change: None, style, node_id: None }
    }

    /// 添加标签页
    pub fn add_tab(mut self, title: impl Into<String>) -> Self {
        self.tabs.push(TabEntry::new(title));
        self
    }

    /// 设置活跃标签索引
    pub fn with_active(mut self, index: usize) -> Self {
        self.active_tab = index;
        self
    }

    /// 设置标签切换回调
    pub fn with_on_tab_change(mut self, on_tab_change: Box<dyn FnMut(usize) + Send + Sync>) -> Self {
        self.on_tab_change = Some(on_tab_change);
        self
    }

    /// 设置样式
    pub fn with_style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }
}

impl Default for TabContainer {
    fn default() -> Self {
        Self::new()
    }
}

impl Widget for TabContainer {
    fn build(&mut self, tree: &mut UiTree) -> GResult<UiNodeId> {
        let root_id = tree.create_node("TabContainer", self.style.clone(), UiNodeData::Container);

        let tab_bar_style = Style::new()
            .with_layout(LayoutStyle::new().with_direction(FlexDirection::Row).with_gap(2.0).with_padding(2.0))
            .with_background_color(Color::new(0.15, 0.15, 0.15, 1.0))
            .with_font(self.style.font.clone().unwrap_or_default());

        let tab_bar_id = tree.create_node("TabContainer_TabBar", tab_bar_style, UiNodeData::Container);

        for (i, tab) in self.tabs.iter().enumerate() {
            let is_active = i == self.active_tab;
            let bg = if is_active { Color::new(0.3, 0.3, 0.3, 1.0) } else { Color::new(0.2, 0.2, 0.2, 1.0) };

            let tab_style = Style::new()
                .with_background_color(bg)
                .with_corner_radius(4.0)
                .with_layout(LayoutStyle::new().with_direction(FlexDirection::Row).with_padding(6.0))
                .with_font(FontStyle::new().with_size(12.0));

            let tab_id = tree.create_node(
                format!("TabContainer_Tab({})", tab.title),
                tab_style,
                UiNodeData::Text { content: tab.title.clone() },
            );

            tree.add_child(tab_bar_id, tab_id);
        }

        tree.add_child(root_id, tab_bar_id);

        let content_style = Style::new()
            .with_layout(LayoutStyle::new().with_padding(4.0))
            .with_background_color(Color::new(0.2, 0.2, 0.2, 1.0));

        let content_id = tree.create_node("TabContainer_Content", content_style, UiNodeData::Container);

        tree.add_child(root_id, content_id);

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
            if !self.tabs.is_empty() {
                self.active_tab = (self.active_tab + 1) % self.tabs.len();
                if let Some(ref mut cb) = self.on_tab_change {
                    cb(self.active_tab);
                }
            }
        }
    }
}

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

/// 属性类型
///
/// 定义属性字段支持的数据类型，用于选择合适的编辑器控件。
#[derive(Debug, Clone)]
pub enum PropertyType {
    /// 字符串类型
    String,
    /// 浮点数类型
    Float,
    /// 布尔值类型
    Bool,
    /// 三维向量类型
    Vec3,
    /// 颜色类型
    Color,
    /// 枚举类型
    Enum(Vec<String>),
    /// 资源引用类型
    Resource,
}

/// 属性字段编辑器
///
/// 提供单个属性的标签-编辑器组合，根据属性类型渲染不同的输入控件。
pub struct PropertyField {
    /// 属性标签
    pub label: String,
    /// 属性类型
    pub prop_type: PropertyType,
    /// 属性值
    pub value: String,
    /// 值变更回调
    pub on_change: Option<Box<dyn FnMut(String) + Send + Sync>>,
    /// 样式
    pub style: Style,
    /// 根节点 ID
    node_id: Option<UiNodeId>,
}

impl PropertyField {
    /// 创建属性字段编辑器
    ///
    /// # 参数
    ///
    /// - `label` - 属性标签文本
    /// - `prop_type` - 属性类型
    /// - `value` - 属性当前值
    pub fn new(label: impl Into<String>, prop_type: PropertyType, value: impl Into<String>) -> Self {
        let style = Style::new()
            .with_layout(
                LayoutStyle::new()
                    .with_direction(FlexDirection::Row)
                    .with_align_items(FlexAlign::Center)
                    .with_gap(8.0)
                    .with_padding(4.0),
            )
            .with_font(FontStyle::new().with_size(12.0));

        Self { label: label.into(), prop_type, value: value.into(), on_change: None, style, node_id: None }
    }

    /// 设置值变更回调
    pub fn with_on_change(mut self, on_change: Box<dyn FnMut(String) + Send + Sync>) -> Self {
        self.on_change = Some(on_change);
        self
    }

    /// 设置样式
    pub fn with_style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }
}

impl Widget for PropertyField {
    fn build(&mut self, tree: &mut UiTree) -> GResult<UiNodeId> {
        let root_id = tree.create_node("PropertyField", self.style.clone(), UiNodeData::Container);

        let label_style = Style::new().with_font(FontStyle::new().with_size(12.0));

        let label_id = tree.create_node(
            format!("PropertyField_Label({})", self.label),
            label_style,
            UiNodeData::Text { content: format!("{}:", self.label) },
        );

        tree.add_child(root_id, label_id);

        let editor_style = Style::new()
            .with_background_color(Color::new(0.25, 0.25, 0.25, 1.0))
            .with_border_color(Color::new(0.4, 0.4, 0.4, 1.0))
            .with_border_width(1.0)
            .with_corner_radius(2.0)
            .with_font(FontStyle::new().with_size(12.0));

        let display_value = match &self.prop_type {
            PropertyType::Bool => if self.value == "true" { "☑ true" } else { "☐ false" }.to_string(),
            PropertyType::Color => format!("■ {}", self.value),
            PropertyType::Enum(variants) => {
                format!("{} ▼", variants.first().unwrap_or(&self.value))
            }
            PropertyType::Resource => format!("📎 {}", self.value),
            _ => self.value.clone(),
        };

        let editor_id = tree.create_node(
            format!("PropertyField_Editor({})", self.label),
            editor_style,
            UiNodeData::Text { content: display_value },
        );

        tree.add_child(root_id, editor_id);

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
            match &self.prop_type {
                PropertyType::Bool => {
                    self.value = if self.value == "true" { "false".to_string() } else { "true".to_string() };
                    if let Some(ref mut cb) = self.on_change {
                        cb(self.value.clone());
                    }
                }
                _ => {}
            }
        }
    }
}

/// 标签
///
/// 表示标签栏中的单个标签项，包含标签文本和是否可关闭。
pub struct Tab {
    /// 标签标签
    pub label: String,
    /// 是否可关闭
    pub closable: bool,
}

impl Tab {
    /// 创建标签
    ///
    /// # 参数
    ///
    /// - `label` - 标签显示文本
    pub fn new(label: impl Into<String>) -> Self {
        Self { label: label.into(), closable: false }
    }

    /// 设置是否可关闭
    pub fn with_closable(mut self, closable: bool) -> Self {
        self.closable = closable;
        self
    }
}

/// 编辑器标签栏组件
///
/// 提供水平排列的标签栏，用于管理编辑器中打开的文档或面板标签页。
pub struct EditorTabBar {
    /// 标签列表
    pub tabs: Vec<Tab>,
    /// 活跃标签索引
    pub active_index: Option<usize>,
    /// 标签点击回调
    pub on_tab_click: Option<Box<dyn FnMut(usize) + Send + Sync>>,
    /// 标签关闭回调
    pub on_tab_close: Option<Box<dyn FnMut(usize) + Send + Sync>>,
    /// 样式
    pub style: Style,
    /// 根节点 ID
    node_id: Option<UiNodeId>,
}

impl EditorTabBar {
    /// 创建空标签栏
    pub fn new() -> Self {
        let style = Style::new()
            .with_layout(
                LayoutStyle::new().with_direction(FlexDirection::Row).with_align_items(FlexAlign::Center).with_gap(2.0),
            )
            .with_font(FontStyle::new());

        Self { tabs: Vec::new(), active_index: None, on_tab_click: None, on_tab_close: None, style, node_id: None }
    }

    /// 添加标签
    pub fn add_tab(mut self, label: impl Into<String>, closable: bool) -> Self {
        self.tabs.push(Tab::new(label).with_closable(closable));
        self
    }

    /// 设置活跃标签索引
    pub fn with_active(mut self, index: usize) -> Self {
        self.active_index = Some(index);
        self
    }

    /// 设置标签点击回调
    pub fn with_on_tab_click(mut self, callback: Box<dyn FnMut(usize) + Send + Sync>) -> Self {
        self.on_tab_click = Some(callback);
        self
    }

    /// 设置标签关闭回调
    pub fn with_on_tab_close(mut self, callback: Box<dyn FnMut(usize) + Send + Sync>) -> Self {
        self.on_tab_close = Some(callback);
        self
    }

    /// 设置样式
    pub fn with_style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }

    /// 设置活跃标签
    pub fn set_active(&mut self, index: usize) {
        if index < self.tabs.len() {
            self.active_index = Some(index);
        }
    }
}

impl Default for EditorTabBar {
    fn default() -> Self {
        Self::new()
    }
}

impl Widget for EditorTabBar {
    fn build(&mut self, tree: &mut UiTree) -> GResult<UiNodeId> {
        let root_id = tree.create_node("EditorTabBar", self.style.clone(), UiNodeData::Container);

        for (i, tab) in self.tabs.iter().enumerate() {
            let is_active = self.active_index == Some(i);

            let tab_bg = if is_active { Color::new(0.3, 0.3, 0.3, 1.0) } else { Color::new(0.2, 0.2, 0.2, 1.0) };

            let tab_border = if is_active { Color::new(0.26, 0.52, 0.96, 1.0) } else { Color::new(0.4, 0.4, 0.4, 1.0) };

            let tab_style = Style::new()
                .with_background_color(tab_bg)
                .with_border_color(tab_border)
                .with_border_width(if is_active { 2.0 } else { 1.0 })
                .with_corner_radius(4.0)
                .with_layout(
                    LayoutStyle::new()
                        .with_direction(FlexDirection::Row)
                        .with_align_items(FlexAlign::Center)
                        .with_gap(6.0)
                        .with_padding(8.0),
                )
                .with_font(self.style.font.clone().unwrap_or_default());

            let tab_id = tree.create_node(format!("EditorTabBar_Tab({})", tab.label), tab_style, UiNodeData::Container);

            let label_style = Style::new().with_font(FontStyle::new());

            let label_id = tree.create_node(
                format!("EditorTabBar_TabLabel({})", tab.label),
                label_style,
                UiNodeData::Text { content: tab.label.clone() },
            );

            tree.add_child(tab_id, label_id);

            if tab.closable {
                let close_style = Style::new().with_font(FontStyle::new().with_size(12.0));

                let close_id = tree.create_node(
                    format!("EditorTabBar_TabClose({})", tab.label),
                    close_style,
                    UiNodeData::Text { content: "×".to_string() },
                );

                tree.add_child(tab_id, close_id);
            }

            tree.add_child(root_id, tab_id);
        }

        self.node_id = Some(root_id);
        Ok(root_id)
    }

    fn update(&self, tree: &mut UiTree) {
        let _ = tree;
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
