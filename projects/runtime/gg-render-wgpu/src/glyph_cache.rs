use ab_glyph::{Font, FontArc, Glyph};
use gg_core::{GError, GErrorKind, GResult};
use gg_render::TextureId;

use crate::{
    glyph_atlas::{AtlasGlyphPlacement, GlyphAtlas},
    texture_cache::TextureCache,
};

/// 字形信息
///
/// 描述一个已光栅化字形在纹理图集中的位置和属性。
#[derive(Debug, Clone, Copy)]
pub struct GlyphInfo {
    /// 字形所在图集页的纹理标识符
    pub texture_id: TextureId,
    /// 字形在图集中的 UV 矩形 `[u_min, v_min, u_max, v_max]`
    pub uv_rect: [f32; 4],
    /// 字形尺寸 `[width, height]`
    pub size: [f32; 2],
    /// 字形偏移 `[x_offset, y_offset]`
    pub offset: [f32; 2],
}

impl From<AtlasGlyphPlacement> for GlyphInfo {
    fn from(p: AtlasGlyphPlacement) -> Self {
        GlyphInfo { texture_id: p.texture_id, uv_rect: p.uv_rect, size: p.size, offset: p.offset }
    }
}

/// 字体回退链
///
/// 维护一个有序的字体列表，当主字体不包含所需字形时，
/// 按顺序查找备选字体，实现多字体回退机制。
/// 典型用法是将主字体放在首位，CJK 字体作为回退。
pub struct FontFallbackChain {
    /// 有序字体列表，索引 0 为主字体
    fonts: Vec<FontArc>,
}

impl FontFallbackChain {
    /// 创建空的字体回退链
    pub fn new() -> Self {
        Self { fonts: Vec::new() }
    }

    /// 添加字体到回退链末尾
    ///
    /// 字体将作为最低优先级的回退字体。
    ///
    /// # 参数
    ///
    /// - `font` - 要添加的字体
    pub fn push(&mut self, font: FontArc) {
        self.fonts.push(font);
    }

    /// 在指定位置插入字体
    ///
    /// 索引 0 为主字体，后续为回退字体。
    ///
    /// # 参数
    ///
    /// - `index` - 插入位置
    /// - `font` - 要插入的字体
    pub fn insert(&mut self, index: usize, font: FontArc) {
        self.fonts.insert(index, font);
    }

    /// 替换主字体（索引 0）
    ///
    /// 如果回退链为空则添加为主字体。
    ///
    /// # 参数
    ///
    /// - `font` - 新的主字体
    pub fn set_primary(&mut self, font: FontArc) {
        if self.fonts.is_empty() {
            self.fonts.push(font);
        }
        else {
            self.fonts[0] = font;
        }
    }

    /// 查找包含指定码位的字体
    ///
    /// 按回退链顺序依次查找，返回第一个包含该码位的字体索引。
    ///
    /// # 参数
    ///
    /// - `codepoint` - Unicode 码位
    ///
    /// # 返回值
    ///
    /// 找到则返回字体索引，否则返回 `None`
    pub fn find_font_for_codepoint(&self, codepoint: u32) -> Option<usize> {
        let c = char::from_u32(codepoint)?;
        self.fonts.iter().position(|font| {
            let glyph_id = font.glyph_id(c);
            glyph_id.0 != 0
        })
    }

    /// 获取指定索引的字体引用
    ///
    /// # 参数
    ///
    /// - `index` - 字体索引
    pub fn get(&self, index: usize) -> Option<&FontArc> {
        self.fonts.get(index)
    }

    /// 获取主字体引用
    pub fn primary(&self) -> Option<&FontArc> {
        self.fonts.first()
    }

    /// 获取字体数量
    pub fn len(&self) -> usize {
        self.fonts.len()
    }

    /// 检查回退链是否为空
    pub fn is_empty(&self) -> bool {
        self.fonts.is_empty()
    }
}

impl Default for FontFallbackChain {
    fn default() -> Self {
        Self::new()
    }
}

/// 字形缓存
///
/// 使用 `ab_glyph` 库光栅化字形，并将结果缓存到纹理图集中。
/// 支持多字体回退链，当主字体不包含所需字形时自动查找备选字体。
///
/// 创建后会自动加载内置的 Noto Sans 字体作为主字体。
pub struct GlyphCache {
    /// 字体回退链
    fallback_chain: FontFallbackChain,
    /// 字形纹理图集
    atlas: GlyphAtlas,
}

/// 内置默认字体数据 (Noto Sans)
const NOTO_SANS_FONT_DATA: &[u8] = include_bytes!("../../../../assets/fonts/NotoSans-Regular.ttf");

impl GlyphCache {
    /// 创建新的字形缓存
    ///
    /// 自动加载内置的 Noto Sans 字体作为主字体。
    pub fn new() -> Self {
        let mut cache = Self { fallback_chain: FontFallbackChain::new(), atlas: GlyphAtlas::new() };
        cache.load_font(NOTO_SANS_FONT_DATA.to_vec());
        cache
    }

    /// 从字节数据加载字体作为主字体
    ///
    /// 加载成功后会清空已有的字形缓存。
    /// 原有的回退字体保持不变。
    pub fn load_font(&mut self, data: Vec<u8>) {
        if let Ok(font) = FontArc::try_from_vec(data) {
            self.fallback_chain.set_primary(font);
            self.atlas.clear();
        }
    }

