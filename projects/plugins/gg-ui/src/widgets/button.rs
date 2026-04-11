use std::sync::{Arc, Mutex};

use crate::{
    event::{EventSystem, UiEvent},
    layout::LayoutResult,
    node::{UiNodeData, UiNodeId, UiTree},
    style::{FlexDirection, FontStyle, LayoutStyle, Style},
    widget::Widget,
};
use gg_core::GResult;

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
    node_id: Option<UiNodeId>,
    /// 当前交互状态
    state: Arc<Mutex<ButtonState>>,
    /// 布局边界缓存
    layout_bounds: Arc<Mutex<Option<LayoutResult>>>,
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
            .with_layout(LayoutStyle::new().with_direction(FlexDirection::Column).with_padding(8.0))
            .with_font(FontStyle::new());

        let hover_style = Style::new()
            .with_background_color(gg_render::Color::new(0.3, 0.3, 0.3, 1.0))
            .with_corner_radius(4.0)
            .with_layout(LayoutStyle::new().with_direction(FlexDirection::Column).with_padding(8.0))
            .with_font(FontStyle::new());

        let pressed_style = Style::new()
            .with_background_color(gg_render::Color::new(0.15, 0.15, 0.15, 1.0))
            .with_corner_radius(4.0)
            .with_layout(LayoutStyle::new().with_direction(FlexDirection::Column).with_padding(8.0))
            .with_font(FontStyle::new());

        Self {
            label: label.into(),
            style,
            hover_style,
            pressed_style,
            on_click: None,
            node_id: None,
            state: Arc::new(Mutex::new(ButtonState::Normal)),
            layout_bounds: Arc::new(Mutex::new(None)),
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
        *self.state.lock().unwrap()
    }

    /// 注册事件处理器到事件系统
    pub fn register_events(&mut self, event_system: &mut EventSystem, _tree: &UiTree) {
        if let Some(id) = self.node_id {
            let state = Arc::clone(&self.state);
            let layout_bounds = Arc::clone(&self.layout_bounds);
            let on_click = self.on_click.take();
            let mut on_click = on_click;

            event_system.register(
                id,
                Box::new(move |event| match event {
                    UiEvent::MouseMove { x, y } => {
                        let inside = is_inside(*x, *y, layout_bounds.lock().unwrap().as_ref());
                        if inside {
                            *state.lock().unwrap() = ButtonState::Hovered;
                        } else {
                            *state.lock().unwrap() = ButtonState::Normal;
                        }
                        true
                    }
                    UiEvent::MouseDown { x, y } => {
                        let inside = is_inside(*x, *y, layout_bounds.lock().unwrap().as_ref());
                        if inside {
                            *state.lock().unwrap() = ButtonState::Pressed;
                        }
                        true
                    }
                    UiEvent::MouseUp { x, y } => {
                        let inside = is_inside(*x, *y, layout_bounds.lock().unwrap().as_ref());
                        if inside {
                            *state.lock().unwrap() = ButtonState::Hovered;
                            if let Some(ref mut cb) = on_click {
                                cb();
                            }
                        } else {
                            *state.lock().unwrap() = ButtonState::Normal;
                        }
                        true
                    }
                    UiEvent::Click { .. } => {
                        if let Some(ref mut cb) = on_click {
                            cb();
                        }
                        true
                    }
                    _ => false,
                }),
            );
        }
    }
}

fn is_inside(x: f32, y: f32, layout: Option<&LayoutResult>) -> bool {
    match layout {
        Some(l) => x >= l.x && x <= l.x + l.width && y >= l.y && y <= l.y + l.height,
        None => false,
    }
}

impl Widget for Button {
    fn build(&mut self, tree: &mut UiTree) -> GResult<UiNodeId> {
        let root_id = tree.create_node(format!("Button({})", self.label), self.style.clone(), UiNodeData::Container);

        let text_id = tree.create_node(
            format!("Button_Text({})", self.label),
            Style::new().with_font(self.style.font.clone().unwrap_or_default()),
            UiNodeData::Text { content: self.label.clone() },
        );

        tree.add_child(root_id, text_id);

        self.node_id = Some(root_id);
        Ok(root_id)
    }

    fn update(&self, tree: &mut UiTree) {
        if let Some(id) = self.node_id {
            if let Some(node) = tree.get(id) {
                *self.layout_bounds.lock().unwrap() = node.layout_result;
            }

            let current_style = match *self.state.lock().unwrap() {
                ButtonState::Normal => self.style.clone(),
                ButtonState::Hovered => self.hover_style.clone(),
                ButtonState::Pressed => self.pressed_style.clone(),
            };
            if let Some(node) = tree.get_mut(id) {
                node.style = current_style;
            }
        }
    }

    fn node_id(&self) -> Option<UiNodeId> {
        self.node_id
    }
}
