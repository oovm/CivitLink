use std::{collections::HashMap, path::Path};

use gg_core::{GError, GErrorKind, GResult};
use gg_render::TextureId;

/// 纹理缓存
///
/// 管理已加载的纹理资源，提供纹理的加载、查询和绑定组创建功能。
/// 每个纹理通过唯一的 `TextureId` 标识。
pub struct TextureCache {
    /// 已加载的纹理映射
    textures: HashMap<TextureId, wgpu::Texture>,
    /// 纹理 ID 计数器，用于生成新的唯一 ID
    next_id: u64,
}

impl TextureCache {
    /// 创建新的纹理缓存
    pub fn new() -> Self {
        Self { textures: HashMap::new(), next_id: 1 }
    }

    /// 从文件加载纹理资源
    ///
    /// 使用 `image` 库解码图像文件，创建 wgpu 纹理并上传到 GPU。
    ///
    /// # 参数
    ///
    /// - `path` - 纹理文件路径
    /// - `device` - wgpu 设备，用于创建纹理资源
    /// - `queue` - wgpu 命令队列，用于上传纹理数据
    ///
    /// # 返回值
    ///
    /// 成功时返回新分配的纹理标识符
    pub fn load_texture(&mut self, path: &Path, device: &wgpu::Device, queue: &wgpu::Queue) -> GResult<TextureId> {
        let data = std::fs::read(path)
            .map_err(|e| GError { kind: GErrorKind::Io, message: format!("无法读取纹理文件 {:?}: {}", path, e) })?;

        let img = image::load_from_memory(&data)
            .map_err(|e| GError { kind: GErrorKind::Asset, message: format!("无法解码纹理图像 {:?}: {}", path, e) })?;

        let rgba = img.to_rgba8();
        let dimensions = rgba.dimensions();

        let size = wgpu::Extent3d { width: dimensions.0, height: dimensions.1, depth_or_array_layers: 1 };

        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some(&format!("texture_{:?}", path)),
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });

        queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &rgba,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(4 * dimensions.0),
                rows_per_image: Some(dimensions.1),
            },
            size,
        );

        let id = TextureId::new(self.next_id);
        self.next_id += 1;
        self.textures.insert(id, texture);
        Ok(id)
    }

    /// 从原始 RGBA 数据创建纹理
    ///
    /// # 参数
    ///
    /// - `width` - 纹理宽度（像素）
    /// - `height` - 纹理高度（像素）
    /// - `data` - RGBA 像素数据
    /// - `device` - wgpu 设备
    /// - `queue` - wgpu 命令队列
    /// - `label` - 纹理标签，用于调试
    ///
    /// # 返回值
    ///
    /// 成功时返回新分配的纹理标识符
    pub fn create_texture_from_data(
        &mut self,
        width: u32,
        height: u32,
        data: &[u8],
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        label: &str,
    ) -> GResult<TextureId> {
        let size = wgpu::Extent3d { width, height, depth_or_array_layers: 1 };

        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some(label),
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });

        queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            data,
            wgpu::TexelCopyBufferLayout { offset: 0, bytes_per_row: Some(4 * width), rows_per_image: Some(height) },
            size,
        );

        let id = TextureId::new(self.next_id);
        self.next_id += 1;
        self.textures.insert(id, texture);
        Ok(id)
    }

    /// 根据标识符查找纹理
    ///
    /// # 参数
    ///
    /// - `id` - 纹理标识符
    ///
    /// # 返回值
    ///
    /// 如果找到则返回纹理引用，否则返回 `None`
    pub fn get(&self, id: TextureId) -> Option<&wgpu::Texture> {
        self.textures.get(&id)
    }

    /// 为指定纹理创建绑定组
    ///
    /// 创建包含采样器和纹理视图的绑定组，用于着色器资源绑定。
    ///
    /// # 参数
    ///
    /// - `id` - 纹理标识符
    /// - `device` - wgpu 设备
    /// - `layout` - 绑定组布局
    ///
    /// # 返回值
    ///
    /// 如果纹理存在则返回绑定组，否则返回 `None`
    pub fn create_bind_group(
        &self,
        id: TextureId,
        device: &wgpu::Device,
        layout: &wgpu::BindGroupLayout,
    ) -> Option<wgpu::BindGroup> {
        let texture = self.textures.get(&id)?;
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        Some(device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("texture_bind_group"),
            layout,
            entries: &[
                wgpu::BindGroupEntry { binding: 0, resource: wgpu::BindingResource::Sampler(&sampler) },
                wgpu::BindGroupEntry { binding: 1, resource: wgpu::BindingResource::TextureView(&view) },
            ],
        }))
    }

    /// 为过渡着色器创建包含两个纹理的绑定组
    ///
    /// # 参数
    ///
    /// - `old_id` - 旧纹理标识符
    /// - `new_id` - 新纹理标识符
    /// - `device` - wgpu 设备
    /// - `layout` - 绑定组布局
    ///
    /// # 返回值
    ///
    /// 如果两个纹理都存在则返回绑定组，否则返回 `None`
    pub fn create_transition_bind_group(
        &self,
        old_id: TextureId,
        new_id: TextureId,
        device: &wgpu::Device,
        layout: &wgpu::BindGroupLayout,
    ) -> Option<wgpu::BindGroup> {
        let old_texture = self.textures.get(&old_id)?;
        let new_texture = self.textures.get(&new_id)?;
        let old_view = old_texture.create_view(&wgpu::TextureViewDescriptor::default());
        let new_view = new_texture.create_view(&wgpu::TextureViewDescriptor::default());
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        Some(device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("transition_bind_group"),
            layout,
            entries: &[
                wgpu::BindGroupEntry { binding: 0, resource: wgpu::BindingResource::Sampler(&sampler) },
                wgpu::BindGroupEntry { binding: 1, resource: wgpu::BindingResource::TextureView(&old_view) },
                wgpu::BindGroupEntry { binding: 2, resource: wgpu::BindingResource::TextureView(&new_view) },
            ],
        }))
    }
}

impl Default for TextureCache {
    fn default() -> Self {
        Self::new()
    }
}
