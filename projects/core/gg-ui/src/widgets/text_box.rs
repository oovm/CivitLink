use crate::{
    gui_event::{EventContext, GuiEvent, Key, MouseButton},
    node::{UiNodeData, UiNodeId, UiTree},
    style::{FontStyle, LayoutStyle, SizeValue, Style},
    widget::Widget,
};
use gg_core::GResult;

/// 文本框控件
///
/// 提供文本显示和输入功能的 UI 元素，支持光标定位、文本选择和键盘输入处理。
pub struct TextBox {
    /// 文本内容
    pub text: String,
    /// 样式
    pub style: Style,
    /// 最大宽度（None 表示不限制）
    pub max_width: Option<f32>,
    /// 是否自动换行
    pub word_wrap: bool,
    /// 光标位置（字节索引）
    pub cursor_position: usize,
    /// 光标是否可见
    pub cursor_visible: bool,
    /// 光标闪烁计时器
    pub cursor_blink_timer: f32,
    /// 选区起始位置
    pub selection_start: Option<usize>,
    /// 是否获得焦点
    pub is_focused: bool,
    /// 文本变更回调
    pub on_text_change: Option<Box<dyn FnMut(&str) + Send + Sync>>,
    /// 根节点 ID
    node_id: Option<UiNodeId>,
}

impl TextBox {
    /// 创建文本框控件
    ///
    /// # 参数
    ///
    /// - `text` - 初始文本内容
    pub fn new(text: impl Into<String>) -> Self {
        let style = Style::new().with_layout(LayoutStyle::new().with_padding(4.0)).with_font(FontStyle::new());

        Self {
            text: text.into(),
            style,
            max_width: None,
            word_wrap: true,
            cursor_position: 0,
            cursor_visible: true,
            cursor_blink_timer: 0.0,
            selection_start: None,
            is_focused: false,
            on_text_change: None,
            node_id: None,
        }
    }

    /// 设置最大宽度
    pub fn with_max_width(mut self, width: f32) -> Self {
        self.max_width = Some(width);
        self
    }

    /// 设置是否自动换行
    pub fn with_word_wrap(mut self, wrap: bool) -> Self {
        self.word_wrap = wrap;
        self
    }

    /// 设置样式
    pub fn with_style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }

    /// 处理键盘输入
    ///
    /// 支持字符插入、退格删除、删除键、Home/End 和方向键导航。
    ///
    /// # 参数
    ///
    /// - `key` - 按键标识
    pub fn handle_key_input(&mut self, key: &str) {
        match key {
            "Backspace" => {
                if self.has_selection() {
                    self.delete_selection();
                }
                else if self.cursor_position > 0 {
                    let prev = self.text[..self.cursor_position].char_indices().next_back().map(|(i, _)| i).unwrap_or(0);
                    self.text.drain(prev..self.cursor_position);
                    self.cursor_position = prev;
                    self.trigger_on_text_change();
                }
            }
            "Delete" => {
                if self.has_selection() {
                    self.delete_selection();
                }
                else {
                    let next = self.text[self.cursor_position..]
                        .char_indices()
                        .nth(1)
                        .map(|(i, _)| self.cursor_position + i)
                        .unwrap_or(self.text.len());
                    if next > self.cursor_position {
                        self.text.drain(self.cursor_position..next);
                        self.trigger_on_text_change();
                    }
                }
            }
            "Home" => {
                self.cursor_position = 0;
                self.clear_selection();
            }
            "End" => {
                self.cursor_position = self.text.len();
                self.clear_selection();
            }
            "Left" => {
                if self.cursor_position > 0 {
                    let prev = self.text[..self.cursor_position].char_indices().next_back().map(|(i, _)| i).unwrap_or(0);
                    self.cursor_position = prev;
                    self.clear_selection();
                }
            }
            "Right" => {
                let next = self.text[self.cursor_position..]
                    .char_indices()
                    .nth(1)
                    .map(|(i, _)| self.cursor_position + i)
                    .unwrap_or(self.text.len());
                self.cursor_position = next;
                self.clear_selection();
            }
            "Shift+Left" => {
                if self.cursor_position > 0 {
                    if self.selection_start.is_none() {
                        self.selection_start = Some(self.cursor_position);
                    }
                    let prev = self.text[..self.cursor_position].char_indices().next_back().map(|(i, _)| i).unwrap_or(0);
                    self.cursor_position = prev;
                }
            }
            "Shift+Right" => {
                if self.selection_start.is_none() {
                    self.selection_start = Some(self.cursor_position);
                }
                let next = self.text[self.cursor_position..]
                    .char_indices()
                    .nth(1)
                    .map(|(i, _)| self.cursor_position + i)
                    .unwrap_or(self.text.len());
                self.cursor_position = next;
            }
            _ => {
                if key.starts_with("Shift+") {
                    return;
                }
                if key.chars().count() == 1 {
                    if self.has_selection() {
                        self.delete_selection();
                    }
                    self.insert_text(key);
                }
            }
        }
    }

    /// 在光标位置插入文本
    ///
    /// # 参数
    ///
    /// - `text` - 要插入的文本
    pub fn insert_text(&mut self, text: &str) {
        self.text.insert_str(self.cursor_position, text);
        self.cursor_position += text.len();
        self.trigger_on_text_change();
    }

    /// 删除选中的文本
    pub fn delete_selection(&mut self) {
        if let Some(start) = self.selection_start {
            let sel_start = start.min(self.cursor_position);
            let sel_end = start.max(self.cursor_position);
            if sel_start < sel_end {
                self.text.drain(sel_start..sel_end);
                self.cursor_position = sel_start;
                self.selection_start = None;
                self.trigger_on_text_change();
            }
        }
    }

    /// 选中所有文本
    pub fn select_all(&mut self) {
        self.selection_start = Some(0);
        self.cursor_position = self.text.len();
    }

    /// 清除选区
    pub fn clear_selection(&mut self) {
        self.selection_start = None;
    }

    /// 是否有选中的文本
    pub fn has_selection(&self) -> bool {
        self.selection_start.map(|start| start != self.cursor_position).unwrap_or(false)
    }

    /// 获取选中的文本
    pub fn selected_text(&self) -> Option<&str> {
        self.selection_start.map(|start| {
            let sel_start = start.min(self.cursor_position);
            let sel_end = start.max(self.cursor_position);
            &self.text[sel_start..sel_end]
        })
    }

    /// 触发文本变更回调
    fn trigger_on_text_change(&mut self) {
        if let Some(ref mut cb) = self.on_text_change {
            cb(&self.text);
        }
    }
}

