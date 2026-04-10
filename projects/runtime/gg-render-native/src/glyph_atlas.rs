//! 字形纹理图集模块
//!
//! 将多个字形光栅化结果打包到同一纹理图集中，
//! 减少 GPU 纹理绑定次数，提升文本渲染性能。

use std::collections::HashMap;

use gg_core::{GError, GErrorKind, GResult};
use gg_render::TextureId;

use crate::texture_cache::TextureCache;

/// 图集页中的字形位置
#[derive(Debug, Clone, Copy)]
pub struct AtlasGlyphPlacement {
    /// 字形所在图集页的纹理标识符
    pub texture_id: TextureId,
    /// 字形在图集中的 UV 矩形 `[u_min, v_min, u_max, v_max]`
    pub uv_rect: [f32; 4],
    /// 字形尺寸 `[width, height]`（像素）
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

/// Shelf-based 图集分配器
///
/// 使用 shelf（行）算法将字形按行排列到图集中。
/// 当前行空间不足时开启新行，当图集空间不足时分配新页。
struct ShelfAllocator {
    /// 图集宽度（像素）
    width: u32,
    /// 图集高度（像素）
    height: u32,
    /// 当前行的 Y 坐标
    shelf_y: u32,
    /// 当前行的高度（取行内最高字形）
    shelf_height: u32,
    /// 当前行内下一个可用 X 坐标
    cursor_x: u32,
}

impl ShelfAllocator {
    /// 创建新的 shelf 分配器
    fn new(width: u32, height: u32) -> Self {
        Self { width, height, shelf_y: 0, shelf_height: 0, cursor_x: 0 }
    }

    /// 在图集中分配指定大小的区域
    ///
    /// 返回 `Some((x, y))` 分配成功的左上角坐标，
    /// 若空间不足返回 `None`。
    fn allocate(&mut self, glyph_width: u32, glyph_height: u32) -> Option<(u32, u32)> {
        if glyph_width > self.width || glyph_height > self.height {
            return None;
        }

        if self.cursor_x + glyph_width <= self.width && glyph_height <= self.shelf_height {
            let x = self.cursor_x;
            self.cursor_x += glyph_width;
            return Some((x, self.shelf_y));
        }

        let new_shelf_y = self.shelf_y + self.shelf_height;
        if new_shelf_y + glyph_height > self.height {
            return None;
        }

        self.shelf_y = new_shelf_y;
        self.shelf_height = glyph_height;
        self.cursor_x = glyph_width;

        Some((0, self.shelf_y))
    }

    /// 重置分配器
    fn reset(&mut self) {
        self.shelf_y = 0;
        self.shelf_height = 0;
        self.cursor_x = 0;
    }
}

/// 单个图集页
struct AtlasPage {
    /// 分配器
    allocator: ShelfAllocator,
    /// 纹理标识符
    texture_id: TextureId,
    /// 页内像素数据（用于写入字形后更新纹理）
    pixel_data: Vec<u8>,
    /// 页是否已被修改（需要上传到 GPU）
    dirty: bool,
}

/// 字形纹理图集
///
/// 将多个字形打包到同一纹理图集中，减少 GPU 纹理绑定次数。
/// 当图集空间不足时自动分配新页。
pub struct GlyphAtlas {
    /// 图集页列表
    pages: Vec<AtlasPage>,
    /// 字形缓存映射
    cache: HashMap<GlyphKey, AtlasGlyphPlacement>,
    /// 图集宽度（像素）
    width: u32,
    /// 图集高度（像素）
    height: u32,
}

impl GlyphAtlas {
    /// 默认图集宽度
    pub const DEFAULT_WIDTH: u32 = 512;
    /// 默认图集高度
    pub const DEFAULT_HEIGHT: u32 = 512;

    /// 创建新的字形纹理图集
    pub fn new() -> Self {
        Self { pages: Vec::new(), cache: HashMap::new(), width: Self::DEFAULT_WIDTH, height: Self::DEFAULT_HEIGHT }
    }

    /// 使用指定尺寸创建字形纹理图集
    pub fn with_size(width: u32, height: u32) -> Self {
        Self { pages: Vec::new(), cache: HashMap::new(), width, height }
    }

