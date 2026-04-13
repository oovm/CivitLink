use gg_core::GResult;
use gg_render::Color;
use gg_ui::{
    EventContext, FlexAlign, FlexDirection, FontStyle, GuiEvent, LayoutStyle, Style, UiNodeData, UiNodeId, UiTree, Widget,
};

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
