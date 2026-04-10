use std::collections::HashMap;
use std::path::Path;

use gg_core::{GError, GErrorKind, GResult};
use gg_render::TextureId;
use piet_common::{Device, Image, ImageFormat};

/// 原生纹理缓存
///
/// 使用 piet-common 的 `Image` 类型存储纹理，
/// 管理已加载的纹理资源，提供纹理的加载、热更新和查询功能。
/// 每个纹理通过唯一的 `TextureId` 标识，同时维护路径到标识符的映射以支持热更新。
pub struct NativeTextureCache {
    /// 已加载的纹理映射
    textures: HashMap<TextureId, Image>,
    /// 文件路径到纹理标识符的映射，用于热更新时查找
    path_to_id: HashMap<String, TextureId>,
    /// 纹理 ID 计数器，用于生成新的唯一 ID
    next_id: u64,
}

impl NativeTextureCache {
    /// 创建新的纹理缓存
    pub fn new() -> Self {
        Self {
            textures: HashMap::new(),
            path_to_id: HashMap::new(),
            next_id: 1,
        }
    }

    /// 从文件加载纹理资源
    ///
    /// 使用 `image` 库解码图像文件，转换为 piet `Image` 并存入缓存。
    /// 如果该路径已加载过，则返回已有纹理的标识符。
    ///
    /// # 参数
    ///
    /// - `path` - 纹理文件路径
    /// - `device` - piet 设备，用于创建图像资源
    ///
    /// # 返回值
    ///
    /// 成功时返回纹理标识符
    pub fn load_texture(&mut self, path: &Path, device: &mut Device) -> GResult<TextureId> {
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

        let piet_image = device
            .make_image(width as usize, height as usize, rgba.as_raw(), ImageFormat::RgbaSeparate)
            .map_err(|e| GError {
                kind: GErrorKind::Platform,
                message: format!("无法创建 piet 图像 {:?}: {}", path, e),
            })?;

        let id = TextureId::new(self.next_id);
        self.next_id += 1;
        self.textures.insert(id, piet_image);
        self.path_to_id.insert(path_str, id);
        Ok(id)
    }

    /// 热更新纹理
    ///
    /// 从指定路径重新加载纹理数据并更新缓存中的图像。
    /// 如果加载失败，保留旧纹理不变。
    /// 如果该路径尚未加载过，则作为新纹理加载并存入缓存。
    ///
    /// # 参数
    ///
    /// - `path` - 纹理文件路径
    /// - `device` - piet 设备，用于创建图像资源
    pub fn reload_texture(&mut self, path: &str, device: &mut Device) -> GResult<()> {
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

        let piet_image = device
            .make_image(width as usize, height as usize, rgba.as_raw(), ImageFormat::RgbaSeparate)
            .map_err(|e| GError {
                kind: GErrorKind::Platform,
                message: format!("无法创建 piet 图像 {:?}: {}", path_ref, e),
            })?;

        if let Some(&id) = self.path_to_id.get(path) {
            self.textures.insert(id, piet_image);
        } else {
            let id = TextureId::new(self.next_id);
            self.next_id += 1;
            self.textures.insert(id, piet_image);
            self.path_to_id.insert(path.to_string(), id);
        }

        Ok(())
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
    pub fn get(&self, id: TextureId) -> Option<&Image> {
        self.textures.get(&id)
    }

    /// 根据标识符查找纹理的可变引用
    ///
    /// # 参数
    ///
    /// - `id` - 纹理标识符
    ///
    /// # 返回值
    ///
    /// 如果找到则返回纹理可变引用，否则返回 `None`
    pub fn get_mut(&mut self, id: TextureId) -> Option<&mut Image> {
        self.textures.get_mut(&id)
    }
}

impl Default for NativeTextureCache {
    fn default() -> Self {
        Self::new()
    }
}
