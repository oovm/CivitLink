use gg_core::GResult;
use gg_render::Color;
use gg_ui::{
    EventContext, FlexAlign, FlexDirection, FontStyle, GuiEvent, LayoutStyle, Style, UiNodeData, UiNodeId, UiTree, Widget,
};

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
