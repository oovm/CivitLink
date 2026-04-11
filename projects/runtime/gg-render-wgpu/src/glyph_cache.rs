//! 字形缓存模块
//! 
//! 使用 `ab_glyph` 库光栅化字形，并将结果缓存到纹理图集中。

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

/// 字形缓存
/// 
/// 使用 `ab_glyph` 库光栅化字形，并将结果缓存到纹理图集中。
/// 支持动态加载字体和按需光栅化字形。
/// 
/// 创建后会自动加载内置的 Noto Sans 字体。
pub struct GlyphCache {
    /// 当前字体
    font: Option<FontArc>,
    /// 字形纹理图集
    atlas: GlyphAtlas,
}

/// 内置默认字体数据 (Noto Sans)
const NOTO_SANS_FONT_DATA: &[u8] = include_bytes!("../../../../assets/fonts/NotoSans-Regular.ttf");

impl GlyphCache {
    /// 创建新的字形缓存
    /// 
    /// 自动加载内置的 Noto Sans 字体。
    pub fn new() -> Self {
        let mut cache = Self { font: None, atlas: GlyphAtlas::new() };
        cache.load_font(NOTO_SANS_FONT_DATA.to_vec());
        cache
    }

    /// 从字节数据加载字体
    /// 
    /// 加载成功后会清空已有的字形缓存。
    pub fn load_font(&mut self, data: Vec<u8>) {
        if let Ok(font) = FontArc::try_from_vec(data) {
            self.font = Some(font);
            self.atlas.clear();
        }
    }

    /// 获取或光栅化字形
    /// 
    /// 如果字形已缓存则直接返回缓存信息，
    /// 否则使用 `ab_glyph` 光栅化字形并放入纹理图集。
    pub fn get_or_rasterize(
        &mut self,
        glyph: Glyph,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        texture_cache: &mut TextureCache,
    ) -> GResult<GlyphInfo> {
        let font = self
            .font
            .as_ref()
            .ok_or_else(|| GError {
                kind: GErrorKind::Runtime, message: "未加载字体，无法光栅化字形".to_string()
            })?;

        let codepoint = glyph.id.0 as u32;
        let font_size = glyph.scale.y as u32;

        let outlined = font.outline_glyph(glyph);

        if let Some(outlined) = outlined {
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
            Ok(GlyphInfo {
                texture_id: TextureId::INVALID,
                uv_rect: [0.0, 0.0, 0.0, 0.0],
                size: [0.0, 0.0],
                offset: [0.0, 0.0],
            })
        }
    }

    /// 获取当前字体的引用
    pub fn font(&self) -> Option<&FontArc> {
        self.font.as_ref()
    }

    /// 检查是否已加载字体
    pub fn has_font(&self) -> bool {
        self.font.is_some()
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
