use crate::{
    node::{UiNodeData, UiNodeId, UiTree},
    style::{FontStyle, LayoutStyle, SizeValue, Style},
    widget::Widget,
};

/// 文本框控件
///
/// 提供文本显示和输入功能的 UI 元素，支持自动换行和最大宽度限制。
pub struct TextBox {
    /// 文本内容
    pub text: String,
    /// 样式
    pub style: Style,
    /// 最大宽度（None 表示不限制）
    pub max_width: Option<f32>,
    /// 是否自动换行
    pub word_wrap: bool,
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

        Self { text: text.into(), style, max_width: None, word_wrap: true, node_id: None }
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
}

impl Widget for TextBox {
    fn build(&self, tree: &mut UiTree) -> UiNodeId {
        let width_value = if let Some(max_w) = self.max_width { SizeValue::Px(max_w) } else { SizeValue::Auto };

        let style = Style { layout: LayoutStyle { width: width_value, ..self.style.layout.clone() }, ..self.style.clone() };

        let id = tree.create_node(
            format!("TextBox({})", &self.text[..self.text.len().min(16)]),
            style,
            UiNodeData::Text { content: self.text.clone() },
        );

        id
    }

    fn update(&self, tree: &mut UiTree) {
        if let Some(id) = self.node_id {
            if let Some(node) = tree.get_mut(id) {
                node.data = UiNodeData::Text { content: self.text.clone() };
            }
        }
    }

    fn node_id(&self) -> Option<UiNodeId> {
        self.node_id
    }
}