    /// 获取或光栅化字形到图集中
    ///
    /// 如果字形已缓存则直接返回位置信息，
    /// 否则光栅化字形并放入图集中。
    ///
    /// # 参数
    ///
    /// - `codepoint` - 字符的 Unicode 码点
    /// - `font_size` - 字体大小
    /// - `glyph_width` - 光栅化后的字形宽度
    /// - `glyph_height` - 光栅化后的字形高度
    /// - `offset_x` - 字形 X 偏移
    /// - `offset_y` - 字形 Y 偏移
    /// - `rgba_data` - 光栅化后的 RGBA 像素数据
    /// - `device` - wgpu 设备
    /// - `queue` - wgpu 命令队列
    /// - `texture_cache` - 纹理缓存
    pub fn get_or_allocate(
        &mut self,
        codepoint: u32,
        font_size: u32,
        glyph_width: u32,
        glyph_height: u32,
        offset_x: f32,
        offset_y: f32,
        rgba_data: &[u8],
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        texture_cache: &mut TextureCache,
    ) -> GResult<AtlasGlyphPlacement> {
        let key = GlyphKey { codepoint, font_size };

        if let Some(placement) = self.cache.get(&key) {
            return Ok(*placement);
        }

        if glyph_width == 0 || glyph_height == 0 {
            let placement = AtlasGlyphPlacement {
                texture_id: TextureId::INVALID,
                uv_rect: [0.0, 0.0, 0.0, 0.0],
                size: [0.0, 0.0],
                offset: [offset_x, offset_y],
            };
            self.cache.insert(key, placement);
            return Ok(placement);
        }

        let placement =
            self.allocate_glyph(glyph_width, glyph_height, offset_x, offset_y, rgba_data, device, queue, texture_cache)?;

        self.cache.insert(key, placement);
        Ok(placement)
    }

    /// 在图集中分配空间并写入字形数据
    fn allocate_glyph(
        &mut self,
        glyph_width: u32,
        glyph_height: u32,
        offset_x: f32,
        offset_y: f32,
        rgba_data: &[u8],
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        texture_cache: &mut TextureCache,
    ) -> GResult<AtlasGlyphPlacement> {
        for page_idx in 0..self.pages.len() {
            if let Some((x, y)) = self.pages[page_idx].allocator.allocate(glyph_width, glyph_height) {
                return self.write_glyph_to_page(
                    page_idx,
                    x,
                    y,
                    glyph_width,
                    glyph_height,
                    offset_x,
                    offset_y,
                    rgba_data,
                    queue,
                    texture_cache,
                );
            }
        }

        let page_idx = self.create_new_page(device, queue, texture_cache)?;

        if let Some((x, y)) = self.pages[page_idx].allocator.allocate(glyph_width, glyph_height) {
            return self.write_glyph_to_page(
                page_idx,
                x,
                y,
                glyph_width,
                glyph_height,
                offset_x,
                offset_y,
                rgba_data,
                queue,
                texture_cache,
            );
        }

        Err(GError { kind: GErrorKind::Runtime, message: "字形尺寸超过图集页大小".to_string() })
    }

    /// 创建新的图集页
    fn create_new_page(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        texture_cache: &mut TextureCache,
    ) -> GResult<usize> {
        let pixel_data = vec![0u8; (self.width * self.height * 4) as usize];
        let texture_id = texture_cache.create_texture_from_data(
            self.width,
            self.height,
            &pixel_data,
            device,
            queue,
            &format!("glyph_atlas_page_{}", self.pages.len()),
        )?;

        let page = AtlasPage { allocator: ShelfAllocator::new(self.width, self.height), texture_id, pixel_data, dirty: false };

        self.pages.push(page);
        Ok(self.pages.len() - 1)
    }

    /// 将字形数据写入图集页
    fn write_glyph_to_page(
        &mut self,
        page_idx: usize,
        x: u32,
        y: u32,
        glyph_width: u32,
        glyph_height: u32,
        offset_x: f32,
        offset_y: f32,
        rgba_data: &[u8],
        queue: &wgpu::Queue,
        texture_cache: &TextureCache,
    ) -> GResult<AtlasGlyphPlacement> {
        let page = &mut self.pages[page_idx];

        for row in 0..glyph_height {
            let src_offset = (row * glyph_width * 4) as usize;
            let dst_offset = ((y + row) * self.width * 4 + x * 4) as usize;
            let copy_len = (glyph_width * 4) as usize;
            page.pixel_data[dst_offset..dst_offset + copy_len].copy_from_slice(&rgba_data[src_offset..src_offset + copy_len]);
        }

        let texture_id = page.texture_id;
        if let Some(texture) = texture_cache.get_texture(texture_id) {
            queue.write_texture(
                wgpu::TexelCopyTextureInfo {
                    texture,
                    mip_level: 0,
                    origin: wgpu::Origin3d { x, y, z: 0 },
                    aspect: wgpu::TextureAspect::All,
                },
                rgba_data,
                wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(glyph_width * 4),
                    rows_per_image: Some(glyph_height),
                },
                wgpu::Extent3d { width: glyph_width, height: glyph_height, depth_or_array_layers: 1 },
            );
        }

        let uv_rect = [
            x as f32 / self.width as f32,
            y as f32 / self.height as f32,
            (x + glyph_width) as f32 / self.width as f32,
            (y + glyph_height) as f32 / self.height as f32,
        ];

        Ok(AtlasGlyphPlacement {
            texture_id,
            uv_rect,
            size: [glyph_width as f32, glyph_height as f32],
            offset: [offset_x, offset_y],
        })
    }

    /// 清空所有缓存
    pub fn clear(&mut self) {
        self.pages.clear();
        self.cache.clear();
    }
}

impl Default for GlyphAtlas {
    fn default() -> Self {
        Self::new()
    }
}
