use std::collections::HashMap;

/// 字体粗细
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FontWeight {
    /// 100
    Thin,
    /// 200
    ExtraLight,
    /// 300
    Light,
    /// 400
    Normal,
    /// 500
    Medium,
    /// 600
    SemiBold,
    /// 700
    Bold,
    /// 800
    ExtraBold,
    /// 900
    Black,
}

impl Default for FontWeight {
    fn default() -> Self {
        Self::Normal
    }
}

/// 字体样式
///
/// 描述字体面的样式，与 [`crate::style::FontStyle`] 不同，
/// 后者描述的是渲染文本的样式（大小、颜色等）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FontStyle {
    /// 正常
    Normal,
    /// 斜体
    Italic,
}

impl Default for FontStyle {
    fn default() -> Self {
        Self::Normal
    }
}

/// 字体面描述
#[derive(Debug, Clone, PartialEq)]
pub struct FontFace {
    /// 字体族名称
    pub family_name: String,
    /// 字体粗细
    pub weight: FontWeight,
    /// 字体样式
    pub style: FontStyle,
    /// 字形数量
    pub glyph_count: u32,
}

impl FontFace {
    /// 创建字体面描述
    pub fn new(family_name: impl Into<String>) -> Self {
        Self { family_name: family_name.into(), weight: FontWeight::default(), style: FontStyle::default(), glyph_count: 0 }
    }

    /// 设置字体粗细
    pub fn with_weight(mut self, weight: FontWeight) -> Self {
        self.weight = weight;
        self
    }

    /// 设置字体样式
    pub fn with_style(mut self, style: FontStyle) -> Self {
        self.style = style;
        self
    }

    /// 设置字形数量
    pub fn with_glyph_count(mut self, count: u32) -> Self {
        self.glyph_count = count;
        self
    }
}

/// 字形度量
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GlyphInfo {
    /// 字形 ID
    pub glyph_id: u32,
    /// 水平前进宽度
    pub advance_width: f32,
    /// 垂直前进高度
    pub advance_height: f32,
    /// 水平轴承偏移
    pub bearing_x: f32,
    /// 垂直轴承偏移
    pub bearing_y: f32,
    /// 字形宽度
    pub width: f32,
    /// 字形高度
    pub height: f32,
}

impl GlyphInfo {
    /// 创建默认字形度量
    pub fn new(glyph_id: u32) -> Self {
        Self { glyph_id, advance_width: 0.0, advance_height: 0.0, bearing_x: 0.0, bearing_y: 0.0, width: 0.0, height: 0.0 }
    }
}

/// 字形位图数据
#[derive(Debug, Clone)]
pub struct GlyphBitmap {
    /// 位图宽度
    pub width: u32,
    /// 位图高度
    pub height: u32,
    /// 位图数据（单通道 alpha）
    pub data: Vec<u8>,
}

impl GlyphBitmap {
    /// 创建字形位图
    pub fn new(width: u32, height: u32, data: Vec<u8>) -> Self {
        Self { width, height, data }
    }
}

/// 字形在图集中的 UV 坐标
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GlyphUv {
    /// 左 U 坐标
    pub u0: f32,
    /// 上 V 坐标
    pub v0: f32,
    /// 右 U 坐标
    pub u1: f32,
    /// 下 V 坐标
    pub v1: f32,
}

/// 字形图集管理
#[derive(Debug, Clone)]
pub struct FontAtlas {
    /// 图集宽度
    pub width: u32,
    /// 图集高度
    pub height: u32,
    /// 字形 UV 映射
    pub glyph_uvs: HashMap<u32, GlyphUv>,
    /// 当前写入 X 位置
    pub cursor_x: u32,
    /// 当前写入 Y 位置
    pub cursor_y: u32,
    /// 当前行高度
    pub row_height: u32,
}

impl FontAtlas {
    /// 创建指定尺寸的图集
    pub fn new(width: u32, height: u32) -> Self {
        Self { width, height, glyph_uvs: HashMap::new(), cursor_x: 0, cursor_y: 0, row_height: 0 }
    }

    /// 添加字形到图集，返回 UV 坐标
    ///
    /// 如果空间不足返回 `None`。简单实现：水平排列，超出宽度换行。
    pub fn add_glyph(&mut self, glyph_id: u32, bitmap: &GlyphBitmap) -> Option<GlyphUv> {
        if bitmap.width > self.width || bitmap.height > self.height {
            return None;
        }

        if self.cursor_x + bitmap.width > self.width {
            self.cursor_x = 0;
            self.cursor_y += self.row_height;
            self.row_height = 0;
        }

        if self.cursor_y + bitmap.height > self.height {
            return None;
        }

        let uv = GlyphUv {
            u0: self.cursor_x as f32 / self.width as f32,
            v0: self.cursor_y as f32 / self.height as f32,
            u1: (self.cursor_x + bitmap.width) as f32 / self.width as f32,
            v1: (self.cursor_y + bitmap.height) as f32 / self.height as f32,
        };

        self.cursor_x += bitmap.width;
        if bitmap.height > self.row_height {
            self.row_height = bitmap.height;
        }

        self.glyph_uvs.insert(glyph_id, uv);
        Some(uv)
    }

    /// 获取字形 UV 坐标
    pub fn get_glyph_uv(&self, glyph_id: u32) -> Option<GlyphUv> {
        self.glyph_uvs.get(&glyph_id).copied()
    }

    /// 获取图集尺寸
    pub fn atlas_size(&self) -> (u32, u32) {
        (self.width, self.height)
    }

    /// 清空图集
    pub fn clear(&mut self) {
        self.glyph_uvs.clear();
        self.cursor_x = 0;
        self.cursor_y = 0;
        self.row_height = 0;
    }
}

/// 整形后的字形
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ShapedGlyph {
    /// 字形 ID
    pub glyph_id: u32,
    /// X 偏移
    pub x: f32,
    /// Y 偏移
    pub y: f32,
    /// 前进宽度
    pub advance: f32,
}

/// 整形后的文本
#[derive(Debug, Clone, Default)]
pub struct ShapedText {
    /// 字形列表
    pub glyphs: Vec<ShapedGlyph>,
    /// 文本总宽度
    pub width: f32,
    /// 文本总高度
    pub height: f32,
}

impl ShapedText {
    /// 创建整形后的文本
    pub fn new() -> Self {
        Self::default()
    }
}

/// 文本整形接口
pub trait TextShaper {
    /// 整形文本
    fn shape_text(&mut self, text: &str, font_size: f32, max_width: Option<f32>) -> ShapedText;

    /// 测量文本尺寸，返回 (width, height)
    fn measure_text(&self, text: &str, font_size: f32, max_width: Option<f32>) -> (f32, f32);
}
