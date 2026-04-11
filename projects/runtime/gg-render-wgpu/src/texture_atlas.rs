use gg_core::{GError, GErrorKind, GResult};
use gg_render::TextureId;

use crate::texture_cache::TextureCache;

/// 纹理图集中的子区域
///
/// 描述一个纹理在图集中的位置和 UV 映射。
#[derive(Debug, Clone, Copy)]
pub struct AtlasRegion {
    /// 图集页纹理标识符
    pub texture_id: TextureId,
    /// 子区域在图集中的 UV 矩形 `[u_min, v_min, u_max, v_max]`
    pub uv_rect: [f32; 4],
    /// 子区域像素偏移 `[x, y]`
    pub offset: [u32; 2],
    /// 子区域像素尺寸 `[width, height]`
    pub size: [u32; 2],
}

/// Shelf 分配器
///
/// 将图集页按行（shelf）分配，每行高度由第一个分配的纹理决定。
/// 同行内按水平方向依次排列。
struct ShelfAllocator {
    /// 图集页宽度
    width: u32,
    /// 图集页高度
    height: u32,
    /// 当前 Y 偏移（下一行的起始 Y）
    current_y: u32,
    /// 当前行高度
    shelf_height: u32,
    /// 当前行内 X 偏移
    current_x: u32,
}

impl ShelfAllocator {
    fn new(width: u32, height: u32) -> Self {
        Self { width, height, current_y: 0, shelf_height: 0, current_x: 0 }
    }

    fn allocate(&mut self, req_width: u32, req_height: u32) -> Option<[u32; 2]> {
        if req_width > self.width || req_height > self.height {
            return None;
        }

        if self.current_x + req_width > self.width {
            self.current_y += self.shelf_height;
            self.shelf_height = 0;
            self.current_x = 0;
        }

        if self.current_y + req_height > self.height {
            return None;
        }

        let x = self.current_x;
        let y = self.current_y;

        self.current_x += req_width;
        self.shelf_height = self.shelf_height.max(req_height);

        Some([x, y])
    }
}

/// 图集页
struct AtlasPage {
    /// 页纹理标识符
    texture_id: TextureId,
    /// 页宽度
    width: u32,
    /// 页高度
    height: u32,
    /// Shelf 分配器
    allocator: ShelfAllocator,
}

/// 运行时纹理图集
///
/// 将不同纹理合并到同一图集页中，提升精灵批渲染的合批率。
/// 使用 Shelf 分配器管理图集页内的空间分配。
/// 当现有页空间不足时自动创建新页。
pub struct DynamicTextureAtlas {
    /// 图集页列表
    pages: Vec<AtlasPage>,
    /// 每页宽度
    page_width: u32,
    /// 每页高度
    page_height: u32,
    /// 纹理像素填充（像素间距）
    padding: u32,
}

impl DynamicTextureAtlas {
    /// 创建新的运行时纹理图集
    ///
    /// # 参数
    ///
    /// - `page_width` - 每页宽度（像素）
    /// - `page_height` - 每页高度（像素）
    /// - `padding` - 纹理间距（像素），防止纹理间渗色
    pub fn new(page_width: u32, page_height: u32, padding: u32) -> Self {
        Self { pages: Vec::new(), page_width, page_height, padding }
    }

    /// 分配纹理到图集
    ///
    /// 尝试将纹理数据分配到现有图集页，空间不足时创建新页。
    ///
    /// # 参数
    ///
    /// - `width` - 纹理宽度（像素）
    /// - `height` - 纹理高度（像素）
    /// - `data` - RGBA 像素数据
    /// - `device` - wgpu 设备
    /// - `queue` - wgpu 命令队列
    /// - `texture_cache` - 纹理缓存
    pub fn allocate(
        &mut self,
        width: u32,
        height: u32,
        data: &[u8],
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        texture_cache: &mut TextureCache,
    ) -> GResult<AtlasRegion> {
        let padded_width = width + self.padding * 2;
        let padded_height = height + self.padding * 2;

        let mut found: Option<(TextureId, [u32; 2], u32, u32)> = None;

        for page in &mut self.pages {
            if let Some([x, y]) = page.allocator.allocate(padded_width, padded_height) {
                found = Some((page.texture_id, [x, y], page.width, page.height));
                break;
            }
        }

        if found.is_none() {
            let page = self.create_new_page(device, texture_cache)?;
            if let Some([x, y]) = page.allocator.allocate(padded_width, padded_height) {
                found = Some((page.texture_id, [x, y], page.width, page.height));
            }
        }

        match found {
            Some((texture_id, [x, y], page_width, page_height)) => {
                let uv_min_x = x as f32 / page_width as f32;
                let uv_min_y = y as f32 / page_height as f32;
                let uv_max_x = (x + width) as f32 / page_width as f32;
                let uv_max_y = (y + height) as f32 / page_height as f32;

                Self::write_region_static(texture_id, x, y, width, height, data, queue, texture_cache);

                Ok(AtlasRegion {
                    texture_id,
                    uv_rect: [uv_min_x, uv_min_y, uv_max_x, uv_max_y],
                    offset: [x, y],
                    size: [width, height],
                })
            }
            None => Err(GError {
                kind: GErrorKind::Runtime,
                message: format!("纹理尺寸 {}x{} 超过图集页 {}x{}", width, height, self.page_width, self.page_height),
            }),
        }
    }

    /// 创建新的图集页
    fn create_new_page(&mut self, device: &wgpu::Device, texture_cache: &mut TextureCache) -> GResult<&mut AtlasPage> {
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some(&format!("dynamic_atlas_page_{}", self.pages.len())),
            size: wgpu::Extent3d { width: self.page_width, height: self.page_height, depth_or_array_layers: 1 },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });

        let texture_id = texture_cache.register_texture(texture);

        self.pages.push(AtlasPage {
            texture_id,
            width: self.page_width,
            height: self.page_height,
            allocator: ShelfAllocator::new(self.page_width, self.page_height),
        });

        Ok(self.pages.last_mut().unwrap())
    }

    /// 将像素数据写入图集页的指定区域
    fn write_region_static(
        page_texture_id: TextureId,
        x: u32,
        y: u32,
        width: u32,
        height: u32,
        data: &[u8],
        queue: &wgpu::Queue,
        texture_cache: &TextureCache,
    ) {
        let texture = match texture_cache.get(page_texture_id) {
            Some(t) => t,
            None => return,
        };

        queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture,
                mip_level: 0,
                origin: wgpu::Origin3d { x, y, z: 0 },
                aspect: wgpu::TextureAspect::All,
            },
            data,
            wgpu::TexelCopyBufferLayout { offset: 0, bytes_per_row: Some(4 * width), rows_per_image: Some(height) },
            wgpu::Extent3d { width, height, depth_or_array_layers: 1 },
        );
    }

    /// 获取图集页数量
    pub fn page_count(&self) -> usize {
        self.pages.len()
    }

    /// 清空所有图集页
    pub fn clear(&mut self) {
        self.pages.clear();
    }
}
