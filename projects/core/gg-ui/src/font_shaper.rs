use std::sync::{Arc, RwLock};

use ab_glyph::{FontArc, PxScale, ScaleFont};
use crate::font::{ShapedGlyph, ShapedText, TextShaper};

static GLOBAL_SHAPER: RwLock<Option<Arc<AbGlyphTextShaper>>> = RwLock::new(None);

/// 基于 ab_glyph 的文本整形器实现
pub struct AbGlyphTextShaper {
    /// 字体数据
    font: FontArc,
}

impl AbGlyphTextShaper {
    /// 从字体文件数据创建整形器
    pub fn from_font_data(data: Vec<u8>) -> Result<Self, ab_glyph::InvalidFont> {
        let font = FontArc::try_from_vec(data)?;
        Ok(Self { font })
    }

    /// 从 FontArc 创建整形器
    pub fn from_font_arc(font: FontArc) -> Self {
        Self { font }
    }

    /// 设置全局整形器实例
    pub fn set_global(shaper: AbGlyphTextShaper) {
        let mut global = GLOBAL_SHAPER.write().unwrap();
        *global = Some(Arc::new(shaper));
    }

    /// 尝试获取全局整形器实例
    pub fn try_global() -> Option<Arc<AbGlyphTextShaper>> {
        let global = GLOBAL_SHAPER.read().unwrap();
        global.clone()
    }
}

impl TextShaper for AbGlyphTextShaper {
    fn shape_text(&mut self, text: &str, font_size: f32, max_width: Option<f32>) -> ShapedText {
        let scale = PxScale::from(font_size);
        let scaled_font = self.font.as_scaled(scale);

        let mut glyphs = Vec::new();
        let mut cursor_x = 0.0f32;
        let mut cursor_y = 0.0f32;
        let mut line_width = 0.0f32;
        let mut max_width_seen = 0.0f32;
        let line_height = scaled_font.height();

        let mut prev_glyph_id = None;

        for c in text.chars() {
            if c == '\n' {
                max_width_seen = max_width_seen.max(line_width);
                line_width = 0.0;
                cursor_x = 0.0;
                cursor_y += line_height;
                prev_glyph_id = None;
                continue;
            }

            let glyph_id = scaled_font.glyph_id(c);

            if let Some(prev_id) = prev_glyph_id {
                cursor_x += scaled_font.kern(prev_id, glyph_id);
            }

            if let Some(max_w) = max_width {
                let advance = scaled_font.h_advance(glyph_id);
                if line_width + advance > max_w && line_width > 0.0 {
                    max_width_seen = max_width_seen.max(line_width);
                    line_width = 0.0;
                    cursor_x = 0.0;
                    cursor_y += line_height;
                    prev_glyph_id = None;
                }
            }

            let advance = scaled_font.h_advance(glyph_id);

            glyphs.push(ShapedGlyph {
                glyph_id: glyph_id.0 as u32,
                x: cursor_x,
                y: cursor_y,
                advance,
            });

            cursor_x += advance;
            line_width += advance;
            prev_glyph_id = Some(glyph_id);
        }

        max_width_seen = max_width_seen.max(line_width);
        let total_height = if text.is_empty() { line_height } else { cursor_y + line_height };

        ShapedText {
            glyphs,
            width: max_width_seen,
            height: total_height,
        }
    }

    fn measure_text(&self, text: &str, font_size: f32, max_width: Option<f32>) -> (f32, f32) {
        let scale = PxScale::from(font_size);
        let scaled_font = self.font.as_scaled(scale);

        let mut line_width = 0.0f32;
        let mut max_line_width = 0.0f32;
        let mut line_count = 1u32;
        let line_height = scaled_font.height();

        let mut prev_glyph_id = None;

        for c in text.chars() {
            if c == '\n' {
                max_line_width = max_line_width.max(line_width);
                line_width = 0.0;
                line_count += 1;
                prev_glyph_id = None;
                continue;
            }

            let glyph_id = scaled_font.glyph_id(c);

            if let Some(prev_id) = prev_glyph_id {
                line_width += scaled_font.kern(prev_id, glyph_id);
            }

            let advance = scaled_font.h_advance(glyph_id);

            if let Some(max_w) = max_width {
                if line_width + advance > max_w && line_width > 0.0 {
                    max_line_width = max_line_width.max(line_width);
                    line_width = 0.0;
                    line_count += 1;
                    prev_glyph_id = None;
                    continue;
                }
            }

            line_width += advance;
            prev_glyph_id = Some(glyph_id);
        }

        max_line_width = max_line_width.max(line_width);
        (max_line_width, line_height * line_count as f32)
    }
}
