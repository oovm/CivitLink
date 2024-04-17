use std::collections::HashMap;

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
    /// 左外边距
    pub margin_left: f32,
    /// 上外边距
    pub margin_top: f32,
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
            margin_left: 0.0,
            margin_top: 0.0,
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

    /// 设置左外边距
    pub fn with_margin_left(mut self, margin_left: f32) -> Self {
        self.margin_left = margin_left;
        self
    }

    /// 设置上外边距
    pub fn with_margin_top(mut self, margin_top: f32) -> Self {
        self.margin_top = margin_top;
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

/// 溢出处理方式
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Overflow {
    /// 可见，子节点超出父节点区域时正常显示
    Visible,
    /// 裁剪，子节点超出父节点区域时被裁剪
    Clip,
}

impl Default for Overflow {
    fn default() -> Self {
        Self::Visible
    }
}

/// 样式值，支持字面值和主题令牌引用
#[derive(Debug, Clone, PartialEq)]
pub enum StyleValue {
    /// 字面值
    Literal(String),
    /// 主题令牌引用（如 $theme-primary）
    ThemeRef(String),
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
    /// 溢出处理方式
    pub overflow: Overflow,
    /// 图片纹理路径
    pub image_path: Option<String>,
    /// 不透明度（0.0 完全透明，1.0 完全不透明）
    pub opacity: f32,
    /// 主题令牌覆盖
    pub theme_token_overrides: HashMap<String, StyleValue>,
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
            overflow: Overflow::default(),
            image_path: None,
            opacity: 1.0,
            theme_token_overrides: HashMap::new(),
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

    /// 设置溢出处理方式
    pub fn with_overflow(mut self, overflow: Overflow) -> Self {
        self.overflow = overflow;
        self
    }

    /// 设置图片纹理路径
    pub fn with_image_path(mut self, path: impl Into<String>) -> Self {
        self.image_path = Some(path.into());
        self
    }

    /// 设置不透明度
    pub fn with_opacity(mut self, opacity: f32) -> Self {
        self.opacity = opacity;
        self
    }

    /// 添加主题令牌覆盖
    pub fn with_theme_token(mut self, name: impl Into<String>, value: StyleValue) -> Self {
        self.theme_token_overrides.insert(name.into(), value);
        self
    }
}

/// 可继承的样式属性
///
/// 定义哪些样式属性可以从父节点继承到子节点。
/// 当子节点未显式设置这些属性时，自动使用父节点的值。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum InheritableProperty {
    /// 字体大小
    FontSize,
    /// 字体颜色
    FontColor,
    /// 行间距
    LineHeight,
    /// 不透明度
    Opacity,
    /// 溢出处理方式
    Overflow,
}

/// 样式优先级
///
/// 定义样式来源的优先级顺序，数值越大优先级越高。
/// 当同一属性在多个来源中定义时，使用优先级最高的来源的值。
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum StylePriority {
    /// 主题默认值（最低优先级）
    ThemeDefault = 0,
    /// 继承自父节点的样式
    Inherited = 1,
    /// 类样式（CSS class）
    ClassStyle = 2,
    /// 内联样式（最高优先级）
    Inline = 3,
}

impl Default for StylePriority {
    fn default() -> Self {
        Self::ThemeDefault
    }
}

/// 样式解析器
///
/// 将主题默认值、继承样式、类样式和内联样式按优先级合并，
/// 计算出节点的最终样式。
pub struct StyleResolver {
    /// 可继承属性列表
    inheritable_properties: Vec<InheritableProperty>,
}

impl StyleResolver {
    /// 创建新的样式解析器
    pub fn new() -> Self {
        Self {
            inheritable_properties: vec![
                InheritableProperty::FontSize,
                InheritableProperty::FontColor,
                InheritableProperty::LineHeight,
                InheritableProperty::Opacity,
                InheritableProperty::Overflow,
            ],
        }
    }

    /// 判断指定属性是否可继承
    pub fn is_inheritable(&self, prop: InheritableProperty) -> bool {
        self.inheritable_properties.contains(&prop)
    }

    /// 从父节点样式继承可继承属性到子节点
    ///
    /// 对于子节点中未显式设置的可继承属性，自动使用父节点的值。
    /// 已显式设置的属性不会被覆盖。
    pub fn inherit(parent_style: &Style, child_style: &mut Style) {
        if child_style.font.is_none() && parent_style.font.is_some() {
            child_style.font = parent_style.font.clone();
        }
        else if let (Some(ref parent_font), Some(ref mut child_font)) = (&parent_style.font, &mut child_style.font) {
            if child_font.size == FontStyle::default().size {
                child_font.size = parent_font.size;
            }
            if child_font.color == FontStyle::default().color {
                child_font.color = parent_font.color;
            }
            if child_font.line_height == FontStyle::default().line_height {
                child_font.line_height = parent_font.line_height;
            }
        }
    }

    /// 解析 StyleValue::ThemeRef 引用
    ///
    /// 从主题注册表中查找令牌名称对应的实际值。
    /// 若令牌不存在，返回 None。
    pub fn resolve_theme_ref(style: &mut Style, resolve_fn: &dyn Fn(&str) -> Option<String>) {
        let mut resolved = HashMap::new();
        for (key, value) in &style.theme_token_overrides {
            if let StyleValue::ThemeRef(token_name) = value {
                if let Some(resolved_value) = resolve_fn(token_name) {
                    resolved.insert(key.clone(), StyleValue::Literal(resolved_value));
                }
            }
        }
        for (key, value) in resolved {
            style.theme_token_overrides.insert(key, value);
        }
    }

    /// 计算节点的最终样式
    ///
    /// 按优先级合并主题默认值、继承样式、类样式和内联样式：
    /// 1. 从主题默认值开始
    /// 2. 应用继承的样式
    /// 3. 应用类样式
    /// 4. 应用内联样式
    pub fn resolve(
        theme_default: &Style,
        inherited: Option<&Style>,
        class_style: Option<&Style>,
        inline_style: &Style,
    ) -> Style {
        let mut result = theme_default.clone();

        if let Some(inherited_style) = inherited {
            Self::inherit(inherited_style, &mut result);
        }

        if let Some(class_style) = class_style {
            Self::apply_higher_priority(&class_style, &mut result);
        }

        Self::apply_higher_priority(inline_style, &mut result);

        result
    }

    /// 将高优先级样式应用到低优先级样式上
    ///
    /// 仅覆盖低优先级样式中未设置（None/默认值）的属性。
    fn apply_higher_priority(higher: &Style, lower: &mut Style) {
        if higher.background_color.is_some() {
            lower.background_color = higher.background_color;
        }
        if higher.border_color.is_some() {
            lower.border_color = higher.border_color;
        }
        if higher.border_width != 0.0 {
            lower.border_width = higher.border_width;
        }
        if higher.corner_radius != 0.0 {
            lower.corner_radius = higher.corner_radius;
        }
        if higher.font.is_some() {
            lower.font = higher.font.clone();
        }
        if higher.image_path.is_some() {
            lower.image_path = higher.image_path.clone();
        }
        if higher.opacity != 1.0 {
            lower.opacity = higher.opacity;
        }
        for (key, value) in &higher.theme_token_overrides {
            lower.theme_token_overrides.insert(key.clone(), value.clone());
        }
    }
}

impl Default for StyleResolver {
    fn default() -> Self {
        Self::new()
    }
}
