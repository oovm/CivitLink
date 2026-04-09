use crate::event::EventSystem;
use crate::node::{UiNodeData, UiTree};
use crate::style::{FlexDirection, FontStyle, LayoutStyle, Style};
use crate::widget::Widget;

/// 按钮交互状态
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ButtonState {
    /// 正常
    Normal,
    /// 悬停
    Hovered,
    /// 按下
    Pressed,
}

/// 按钮控件
///
/// 提供可点击的按钮 UI 元素，支持正常、悬停和按下三种状态的样式切换。
pub struct Button {
    /// 文本标签
    pub label: String,
    /// 正常样式
    pub style: Style,
    /// 悬停样式
    pub hover_style: Style,
    /// 按下样式
    pub pressed_style: Style,
    /// 点击回调
    pub on_click: Option<Box<dyn FnMut() + Send + Sync>>,
    /// 根节点 ID
    node_id: Option<crate::node::UiNodeId>,
    /// 当前交互状态
    state: ButtonState,
}

impl Button {
    /// 创建按钮控件
    ///
    /// # 参数
    ///
    /// - `label` - 按钮文本标签
    pub fn new(label: impl Into<String>) -> Self {
        let style = Style::new()
            .with_background_color(gg_render::Color::new(0.2, 0.2, 0.2, 1.0))
            .with_corner_radius(4.0)
            .with_layout(
                LayoutStyle::new()
                    .with_direction(FlexDirection::Column)
                    .with_padding(8.0),
            )
            .with_font(FontStyle::new());

        let hover_style = Style::new()
            .with_background_color(gg_render::Color::new(0.3, 0.3, 0.3, 1.0))
            .with_corner_radius(4.0)
            .with_layout(
                LayoutStyle::new()
                    .with_direction(FlexDirection::Column)
                    .with_padding(8.0),
            )
            .with_font(FontStyle::new());

        let pressed_style = Style::new()
            .with_background_color(gg_render::Color::new(0.15, 0.15, 0.15, 1.0))
            .with_corner_radius(4.0)
            .with_layout(
                LayoutStyle::new()
                    .with_direction(FlexDirection::Column)
                    .with_padding(8.0),
            )
            .with_font(FontStyle::new());

        Self {
            label: label.into(),
            style,
            hover_style,
            pressed_style,
            on_click: None,
            node_id: None,
            state: ButtonState::Normal,
        }
    }

    /// 设置点击回调
    pub fn on_click(mut self, callback: Box<dyn FnMut() + Send + Sync>) -> Self {
        self.on_click = Some(callback);
        self
    }

    /// 设置正常样式
    pub fn with_style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }

    /// 设置悬停样式
    pub fn with_hover_style(mut self, style: Style) -> Self {
        self.hover_style = style;
        self
    }

    /// 设置按下样式
    pub fn with_pressed_style(mut self, style: Style) -> Self {
        self.pressed_style = style;
        self
    }

    /// 获取当前交互状态
    pub fn state(&self) -> ButtonState {
        self.state
    }

    /// 注册事件处理器到事件系统
    pub fn register_events(&mut self, event_system: &mut EventSystem, tree: &UiTree) {
        if let Some(id) = self.node_id {
            let label = self.label.clone();
            let on_click = self.on_click.take();
            let mut on_click = on_click;
            event_system.register(
                id,
                Box::new(move |event| {
                    match event {
                        crate::event::UiEvent::Click { .. } => {
                            if let Some(ref mut cb) = on_click {
                                cb();
                            }
                            true
                        }
                        _ => false,
                    }
                }),
            );
            let _ = label;
            let _ = tree;
        }
    }
}

impl Widget for Button {
    fn build(&self, tree: &mut UiTree) -> crate::node::UiNodeId {
        let root_id = tree.create_node(
            format!("Button({})", self.label),
            self.style.clone(),
            UiNodeData::Container,
        );

        let text_id = tree.create_node(
            format!("Button_Text({})", self.label),
            Style::new().with_font(
                self.style
                    .font
                    .clone()
                    .unwrap_or_default(),
            ),
            UiNodeData::Text {
                content: self.label.clone(),
            },
        );

        tree.add_child(root_id, text_id);

        root_id
    }

    fn update(&self, tree: &mut UiTree) {
        if let Some(id) = self.node_id {
            let current_style = match self.state {
                ButtonState::Normal => self.style.clone(),
                ButtonState::Hovered => self.hover_style.clone(),
                ButtonState::Pressed => self.pressed_style.clone(),
            };
            if let Some(node) = tree.get_mut(id) {
                node.style = current_style;
            }
        }
    }

    fn node_id(&self) -> Option<crate::node::UiNodeId> {
        self.node_id
    }
}
