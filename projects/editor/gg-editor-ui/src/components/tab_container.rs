use gg_core::GResult;
use gg_render::Color;
use gg_ui::{EventContext, FlexDirection, FontStyle, GuiEvent, LayoutStyle, Style, UiNodeData, UiNodeId, UiTree, Widget};

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
