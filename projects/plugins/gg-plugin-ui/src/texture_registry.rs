//! UI 纹理注册表模块
//! 提供从文件路径到纹理标识符的映射管理

use std::collections::HashMap;
use std::path::Path;

use gg_core::GResult;
use gg_render::{Renderer, TextureId};

/// 纹理注册表
///
/// 管理从文件路径到纹理标识符的映射，
/// 使 UI 系统可通过路径引用已加载的纹理资源。
#[derive(Debug, Clone, Default)]
pub struct TextureRegistry {
    /// 路径到纹理标识符的映射
    path_to_id: HashMap<String, TextureId>,
}

impl TextureRegistry {
    /// 创建新的纹理注册表
    pub fn new() -> Self {
        Self::default()
    }

    /// 从文件加载纹理并注册到注册表
    ///
    /// 通过渲染器加载纹理文件，并将路径到纹理标识符的映射记录到注册表中。
    pub fn load(&mut self, renderer: &mut dyn Renderer, path: &str) -> GResult<TextureId> {
        if let Some(&id) = self.path_to_id.get(path) {
            return Ok(id);
        }
        let texture_id = renderer.load_texture(Path::new(path))?;
        self.path_to_id.insert(path.to_string(), texture_id);
        Ok(texture_id)
    }

    /// 通过路径查询纹理标识符
    ///
    /// 如果指定路径的纹理已注册，返回对应的纹理标识符；否则返回 `None`。
    pub fn get(&self, path: &str) -> Option<TextureId> {
        self.path_to_id.get(path).copied()
    }

    /// 检查指定路径的纹理是否已注册
    pub fn contains(&self, path: &str) -> bool {
        self.path_to_id.contains_key(path)
    }

    /// 移除指定路径的纹理注册
    ///
    /// 返回被移除的纹理标识符（如果存在）。
    pub fn remove(&mut self, path: &str) -> Option<TextureId> {
        self.path_to_id.remove(path)
    }

    /// 清空所有纹理注册
    pub fn clear(&mut self) {
        self.path_to_id.clear();
    }
}
