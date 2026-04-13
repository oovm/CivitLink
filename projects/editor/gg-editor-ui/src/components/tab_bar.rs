use gg_core::GResult;
use gg_render::Color;
use gg_ui::{
    EventContext, FlexAlign, FlexDirection, FontStyle, GuiEvent, LayoutStyle, Style, UiNodeData, UiNodeId, UiTree, Widget,
};

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
