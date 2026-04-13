use crate::{
    gui_event::{EventContext, GuiEvent, MouseButton},
    node::{UiNodeData, UiNodeId, UiTree},
    reactive::Signal,
    style::{FlexAlign, FlexDirection, FontStyle, LayoutStyle, Style},
    widget::Widget,
};
use gg_core::GResult;

/// 下拉菜单组件
///
/// 提供可展开的选项列表，用户可以从中选择一个选项。
pub struct Dropdown {
    /// 选项列表
    pub options: Vec<String>,
    /// 当前选中索引
    pub selected_index: Signal<Option<usize>>,
    /// 占位文本
    pub placeholder: String,
    /// 选项变更回调
    pub on_change: Option<Box<dyn FnMut(usize) + Send + Sync>>,
    /// 样式
    pub style: Style,
    /// 根节点 ID
    node_id: Option<UiNodeId>,
    /// 下拉状态
    is_open: bool,
    /// 显示行节点 ID
    display_row_id: Option<UiNodeId>,
    /// 选中文本节点 ID
    selected_text_id: Option<UiNodeId>,
    /// 箭头节点 ID
    arrow_id: Option<UiNodeId>,
    /// 下拉列表节点 ID
    dropdown_list_id: Option<UiNodeId>,
}

impl Dropdown {
    /// 创建下拉菜单
    ///
    /// # 参数
    ///
    /// - `options` - 选项文本列表
    pub fn new(options: Vec<String>) -> Self {
        let style = Style::new()
            .with_background_color(gg_render::Color::new(0.2, 0.2, 0.2, 1.0))
            .with_border_color(gg_render::Color::new(0.5, 0.5, 0.5, 1.0))
            .with_border_width(1.0)
            .with_corner_radius(4.0)
            .with_layout(LayoutStyle::new().with_direction(FlexDirection::Column).with_padding(8.0))
            .with_font(FontStyle::new());

        Self {
            options,
            selected_index: Signal::new(None),
            placeholder: String::new(),
            on_change: None,
            style,
            node_id: None,
            is_open: false,
            display_row_id: None,
            selected_text_id: None,
            arrow_id: None,
            dropdown_list_id: None,
        }
    }

    /// 设置占位文本
    pub fn with_placeholder(mut self, placeholder: impl Into<String>) -> Self {
        self.placeholder = placeholder.into();
        self
    }

    /// 设置样式
    pub fn with_style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }

    /// 设置选项变更回调
    pub fn with_on_change(mut self, callback: Box<dyn FnMut(usize) + Send + Sync>) -> Self {
        self.on_change = Some(callback);
        self
    }

    /// 切换下拉状态
    pub fn toggle_open(&mut self) {
        self.is_open = !self.is_open;
    }

    /// 关闭下拉
    pub fn close(&mut self) {
        self.is_open = false;
    }

    /// 获取下拉状态
    pub fn is_open(&self) -> bool {
        self.is_open
    }
}

impl Widget for Dropdown {
    fn build(&mut self, tree: &mut UiTree) -> GResult<UiNodeId> {
        let root_id = tree.create_node("Dropdown", self.style.clone(), UiNodeData::Container);

        let display_row_style = Style::new().with_layout(
            LayoutStyle::new()
                .with_direction(FlexDirection::Row)
                .with_align_items(FlexAlign::Center)
                .with_justify_content(FlexAlign::SpaceBetween),
        );

        let display_row_id = tree.create_node("Dropdown_DisplayRow", display_row_style, UiNodeData::Container);

        let selected_text =
            self.selected_index.get().and_then(|i| self.options.get(i)).cloned().unwrap_or_else(|| self.placeholder.clone());

        let selected_text_style = Style::new().with_font(self.style.font.clone().unwrap_or_default());

        let selected_text_id =
            tree.create_node("Dropdown_SelectedText", selected_text_style, UiNodeData::Text { content: selected_text });

        let arrow_style = Style::new().with_font(FontStyle::new().with_size(12.0));

        let arrow_id = tree.create_node("Dropdown_Arrow", arrow_style, UiNodeData::Text { content: "▼".to_string() });

        tree.add_child(display_row_id, selected_text_id);
        tree.add_child(display_row_id, arrow_id);
        tree.add_child(root_id, display_row_id);

        let mut dropdown_list_id = None;

        if self.is_open {
            let dropdown_style = Style::new()
                .with_background_color(gg_render::Color::new(0.25, 0.25, 0.25, 1.0))
                .with_border_color(gg_render::Color::new(0.4, 0.4, 0.4, 1.0))
                .with_border_width(1.0)
                .with_corner_radius(4.0)
                .with_layout(LayoutStyle::new().with_direction(FlexDirection::Column).with_padding(4.0).with_gap(2.0));

            let list_id = tree.create_node("Dropdown_List", dropdown_style, UiNodeData::Container);

            for option in &self.options {
                let option_style = Style::new()
                    .with_corner_radius(2.0)
                    .with_layout(LayoutStyle::new().with_direction(FlexDirection::Row).with_padding(6.0))
                    .with_font(self.style.font.clone().unwrap_or_default());

                let option_id = tree.create_node(
                    format!("Dropdown_Option({})", option),
                    option_style,
                    UiNodeData::Text { content: option.clone() },
                );

                tree.add_child(list_id, option_id);
            }

            tree.add_child(root_id, list_id);
            dropdown_list_id = Some(list_id);
        }

        self.node_id = Some(root_id);
        self.display_row_id = Some(display_row_id);
        self.selected_text_id = Some(selected_text_id);
        self.arrow_id = Some(arrow_id);
        self.dropdown_list_id = dropdown_list_id;

        Ok(root_id)
    }

    fn update(&self, tree: &mut UiTree) {
        if let Some(selected_text_id) = self.selected_text_id {
            let selected_text = self
                .selected_index
                .get()
                .and_then(|i| self.options.get(i))
                .cloned()
                .unwrap_or_else(|| self.placeholder.clone());

            if let Some(node) = tree.get_mut(selected_text_id) {
                if let UiNodeData::Text { ref mut content } = node.data {
                    *content = selected_text;
                }
            }
        }

        if let Some(arrow_id) = self.arrow_id {
            let arrow_text = if self.is_open { "▲" } else { "▼" };
            if let Some(node) = tree.get_mut(arrow_id) {
                if let UiNodeData::Text { ref mut content } = node.data {
                    *content = arrow_text.to_string();
                }
            }
        }

        if let Some(dropdown_list_id) = self.dropdown_list_id {
            if let Some(node) = tree.get_mut(dropdown_list_id) {
                node.visible = self.is_open;
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
        match event {
            GuiEvent::MouseClick { button: MouseButton::Left, .. } => {
                if self.is_open {
                    self.close();
                } else {
                    self.toggle_open();
                }
            }
            _ => {}
        }
    }
}
