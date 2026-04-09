#![warn(missing_docs)]

//! GG 引擎资源管理模块
//! 提供资源加载和管理功能

use gg_core::{GResult, GError, GErrorKind};
use std::any::Any;
use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::sync::Arc;

/// 资源 trait
pub trait Asset: Send + Sync {
    /// 资源类型名称
    fn name(&self) -> &str;
    
    /// 资源大小
    fn size(&self) -> usize;
}

/// 资源句柄
pub type AssetHandle<T> = Arc<T>;

/// 资源加载器 trait
pub trait AssetLoader<T: Asset> {
    /// 加载资源
    fn load(&self, path: &Path) -> GResult<T>;
}

/// 资源管理器
pub struct AssetManager {
    /// 资源存储
    assets: HashMap<String, Arc<dyn Any + Send + Sync>>,
    /// 资源加载器
    loaders: HashMap<String, Box<dyn Fn(&Path) -> GResult<Arc<dyn Any + Send + Sync>> + Send + Sync>>,
}

impl AssetManager {
    /// 创建新的资源管理器
    pub fn new() -> Self {
        Self {
            assets: HashMap::new(),
            loaders: HashMap::new(),
        }
    }
    
    /// 注册资源加载器
    pub fn register_loader<T: Asset + 'static>(&mut self, extension: &str, loader: impl AssetLoader<T> + Send + Sync + 'static) {
        self.loaders.insert(extension.to_string(), Box::new(move |path| {
            let asset = loader.load(path)?;
            Ok(Arc::new(asset) as Arc<dyn Any + Send + Sync>)
        }));
    }
    
    /// 加载资源
    pub fn load<T: Asset + 'static>(&mut self, path: &Path) -> GResult<AssetHandle<T>> {
        let path_str = path.to_str().ok_or_else(|| GError {
            kind: GErrorKind::Asset,
            message: "Invalid path".to_string(),
        })?;
        
        // 检查是否已加载
        if let Some(asset) = self.assets.get(path_str) {
            if let Ok(asset) = asset.clone().downcast::<T>() {
                return Ok(asset);
            }
        }
        
        // 获取文件扩展名
        let extension = path.extension()
            .and_then(|ext| ext.to_str())
            .ok_or_else(|| GError {
                kind: GErrorKind::Asset,
                message: "No file extension".to_string(),
            })?;
        
        // 查找加载器
        let loader = self.loaders.get(extension)
            .ok_or_else(|| GError {
                kind: GErrorKind::Asset,
                message: format!("No loader for extension: {}", extension),
            })?;
        
        // 加载资源
        let asset = loader(path)?;
        
        // 存储资源
        self.assets.insert(path_str.to_string(), asset.clone());
        
        // 类型转换
        asset.downcast::<T>()
            .map_err(|_| GError {
                kind: GErrorKind::Asset,
                message: "Asset type mismatch".to_string(),
            })
    }
    
    /// 获取资源
    pub fn get<T: Asset + 'static>(&self, path: &str) -> Option<AssetHandle<T>> {
        self.assets.get(path)
            .and_then(|asset| asset.clone().downcast::<T>().ok())
    }
    
    /// 卸载资源
    pub fn unload(&mut self, path: &str) -> bool {
        self.assets.remove(path).is_some()
    }
    
    /// 清理所有资源
    pub fn clear(&mut self) {
        self.assets.clear();
    }
}

/// 文本资源
pub struct TextAsset {
    /// 文本内容
    content: String,
    /// 资源名称
    name: String,
}

impl TextAsset {
    /// 创建新的文本资源
    pub fn new(name: &str, content: String) -> Self {
        Self {
            content,
            name: name.to_string(),
        }
    }
    
    /// 获取文本内容
    pub fn content(&self) -> &str {
        &self.content
    }
}

impl Asset for TextAsset {
    fn name(&self) -> &str {
        &self.name
    }
    
    fn size(&self) -> usize {
        self.content.len()
    }
}

/// 文本资源加载器
pub struct TextLoader;

impl AssetLoader<TextAsset> for TextLoader {
    fn load(&self, path: &Path) -> GResult<TextAsset> {
        let mut file = File::open(path)
            .map_err(|e| GError {
                kind: GErrorKind::Io,
                message: format!("Failed to open file: {}", e),
            })?;
        
        let mut content = String::new();
        file.read_to_string(&mut content)
            .map_err(|e| GError {
                kind: GErrorKind::Io,
                message: format!("Failed to read file: {}", e),
            })?;
        
        let name = path.file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("unknown");
        
        Ok(TextAsset::new(name, content))
    }
}

/// 二进制资源
pub struct BinaryAsset {
    /// 二进制数据
    data: Vec<u8>,
    /// 资源名称
    name: String,
}

impl BinaryAsset {
    /// 创建新的二进制资源
    pub fn new(name: &str, data: Vec<u8>) -> Self {
        Self {
            data,
            name: name.to_string(),
        }
    }
    
    /// 获取二进制数据
    pub fn data(&self) -> &[u8] {
        &self.data
    }
}

impl Asset for BinaryAsset {
    fn name(&self) -> &str {
        &self.name
    }
    
    fn size(&self) -> usize {
        self.data.len()
    }
}

/// 二进制资源加载器
pub struct BinaryLoader;

impl AssetLoader<BinaryAsset> for BinaryLoader {
    fn load(&self, path: &Path) -> GResult<BinaryAsset> {
        let mut file = File::open(path)
            .map_err(|e| GError {
                kind: GErrorKind::Io,
                message: format!("Failed to open file: {}", e),
            })?;
        
        let mut data = Vec::new();
        file.read_to_end(&mut data)
            .map_err(|e| GError {
                kind: GErrorKind::Io,
                message: format!("Failed to read file: {}", e),
            })?;
        
        let name = path.file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("unknown");
        
        Ok(BinaryAsset::new(name, data))
    }
}