    /// 添加回退字体
    ///
    /// 将字体添加到回调链末尾，作为最低优先级的回退字体。
    ///
    /// # 参数
    ///
    /// - `data` - 字体文件字节数据
    ///
    /// # 返回值
    ///
    /// 成功返回 `true`，字体数据无效返回 `false`
    pub fn add_fallback_font(&mut self, data: Vec<u8>) -> bool {
        if let Ok(font) = FontArc::try_from_vec(data) {
            self.fallback_chain.push(font);
            true
        }
        else {
            false
        }
    }

    /// 在指定位置插入回退字体
    ///
    /// 索引 0 为主字体，后续为回退字体。
    /// 插入后原有字体的优先级会后移。
    ///
    /// # 参数
    ///
    /// - `index` - 插入位置（0 为主字体位置）
    /// - `data` - 字体文件字节数据
    ///
    /// # 返回值
    ///
    /// 成功返回 `true`，字体数据无效返回 `false`
    pub fn insert_fallback_font(&mut self, index: usize, data: Vec<u8>) -> bool {
        if let Ok(font) = FontArc::try_from_vec(data) {
            self.fallback_chain.insert(index, font);
            true
        }
        else {
            false
        }
    }

    /// 获取或光栅化字形
    ///
    /// 如果字形已缓存则直接返回缓存信息，
    /// 否则使用 `ab_glyph` 光栅化字形并放入纹理图集。
    /// 当主字体不包含该字形时，自动按回退链查找备选字体。
    pub fn get_or_rasterize(
        &mut self,
        glyph: Glyph,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        texture_cache: &mut TextureCache,
    ) -> GResult<GlyphInfo> {
        let codepoint = glyph.id.0 as u32;
        let font_size = glyph.scale.y as u32;

        let font_index = self.fallback_chain.find_font_for_codepoint(codepoint).unwrap_or(0);

        let font = self
            .fallback_chain
            .get(font_index)
            .ok_or_else(|| GError {
                kind: GErrorKind::Runtime, message: "未加载字体，无法光栅化字形".to_string()
            })?;

        let scaled_glyph = font.outline_glyph(glyph);

        if let Some(outlined) = scaled_glyph {
            let bounds = outlined.px_bounds();
            let width = bounds.width().ceil() as u32;
            let height = bounds.height().ceil() as u32;

            if width == 0 || height == 0 {
                return Ok(GlyphInfo {
                    texture_id: TextureId::INVALID,
                    uv_rect: [0.0, 0.0, 0.0, 0.0],
                    size: [0.0, 0.0],
                    offset: [bounds.min.x, bounds.min.y],
                });
            }

            let mut rgba_data = vec![0u8; (width * height * 4) as usize];
            outlined.draw(|x, y, v| {
                let idx = ((y * width + x) * 4) as usize;
                if (idx + 3) < rgba_data.len() {
                    rgba_data[idx] = 255;
                    rgba_data[idx + 1] = 255;
                    rgba_data[idx + 2] = 255;
                    rgba_data[idx + 3] = (v * 255.0) as u8;
                }
            });

            let placement = self.atlas.get_or_allocate(
                codepoint,
                font_size,
                width,
                height,
                bounds.min.x,
                bounds.min.y,
                &rgba_data,
                device,
                queue,
                texture_cache,
            )?;

            Ok(placement.into())
        }
        else {
            self.rasterize_tofu(codepoint, font_size, device, queue, texture_cache)
        }
    }

    /// 渲染占位符（豆腐块）
    ///
    /// 当所有字体均不包含所需字形时，渲染一个矩形占位符。
    fn rasterize_tofu(
        &mut self,
        codepoint: u32,
        font_size: u32,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        texture_cache: &mut TextureCache,
    ) -> GResult<GlyphInfo> {
        let width = (font_size / 2).max(4);
        let height = font_size.max(4);

        let mut rgba_data = vec![0u8; (width * height * 4) as usize];
        for y in 0..height {
            for x in 0..width {
                let idx = ((y * width + x) * 4) as usize;
                let is_border = x == 0 || x == width - 1 || y == 0 || y == height - 1;
                if is_border {
                    rgba_data[idx] = 128;
                    rgba_data[idx + 1] = 128;
                    rgba_data[idx + 2] = 128;
                    rgba_data[idx + 3] = 255;
                }
            }
        }

        let placement = self.atlas.get_or_allocate(
            codepoint,
            font_size,
            width,
            height,
            0.0,
            0.0,
            &rgba_data,
            device,
            queue,
            texture_cache,
        )?;

        Ok(placement.into())
    }

    /// 获取当前主字体的引用
    pub fn font(&self) -> Option<&FontArc> {
        self.fallback_chain.primary()
    }

    /// 获取字体回退链的引用
    pub fn fallback_chain(&self) -> &FontFallbackChain {
        &self.fallback_chain
    }

    /// 检查是否已加载字体
    pub fn has_font(&self) -> bool {
        !self.fallback_chain.is_empty()
    }

    /// 清空字形缓存
    pub fn clear(&mut self) {
        self.atlas.clear();
    }
}

impl Default for GlyphCache {
    fn default() -> Self {
        Self::new()
    }
}
