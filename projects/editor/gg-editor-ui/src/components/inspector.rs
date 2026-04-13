use gg_core::GResult;
use gg_render::Color;
use gg_ui::{
    EventContext, FlexAlign, FlexDirection, FontStyle, GuiEvent, LayoutStyle, Style, UiNodeData, UiNodeId, UiTree, Widget,
};

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
