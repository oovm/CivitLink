use gg_render::Color;

/// 主轴方向
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FlexDirection {
    /// 从左到右
    Row,
    /// 从上到下
    Column,
}

impl Default for FlexDirection {
    fn default() -> Self {
        Self::Row
    }
}

/// 主轴对齐
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FlexAlign {
    /// 起点
    Start,
    /// 居中
    Center,
    /// 终点
    End,
    /// 两端对齐
    SpaceBetween,
}

impl Default for FlexAlign {
    fn default() -> Self {
        Self::Start
    }
}

/// 尺寸值
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SizeValue {
    /// 自动（由内容决定）
    Auto,
    /// 固定像素值
    Px(f32),
    /// 百分比（0.0-1.0）
    Percent(f32),
}

impl Default for SizeValue {
    fn default() -> Self {
        Self::Auto
    }
}

/// 布局样式
#[derive(Debug, Clone, PartialEq)]
pub struct LayoutStyle {
    /// 主轴方向
    pub direction: FlexDirection,
    /// 主轴对齐
    pub justify_content: FlexAlign,
    /// 交叉轴对齐
    pub align_items: FlexAlign,
    /// 是否换行
    pub wrap: bool,
    /// 间距
    pub gap: f32,
    /// 内边距
    pub padding: f32,
    /// 外边距
    pub margin: f32,
    /// 宽度
    pub width: SizeValue,
    /// 高度
    pub height: SizeValue,
    /// 最小宽度
    pub min_width: SizeValue,
    /// 最小高度
    pub min_height: SizeValue,
}

impl Default for LayoutStyle {
    fn default() -> Self {
        Self {
            direction: FlexDirection::default(),
            justify_content: FlexAlign::default(),
            align_items: FlexAlign::default(),
            wrap: false,
            gap: 0.0,
            padding: 0.0,
            margin: 0.0,
            width: SizeValue::Auto,
            height: SizeValue::Auto,
            min_width: SizeValue::Auto,
            min_height: SizeValue::Auto,
        }
    }
}

impl LayoutStyle {
    /// 创建默认布局样式
    pub fn new() -> Self {
        Self::default()
    }

    /// 设置主轴方向
    pub fn with_direction(mut self, direction: FlexDirection) -> Self {
        self.direction = direction;
        self
    }

    /// 设置主轴对齐
    pub fn with_justify_content(mut self, align: FlexAlign) -> Self {
        self.justify_content = align;
        self
    }

    /// 设置交叉轴对齐
    pub fn with_align_items(mut self, align: FlexAlign) -> Self {
        self.align_items = align;
        self
    }

    /// 设置是否换行
    pub fn with_wrap(mut self, wrap: bool) -> Self {
        self.wrap = wrap;
        self
    }

    /// 设置间距
    pub fn with_gap(mut self, gap: f32) -> Self {
        self.gap = gap;
        self
    }

    /// 设置内边距
    pub fn with_padding(mut self, padding: f32) -> Self {
        self.padding = padding;
        self
    }

    /// 设置外边距
    pub fn with_margin(mut self, margin: f32) -> Self {
        self.margin = margin;
        self
    }

    /// 设置宽度
    pub fn with_width(mut self, width: SizeValue) -> Self {
        self.width = width;
        self
    }

    /// 设置高度
    pub fn with_height(mut self, height: SizeValue) -> Self {
        self.height = height;
        self
    }

    /// 设置最小宽度
    pub fn with_min_width(mut self, min_width: SizeValue) -> Self {
        self.min_width = min_width;
        self
    }

    /// 设置最小高度
    pub fn with_min_height(mut self, min_height: SizeValue) -> Self {
        self.min_height = min_height;
        self
    }
}

/// 字体样式
#[derive(Debug, Clone, PartialEq)]
pub struct FontStyle {
    /// 字体大小
    pub size: f32,
    /// 字体颜色
    pub color: Color,
    /// 行间距
    pub line_height: f32,
}

impl Default for FontStyle {
    fn default() -> Self {
        Self { size: 16.0, color: Color::WHITE, line_height: 1.2 }
    }
}

impl FontStyle {
    /// 创建默认字体样式
    pub fn new() -> Self {
        Self::default()
    }

    /// 设置字体大小
    pub fn with_size(mut self, size: f32) -> Self {
        self.size = size;
        self
    }

    /// 设置字体颜色
    pub fn with_color(mut self, color: Color) -> Self {
        self.color = color;
        self
    }

    /// 设置行间距
    pub fn with_line_height(mut self, line_height: f32) -> Self {
        self.line_height = line_height;
        self
    }
}

/// UI 样式
#[derive(Debug, Clone, PartialEq)]
pub struct Style {
    /// 布局样式
    pub layout: LayoutStyle,
    /// 背景色
    pub background_color: Option<Color>,
    /// 边框颜色
    pub border_color: Option<Color>,
    /// 边框宽度
    pub border_width: f32,
    /// 圆角半径
    pub corner_radius: f32,
    /// 字体样式
    pub font: Option<FontStyle>,
}

impl Default for Style {
    fn default() -> Self {
        Self {
            layout: LayoutStyle::default(),
            background_color: None,
            border_color: None,
            border_width: 0.0,
            corner_radius: 0.0,
            font: None,
        }
    }
}

impl Style {
    /// 创建默认样式
    pub fn new() -> Self {
        Self::default()
    }

    /// 设置布局样式
    pub fn with_layout(mut self, layout: LayoutStyle) -> Self {
        self.layout = layout;
        self
    }

    /// 设置背景色
    pub fn with_background_color(mut self, color: Color) -> Self {
        self.background_color = Some(color);
        self
    }

    /// 设置边框颜色
    pub fn with_border_color(mut self, color: Color) -> Self {
        self.border_color = Some(color);
        self
    }

    /// 设置边框宽度
    pub fn with_border_width(mut self, width: f32) -> Self {
        self.border_width = width;
        self
    }

    /// 设置圆角半径
    pub fn with_corner_radius(mut self, radius: f32) -> Self {
        self.corner_radius = radius;
        self
    }

    /// 设置字体样式
    pub fn with_font(mut self, font: FontStyle) -> Self {
        self.font = Some(font);
        self
    }
}
