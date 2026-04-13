use std::collections::HashMap;

use gg_render::TextureId;

/// 离屏渲染目标
///
/// 封装一个可渲染的离屏纹理视图。
/// 渲染目标的纹理已注册到纹理缓存中，可通过 `texture_id` 作为精灵纹理使用。
/// 同时持有 `wgpu::Texture` 以确保 GPU 资源在视图存在期间保持存活。
pub struct RenderTarget {
    /// wgpu 纹理对象，保持 GPU 资源存活
    pub(crate) texture: wgpu::Texture,
    /// 纹理视图，用于渲染通道的颜色附件
    pub(crate) view: wgpu::TextureView,
    /// 纹理标识符，可用于精灵绘制
    pub(crate) texture_id: TextureId,
    /// 渲染目标宽度（像素）
    width: u32,
    /// 渲染目标高度（像素）
    height: u32,
}

impl RenderTarget {
    /// 创建新的渲染目标
    ///
    /// # 参数
    ///
    /// - `texture` - wgpu 纹理对象
    /// - `view` - 纹理视图
    /// - `texture_id` - 纹理标识符
    /// - `width` - 宽度（像素）
    /// - `height` - 高度（像素）
    pub(crate) fn new(texture: wgpu::Texture, view: wgpu::TextureView, texture_id: TextureId, width: u32, height: u32) -> Self {
        Self { texture, view, texture_id, width, height }
    }

    /// 获取纹理标识符
    ///
    /// 返回的标识符可用于精灵绘制命令中的纹理引用。
    pub fn texture_id(&self) -> TextureId {
        self.texture_id
    }

    /// 获取渲染目标宽度（像素）
    pub fn width(&self) -> u32 {
        self.width
    }

    /// 获取渲染目标高度（像素）
    pub fn height(&self) -> u32 {
        self.height
    }
}

/// RenderTarget 复用池
///
/// 缓存已释放的 RenderTarget，按尺寸分组管理，
/// 避免频繁的 GPU 纹理分配和释放。
pub struct RenderTargetPool {
    /// 按尺寸分组的可用 RenderTarget
    available: HashMap<(u32, u32), Vec<RenderTarget>>,
}

impl RenderTargetPool {
    /// 创建空的渲染目标复用池
    pub fn new() -> Self {
        Self { available: HashMap::new() }
    }

    /// 从池中获取指定尺寸的渲染目标
    ///
    /// 如果池中有匹配尺寸的缓存目标则返回，否则返回 `None`。
    ///
    /// # 参数
    ///
    /// - `width` - 所需宽度（像素）
    /// - `height` - 所需高度（像素）
    pub fn acquire(&mut self, width: u32, height: u32) -> Option<RenderTarget> {
        self.available.get_mut(&(width, height)).and_then(|vec| vec.pop())
    }

    /// 将渲染目标归还到池中
    ///
    /// 归还的目标可被后续的 `acquire` 调用复用，
    /// 避免重新分配 GPU 纹理资源。
    ///
    /// # 参数
    ///
    /// - `target` - 要归还的渲染目标
    pub fn release(&mut self, target: RenderTarget) {
        let key = (target.width(), target.height());
        self.available.entry(key).or_default().push(target);
    }

    /// 清空池中所有缓存的渲染目标
    ///
    /// 释放所有缓存的 GPU 纹理资源。
    pub fn clear(&mut self) {
        self.available.clear();
    }
}

impl Default for RenderTargetPool {
    fn default() -> Self {
        Self::new()
    }
}
