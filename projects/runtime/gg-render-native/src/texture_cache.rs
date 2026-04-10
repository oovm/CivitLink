use std::collections::HashMap;
use std::path::Path;

use gg_core::{GError, GErrorKind, GResult};
use gg_render::TextureId;
use piet_common::{PietImage, RenderContext as PietRenderContext};

/// 原始纹理数据
///
/// 存储从图像文件解码得到的原始像素数据，
/// 在渲染时通过 piet 的 RenderContext 创建平台原生图像。
struct RawTexture {
    /// 像素宽度
    width: usize,
    /// 像素高度
    height: usize,
    /// RGBA 像素数据
    data: Vec<u8>,
}

/// 原生纹理缓存
///
/// 管理已加载的纹理资源，提供纹理的加载、热更新和查询功能。
/// 使用原始像素数据存储纹理，在渲染时通过 piet 的 RenderContext 创建平台原生图像。
/// 每个纹理通过唯一的 `TextureId` 标识，同时维护路径到标识符的映射以支持热更新。
pub struct NativeTextureCache {
    /// 原始纹理数据映射
    raw_textures: HashMap<TextureId, RawTexture>,
    /// 已创建的 piet 图像映射（按帧缓存）
    piet_images: HashMap<TextureId, PietImage>,
    /// 文件路径到纹理标识符的映射，用于热更新时查找
    path_to_id: HashMap<String, TextureId>,
    /// 纹理 ID 计数器，用于生成新的唯一 ID
    next_id: u64,
}

impl NativeTextureCache {
    /// 创建新的纹理缓存
    pub fn new() -> Self {
        Self {
            raw_textures: HashMap::new(),
            piet_images: HashMap::new(),
            path_to_id: HashMap::new(),
            next_id: 1,
        }
    }

    /// 从文件加载纹理资源
    ///
    /// 使用 `image` 库解码图像文件，存储原始像素数据。
    /// 如果该路径已加载过，则返回已有纹理的标识符。
    ///
    /// # 参数
    ///
    /// - `path` - 纹理文件路径
    ///
    /// # 返回值
    ///
    /// 成功时返回纹理标识符
    pub fn load_texture(&mut self, path: &Path) -> GResult<TextureId> {
        let path_str = path.to_string_lossy().to_string();

        if let Some(&id) = self.path_to_id.get(&path_str) {
            return Ok(id);
        }

        let data = std::fs::read(path).map_err(|e| GError {
            kind: GErrorKind::Io,
            message: format!("无法读取纹理文件 {:?}: {}", path, e),
        })?;

        let img = image::load_from_memory(&data).map_err(|e| GError {
            kind: GErrorKind::Asset,
            message: format!("无法解码纹理图像 {:?}: {}", path, e),
        })?;

        let rgba = img.to_rgba8();
        let (width, height) = rgba.dimensions();

        let id = TextureId::new(self.next_id);
        self.next_id += 1;
        self.raw_textures.insert(id, RawTexture { width: width as usize, height: height as usize, data: rgba.into_raw() });
        self.piet_images.remove(&id);
        self.path_to_id.insert(path_str, id);
        Ok(id)
    }

    /// 热更新纹理
    ///
    /// 从指定路径重新加载纹理数据并更新缓存。
    /// 如果加载失败，保留旧纹理不变。
    /// 如果该路径尚未加载过，则作为新纹理加载并存入缓存。
    ///
    /// # 参数
    ///
    /// - `path` - 纹理文件路径
    pub fn reload_texture(&mut self, path: &str) -> GResult<()> {
        let path_ref = Path::new(path);

        let data = std::fs::read(path_ref).map_err(|e| GError {
            kind: GErrorKind::Io,
            message: format!("无法读取纹理文件 {:?}: {}", path_ref, e),
        })?;

        let img = image::load_from_memory(&data).map_err(|e| GError {
            kind: GErrorKind::Asset,
            message: format!("无法解码纹理图像 {:?}: {}", path_ref, e),
        })?;

        let rgba = img.to_rgba8();
        let (width, height) = rgba.dimensions();

        if let Some(&id) = self.path_to_id.get(path) {
            self.raw_textures.insert(id, RawTexture { width: width as usize, height: height as usize, data: rgba.into_raw() });
            self.piet_images.remove(&id);
        } else {
            let id = TextureId::new(self.next_id);
            self.next_id += 1;
            self.raw_textures.insert(id, RawTexture { width: width as usize, height: height as usize, data: rgba.into_raw() });
            self.path_to_id.insert(path.to_string(), id);
        }

        Ok(())
    }

    /// 根据标识符获取或创建 piet 图像
    ///
    /// 如果图像已缓存则直接返回，否则从原始像素数据创建并缓存。
    ///
    /// # 参数
    ///
    /// - `id` - 纹理标识符
    /// - `rc` - piet 渲染上下文，用于创建图像
    ///
    /// # 返回值
    ///
    /// 如果找到或成功创建则返回图像引用，否则返回 `None`
    pub fn get_or_create(&mut self, id: TextureId, rc: &mut impl PietRenderContext<Image = PietImage>) -> Option<&PietImage> {
        if self.piet_images.contains_key(&id) {
            return self.piet_images.get(&id);
        }

        let raw = self.raw_textures.get(&id)?;
        let piet_image = rc
            .make_image(raw.width, raw.height, &raw.data, piet_common::ImageFormat::RgbaSeparate)
            .ok()?;

        self.piet_images.insert(id, piet_image);
        self.piet_images.get(&id)
    }

    /// 使所有缓存的 piet 图像失效
    ///
    /// 在新帧开始时调用，确保图像与当前渲染上下文兼容。
    pub fn invalidate_piet_images(&mut self) {
        self.piet_images.clear();
    }
}

impl Default for NativeTextureCache {
    fn default() -> Self {
        Self::new()
    }
}
