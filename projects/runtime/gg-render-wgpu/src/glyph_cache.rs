use std::collections::HashMap;

use ab_glyph::{Font, FontArc, Glyph};
use gg_core::{GError, GErrorKind, GResult};
use gg_render::TextureId;

use crate::texture_cache::TextureCache;

/// 字形信息
///
/// 描述一个已光栅化字形在纹理中的位置和属性。
#[derive(Debug, Clone, Copy)]
pub struct GlyphInfo {
    /// 字形所在纹理的标识符
    pub texture_id: TextureId,
    /// 字形在纹理中的 UV 矩形 `[u_min, v_min, u_max, v_max]`
    pub uv_rect: [f32; 4],
    /// 字形尺寸 `[width, height]`
    pub size: [f32; 2],
    /// 字形偏移 `[x_offset, y_offset]`
    pub offset: [f32; 2],
}

/// 字形缓存键
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct GlyphKey {
    /// 字符的 Unicode 码点
    codepoint: u32,
    /// 字体大小（像素高度的整数部分）
    font_size: u32,
}

/// 字形缓存
///
/// 使用 `ab_glyph` 库光栅化字形，并将结果缓存为 GPU 纹理。
/// 支持动态加载字体和按需光栅化字形。
///
/// 创建后需要通过 [`GlyphCache::load_font`] 加载字体才能渲染文本。
pub struct GlyphCache {
    /// 当前字体
    font: Option<FontArc>,
    /// 已缓存的字形映射
    cache: HashMap<GlyphKey, GlyphInfo>,
}

impl GlyphCache {
    /// 创建新的字形缓存
    ///
    /// 初始状态下不包含字体，需要调用 [`GlyphCache::load_font`] 加载字体后才能渲染文本。
    pub fn new() -> Self {
        Self {
            font: None,
            cache: HashMap::new(),
        }
    }

    /// 从字节数据加载字体
    ///
    /// 加载成功后会清空已有的字形缓存。
    ///
    /// # 参数
    ///
    /// - `data` - 字体文件的原始字节数据
    pub fn load_font(&mut self, data: Vec<u8>) {
        if let Ok(font) = FontArc::try_from_vec(data) {
            self.font = Some(font);
            self.cache.clear();
        }
    }

    /// 获取或光栅化字形
    ///
    /// 如果字形已缓存则直接返回缓存信息，
    /// 否则使用 `ab_glyph` 光栅化字形并上传到 GPU 纹理。
    ///
    /// # 参数
    ///
    /// - `glyph` - 要光栅化的字形
    /// - `device` - wgpu 设备
    /// - `queue` - wgpu 命令队列
    /// - `texture_cache` - 纹理缓存，用于存储光栅化结果
    ///
    /// # 返回值
    ///
    /// 成功时返回字形信息，如果未加载字体则返回错误
    pub fn get_or_rasterize(
        &mut self,
        glyph: Glyph,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        texture_cache: &mut TextureCache,
    ) -> GResult<GlyphInfo> {
        let font = self.font.as_ref().ok_or_else(|| GError {
            kind: GErrorKind::Runtime,
            message: "未加载字体，无法光栅化字形".to_string(),
        })?;

        let key = GlyphKey {
            codepoint: glyph.id.0 as u32,
            font_size: glyph.scale.y as u32,
        };

        if let Some(info) = self.cache.get(&key) {
            return Ok(*info);
        }

        let outlined = font.outline_glyph(glyph);
        let glyph_info = if let Some(outlined) = outlined {
            let bounds = outlined.px_bounds();
            let width = bounds.width().ceil() as u32;
            let height = bounds.height().ceil() as u32;

            if width == 0 || height == 0 {
                GlyphInfo {
                    texture_id: TextureId::INVALID,
                    uv_rect: [0.0, 0.0, 0.0, 0.0],
                    size: [0.0, 0.0],
                    offset: [bounds.min.x, bounds.min.y],
                }
            } else {
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

                let texture_id = texture_cache.create_texture_from_data(
                    width,
                    height,
                    &rgba_data,
                    device,
                    queue,
                    &format!("glyph_{}_{}", key.codepoint, key.font_size),
                )?;

                GlyphInfo {
                    texture_id,
                    uv_rect: [0.0, 0.0, 1.0, 1.0],
                    size: [width as f32, height as f32],
                    offset: [bounds.min.x, bounds.min.y],
                }
            }
        } else {
            GlyphInfo {
                texture_id: TextureId::INVALID,
                uv_rect: [0.0, 0.0, 0.0, 0.0],
                size: [0.0, 0.0],
                offset: [0.0, 0.0],
            }
        };

        self.cache.insert(key, glyph_info);
        Ok(glyph_info)
    }

    /// 获取当前字体的引用
    ///
    /// 如果已加载字体则返回 `Some`，否则返回 `None`。
    pub fn font(&self) -> Option<&FontArc> {
        self.font.as_ref()
    }

    /// 检查是否已加载字体
    pub fn has_font(&self) -> bool {
        self.font.is_some()
    }

    /// 清空字形缓存
    pub fn clear(&mut self) {
        self.cache.clear();
    }
}

impl Default for GlyphCache {
    fn default() -> Self {
        Self::new()
    }
}
