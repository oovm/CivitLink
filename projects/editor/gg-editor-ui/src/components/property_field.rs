use gg_core::GResult;
use gg_render::Color;
use gg_ui::{
    EventContext, FlexAlign, FlexDirection, FontStyle, GuiEvent, LayoutStyle, Style, UiNodeData, UiNodeId, UiTree, Widget,
};

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