impl Widget for TextBox {
    fn build(&mut self, tree: &mut UiTree) -> GResult<UiNodeId> {
        let width_value = if let Some(max_w) = self.max_width { SizeValue::Px(max_w) } else { SizeValue::Auto };

        let style = Style { layout: LayoutStyle { width: width_value, ..self.style.layout.clone() }, ..self.style.clone() };

        let display_text = if self.is_focused && self.cursor_visible {
            let mut display = self.text.clone();
            display.insert_str(self.cursor_position, "|");
            display
        }
        else {
            self.text.clone()
        };

        let id = tree.create_node(
            format!("TextBox({})", &self.text[..self.text.len().min(16)]),
            style,
            UiNodeData::Text { content: display_text },
        );

        self.node_id = Some(id);
        Ok(id)
    }

    fn update(&self, tree: &mut UiTree) {
        if let Some(id) = self.node_id {
            let display_text = if self.is_focused && self.cursor_visible {
                let mut display = self.text.clone();
                display.insert_str(self.cursor_position, "|");
                display
            }
            else {
                self.text.clone()
            };
            if let Some(node) = tree.get_mut(id) {
                node.data = UiNodeData::Text { content: display_text };
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
            GuiEvent::KeyPress { key, .. } => {
                let key_str = match key {
                    Key::Backspace => "backspace",
                    Key::Enter => "enter",
                    Key::Escape => "escape",
                    Key::Tab => "tab",
                    Key::Space => " ",
                    Key::A => "a",
                    Key::B => "b",
                    Key::C => "c",
                    Key::D => "d",
                    Key::E => "e",
                    Key::F => "f",
                    Key::G => "g",
                    Key::H => "h",
                    Key::I => "i",
                    Key::J => "j",
                    Key::K => "k",
                    Key::L => "l",
                    Key::M => "m",
                    Key::N => "n",
                    Key::O => "o",
                    Key::P => "p",
                    Key::Q => "q",
                    Key::R => "r",
                    Key::S => "s",
                    Key::T => "t",
                    Key::U => "u",
                    Key::V => "v",
                    Key::W => "w",
                    Key::X => "x",
                    Key::Y => "y",
                    Key::Z => "z",
                    Key::Number(n) => return self.handle_key_input(&n.to_string()),
                    Key::FKey(n) => return self.handle_key_input(&format!("f{}", n)),
                    Key::Other(s) => return self.handle_key_input(s),
                };
                self.handle_key_input(key_str);
            }
            GuiEvent::TextInput { text } => {
                self.insert_text(text);
            }
            GuiEvent::MouseClick { button: MouseButton::Left, .. } => {
                self.is_focused = true;
            }
            _ => {}
        }
    }
}
