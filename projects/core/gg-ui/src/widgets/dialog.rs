use crate::{
    gui_event::{EventContext, GuiEvent, MouseButton},
    node::{UiNodeData, UiNodeId, UiTree},
    reactive::Signal,
    style::{FlexAlign, FlexDirection, FontStyle, LayoutStyle, SizeValue, Style},
    widget::Widget,
};
use gg_core::GResult;

/// 对话框按钮
pub struct DialogButton {
    /// 按钮标签
    pub label: String,
    /// 点击回调
    pub on_click: Option<Box<dyn FnMut() + Send + Sync>>,
}

/// 模态对话框组件
///
/// 提供模态对话框 UI 元素，支持标题、内容区域和按钮列表。
pub struct Dialog {
    /// 对话框标题
    pub title: String,
    /// 内容节点 ID
    pub content: Option<UiNodeId>,
    /// 按钮列表
    pub buttons: Vec<DialogButton>,
    /// 打开状态
    pub is_open: Signal<bool>,
    /// 关闭回调
    pub on_close: Option<Box<dyn FnMut() + Send + Sync>>,
    /// 样式
    pub style: Style,
    /// 根节点 ID
    node_id: Option<UiNodeId>,
    /// 标题节点 ID
    title_node_id: Option<UiNodeId>,
    /// 按钮行节点 ID
    button_row_id: Option<UiNodeId>,
}

impl Dialog {
    /// 创建对话框
    ///
    /// # 参数
    ///
    /// - `title` - 对话框标题文本
    pub fn new(title: impl Into<String>) -> Self {
        let style = Style::new()
            .with_background_color(gg_render::Color::new(0.15, 0.15, 0.15, 1.0))
            .with_corner_radius(8.0)
            .with_layout(
                LayoutStyle::new()
                    .with_direction(FlexDirection::Column)
                    .with_padding(16.0)
                    .with_gap(12.0)
                    .with_min_width(SizeValue::Px(300.0)),
            )
            .with_font(FontStyle::new());

        Self {
            title: title.into(),
            content: None,
            buttons: Vec::new(),
            is_open: Signal::new(false),
            on_close: None,
            style,
            node_id: None,
            title_node_id: None,
            button_row_id: None,
        }
    }

    /// 设置样式
    pub fn with_style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }

    /// 添加按钮
    pub fn add_button(mut self, label: impl Into<String>, on_click: Box<dyn FnMut() + Send + Sync>) -> Self {
        self.buttons.push(DialogButton { label: label.into(), on_click: Some(on_click) });
        self
    }

    /// 打开对话框
    pub fn open(&mut self) {
        self.is_open.set(true);
    }

    /// 关闭对话框
    pub fn close(&mut self) {
        self.is_open.set(false);
    }
}

impl Widget for Dialog {
    fn build(&mut self, tree: &mut UiTree) -> GResult<UiNodeId> {
        let overlay_style = Style::new().with_background_color(gg_render::Color::new(0.0, 0.0, 0.0, 0.5)).with_layout(
            LayoutStyle::new()
                .with_direction(FlexDirection::Column)
                .with_align_items(FlexAlign::Center)
                .with_justify_content(FlexAlign::Center)
                .with_width(SizeValue::Percent(1.0))
                .with_height(SizeValue::Percent(1.0)),
        );

        let overlay_id = tree.create_node("Dialog_Overlay", overlay_style, UiNodeData::Container);

        let container_id = tree.create_node("Dialog", self.style.clone(), UiNodeData::Container);

        let title_style = Style::new()
            .with_font(FontStyle::new().with_size(18.0))
            .with_layout(LayoutStyle::new().with_direction(FlexDirection::Row).with_padding(4.0));

        let title_id = tree.create_node("Dialog_Title", title_style, UiNodeData::Text { content: self.title.clone() });

        tree.add_child(container_id, title_id);

        let content_style =
            Style::new().with_layout(LayoutStyle::new().with_direction(FlexDirection::Column).with_padding(8.0));

        let content_id = tree.create_node("Dialog_Content", content_style, UiNodeData::Container);

        tree.add_child(container_id, content_id);
        self.content = Some(content_id);

        let button_row_style = Style::new().with_layout(
            LayoutStyle::new()
                .with_direction(FlexDirection::Row)
                .with_justify_content(FlexAlign::End)
                .with_gap(8.0)
                .with_padding(4.0),
        );

        let button_row_id = tree.create_node("Dialog_ButtonRow", button_row_style, UiNodeData::Container);

        for button in &self.buttons {
            let btn_style = Style::new()
                .with_background_color(gg_render::Color::new(0.26, 0.52, 0.96, 1.0))
                .with_corner_radius(4.0)
                .with_layout(LayoutStyle::new().with_direction(FlexDirection::Row).with_padding(8.0))
                .with_font(FontStyle::new());

            let btn_id = tree.create_node(
                format!("Dialog_Button({})", button.label),
                btn_style,
                UiNodeData::Text { content: button.label.clone() },
            );

            tree.add_child(button_row_id, btn_id);
        }

        tree.add_child(container_id, button_row_id);
        tree.add_child(overlay_id, container_id);

        self.node_id = Some(overlay_id);
        self.title_node_id = Some(title_id);
        self.button_row_id = Some(button_row_id);

        Ok(overlay_id)
    }

    fn update(&self, tree: &mut UiTree) {
        if let Some(node_id) = self.node_id {
            if let Some(node) = tree.get_mut(node_id) {
                node.visible = *self.is_open.get();
            }
        }

        if let Some(title_node_id) = self.title_node_id {
            if let Some(node) = tree.get_mut(title_node_id) {
                if let UiNodeData::Text { ref mut content } = node.data {
                    *content = self.title.clone();
                }
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
            self.close();
            if let Some(ref mut cb) = self.on_close {
                cb();
            }
        }
    }
}
