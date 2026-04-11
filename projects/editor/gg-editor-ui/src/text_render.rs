use std::{
    collections::HashMap,
    hash::{Hash, Hasher},
};

use gg_render::Color;
use gg_ui::{
    DpiScale,
    font::{FontAtlas, GlyphBitmap, GlyphUv, ShapedText, TextShaper},
};

/// 文本渲染顶点
#[derive(Debug, Clone, Copy)]
pub struct TextVertex {
    /// 顶点位置
    pub position: [f32; 2],
    /// 纹理坐标
    pub uv: [f32; 2],
    /// 顶点颜色（RGBA）
    pub color: [f32; 4],
}

impl TextVertex {
    /// 创建文本渲染顶点
    pub fn new(position: [f32; 2], uv: [f32; 2], color: [f32; 4]) -> Self {
        Self { position, uv, color }
    }
}

/// 文本缓存键
#[derive(Debug, Clone)]
pub struct TextCacheKey {
    /// 文本内容
    pub text: String,
    /// 字体大小
    pub font_size: f32,
    /// 最大宽度
    pub max_width: Option<f32>,
}

impl PartialEq for TextCacheKey {
    fn eq(&self, other: &Self) -> bool {
        self.text == other.text
            && self.font_size.to_bits() == other.font_size.to_bits()
            && self.max_width.map(|v| v.to_bits()) == other.max_width.map(|v| v.to_bits())
    }
}

impl Eq for TextCacheKey {}

impl Hash for TextCacheKey {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.text.hash(state);
        self.font_size.to_bits().hash(state);
        self.max_width.map(|v| v.to_bits()).hash(state);
    }
}

impl TextCacheKey {
    /// 创建文本缓存键
    pub fn new(text: impl Into<String>, font_size: f32, max_width: Option<f32>) -> Self {
        Self { text: text.into(), font_size, max_width }
    }
}

/// 文本布局缓存
#[derive(Debug, Default)]
pub struct TextCache {
    /// 缓存映射
    cache: HashMap<TextCacheKey, ShapedText>,
}

impl TextCache {
    /// 创建空缓存
    pub fn new() -> Self {
        Self::default()
    }

    /// 缓存获取或计算
    ///
    /// 如果缓存命中直接返回，否则调用 shaper.shape_text 计算并存入缓存
    pub fn get_or_shape(&mut self, key: TextCacheKey, shaper: &mut dyn TextShaper) -> &ShapedText {
        if !self.cache.contains_key(&key) {
            let shaped = shaper.shape_text(&key.text, key.font_size, key.max_width);
            self.cache.insert(key.clone(), shaped);
        }
        self.cache.get(&key).unwrap()
    }

    /// 使缓存失效
    pub fn invalidate(&mut self, key: &TextCacheKey) {
        self.cache.remove(key);
    }

    /// 清空缓存
    pub fn clear(&mut self) {
        self.cache.clear();
    }
}

/// 文本渲染引擎
pub struct TextRenderEngine {
    /// DPI 缩放因子
    dpi_scale: DpiScale,
    /// 字形图集
    atlas: FontAtlas,
    /// 文本缓存
    cache: TextCache,
    /// 文本整形器
    shaper: Option<Box<dyn TextShaper + Send + Sync>>,
}

impl TextRenderEngine {
    /// 创建文本渲染引擎，使用 2048x2048 图集
    pub fn new(dpi_scale: DpiScale) -> Self {
        Self { dpi_scale, atlas: FontAtlas::new(2048, 2048), cache: TextCache::new(), shaper: None }
    }

    /// 使用指定整形器创建文本渲染引擎
    pub fn with_shaper(dpi_scale: DpiScale, shaper: Box<dyn TextShaper + Send + Sync>) -> Self {
        Self { dpi_scale, atlas: FontAtlas::new(2048, 2048), cache: TextCache::new(), shaper: Some(shaper) }
    }

    /// 设置文本整形器
    pub fn set_shaper(&mut self, shaper: Box<dyn TextShaper + Send + Sync>) {
        self.shaper = Some(shaper);
    }

    /// 文本整形
    ///
    /// 如果有 shaper 则使用缓存，否则返回空的 ShapedText
    pub fn shape_text(&mut self, text: &str, font_size: f32, max_width: Option<f32>) -> ShapedText {
        match &mut self.shaper {
            Some(shaper) => {
                let key = TextCacheKey::new(text, font_size, max_width);
                self.cache.get_or_shape(key, shaper.as_mut()).clone()
            }
            None => ShapedText::new(),
        }
    }

    /// 生成字形顶点
    ///
    /// 遍历 shaped.glyphs，为每个字形生成 4 个顶点的矩形（使用图集 UV 坐标），
    /// 每个字形 6 个索引（2 个三角形）
    pub fn generate_vertices(&self, shaped: &ShapedText, position: (f32, f32), color: Color) -> Vec<TextVertex> {
        let color_arr = [color.r, color.g, color.b, color.a];
        let mut vertices = Vec::new();

        for glyph in &shaped.glyphs {
            let uv = match self.atlas.get_glyph_uv(glyph.glyph_id) {
                Some(uv) => uv,
                None => continue,
            };

            let x = position.0 + glyph.x;
            let y = position.1 + glyph.y;
            let w = glyph.advance;
            let h = shaped.height;

            let v0 = TextVertex::new([x, y], [uv.u0, uv.v0], color_arr);
            let v1 = TextVertex::new([x + w, y], [uv.u1, uv.v0], color_arr);
            let v2 = TextVertex::new([x + w, y + h], [uv.u1, uv.v1], color_arr);
            let v3 = TextVertex::new([x, y + h], [uv.u0, uv.v1], color_arr);

            vertices.push(v0);
            vertices.push(v1);
            vertices.push(v2);
            vertices.push(v3);
        }

        vertices
    }

    /// 更新图集纹理
    pub fn update_atlas(&mut self, glyph_id: u32, bitmap: &GlyphBitmap) -> Option<GlyphUv> {
        self.atlas.add_glyph(glyph_id, bitmap)
    }

    /// 设置 DPI 缩放因子并清空缓存
    pub fn set_dpi_scale(&mut self, scale: DpiScale) {
        self.dpi_scale = scale;
        self.cache.clear();
    }

    /// 获取图集引用
    pub fn atlas(&self) -> &FontAtlas {
        &self.atlas
    }

    /// 获取缓存引用
    pub fn cache(&self) -> &TextCache {
        &self.cache
    }
}
