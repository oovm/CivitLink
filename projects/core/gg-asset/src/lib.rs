#![warn(missing_docs)]

//! GG 引擎资源管理模块
//! 提供资源加载、缓存和类型安全的资源句柄功能

use std::any::Any;
use std::collections::HashMap;
use std::marker::PhantomData;
use std::path::Path;
use std::sync::Arc;

/// 资源错误类型
#[derive(Debug)]
pub enum AssetError {
    /// 资源未找到，包含资源路径
    NotFound(String),
    /// 资源加载失败，包含错误信息
    LoadError(String),
    /// 资源类型不匹配
    TypeMismatch,
}

impl std::fmt::Display for AssetError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AssetError::NotFound(path) => write!(f, "Asset not found: {}", path),
            AssetError::LoadError(msg) => write!(f, "Asset load error: {}", msg),
            AssetError::TypeMismatch => write!(f, "Asset type mismatch"),
        }
    }
}

impl std::error::Error for AssetError {}

/// 类型安全的资源句柄
///
/// 通过唯一标识和路径引用缓存中的资源，
/// 泛型参数 `T` 在编译时保证句柄与资源类型的对应关系。
pub struct Handle<T> {
    /// 资源唯一标识
    pub id: u64,
    /// 资源路径
    pub path: Arc<str>,
    /// 类型标记，用于编译时类型安全
    pub _marker: PhantomData<T>,
}

impl<T> Handle<T> {
    /// 创建新的资源句柄
    pub fn new(id: u64, path: Arc<str>) -> Self {
        Self {
            id,
            path,
            _marker: PhantomData,
        }
    }

    /// 获取资源唯一标识
    pub fn id(&self) -> u64 {
        self.id
    }

    /// 获取资源路径
    pub fn path(&self) -> &Arc<str> {
        &self.path
    }
}

impl<T> Clone for Handle<T> {
    fn clone(&self) -> Self {
        Self {
            id: self.id,
            path: self.path.clone(),
            _marker: PhantomData,
        }
    }
}

impl<T> std::fmt::Debug for Handle<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Handle")
            .field("id", &self.id)
            .field("path", &self.path.as_ref())
            .finish()
    }
}

impl<T> PartialEq for Handle<T> {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl<T> Eq for Handle<T> {}

impl<T> std::hash::Hash for Handle<T> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

/// 资源缓存
///
/// 存储类型擦除的资源，并通过路径和唯一标识进行索引。
/// 支持类型安全的资源插入和查询。
pub struct AssetCache {
    /// 资源存储，按唯一标识索引
    assets: HashMap<u64, Arc<dyn Any + Send + Sync>>,
    /// 路径到唯一标识的映射
    handles: HashMap<Arc<str>, u64>,
    /// 下一个资源唯一标识
    next_id: u64,
}

impl AssetCache {
    /// 创建空的资源缓存
    pub fn new() -> Self {
        Self {
            assets: HashMap::new(),
            handles: HashMap::new(),
            next_id: 0,
        }
    }

    /// 插入资源并返回类型安全的句柄
    ///
    /// 如果路径已存在，则更新对应的资源并返回相同标识的句柄。
    pub fn insert<T: Send + Sync + 'static>(&mut self, path: &str, asset: T) -> Handle<T> {
        let path_arc: Arc<str> = Arc::from(path);

        if let Some(&id) = self.handles.get(path) {
            self.assets.insert(id, Arc::new(asset));
            return Handle::new(id, path_arc);
        }

        let id = self.next_id;
        self.next_id += 1;

        self.assets.insert(id, Arc::new(asset));
        self.handles.insert(path_arc.clone(), id);

        Handle::new(id, path_arc)
    }

    /// 根据句柄获取资源的共享引用
    ///
    /// 如果句柄对应的资源不存在或类型不匹配，返回 `None`。
    pub fn get<T: Send + Sync + 'static>(&self, handle: &Handle<T>) -> Option<Arc<T>> {
        self.assets
            .get(&handle.id)
            .and_then(|asset| asset.clone().downcast::<T>().ok())
    }

    /// 根据路径获取类型安全的句柄
    ///
    /// 如果路径不存在于缓存中，返回 `None`。
    pub fn get_handle<T: Send + Sync + 'static>(&self, path: &str) -> Option<Handle<T>> {
        self.handles
            .get_key_value(path)
            .map(|(path_arc, &id)| Handle::new(id, path_arc.clone()))
    }

    /// 检查指定路径的资源是否存在于缓存中
    pub fn contains(&self, path: &str) -> bool {
        self.handles.contains_key(path)
    }

    /// 根据路径移除资源
    ///
    /// 如果路径存在则移除并返回 `true`，否则返回 `false`。
    pub fn remove(&mut self, path: &str) -> bool {
        if let Some((_, id)) = self.handles.remove_entry(path) {
            self.assets.remove(&id);
            true
        } else {
            false
        }
    }

    /// 清空所有缓存资源
    pub fn clear(&mut self) {
        self.assets.clear();
        self.handles.clear();
    }
}

/// 资源服务器
///
/// 提供资源管理的上层接口，内部封装资源缓存。
pub struct AssetServer {
    /// 内部资源缓存
    cache: AssetCache,
}

impl AssetServer {
    /// 创建新的资源服务器
    pub fn new() -> Self {
        Self {
            cache: AssetCache::new(),
        }
    }

    /// 添加资源到服务器
    ///
    /// 将资源插入内部缓存并返回类型安全的句柄。
    pub fn add_asset<T: Asset + 'static>(&mut self, path: &str, asset: T) -> Handle<T> {
        self.cache.insert(path, asset)
    }

    /// 获取内部资源缓存的引用
    pub fn cache(&self) -> &AssetCache {
        &self.cache
    }
}

/// 资源 trait
///
/// 所有资源类型必须实现此 trait，以支持类型擦除的存储和查询。
pub trait Asset: Send + Sync + 'static {
    /// 获取资源类型的静态名称
    fn type_name() -> &'static str
    where
        Self: Sized;
}

/// 资源加载器 trait
///
/// 定义异步加载资源的接口，由具体的资源加载器实现。
pub trait AssetLoader<T: Asset> {
    /// 异步加载指定路径的资源
    async fn load(&self, path: &Path) -> Result<T, AssetError>;
}

/// 文本资源
///
/// 封装文本内容及其名称。
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

    /// 获取资源名称
    pub fn name(&self) -> &str {
        &self.name
    }
}

impl Asset for TextAsset {
    fn type_name() -> &'static str
    where
        Self: Sized,
    {
        "TextAsset"
    }
}

/// 文本资源加载器
///
/// 从文件系统异步加载文本文件为 `TextAsset`。
pub struct TextLoader;

impl AssetLoader<TextAsset> for TextLoader {
    async fn load(&self, path: &Path) -> Result<TextAsset, AssetError> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| AssetError::LoadError(format!("Failed to read text file: {}", e)))?;
        let name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown");
        Ok(TextAsset::new(name, content))
    }
}

/// 二进制资源
///
/// 封装二进制数据及其名称。
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

    /// 获取资源名称
    pub fn name(&self) -> &str {
        &self.name
    }
}

impl Asset for BinaryAsset {
    fn type_name() -> &'static str
    where
        Self: Sized,
    {
        "BinaryAsset"
    }
}

/// 二进制资源加载器
///
/// 从文件系统异步加载二进制文件为 `BinaryAsset`。
pub struct BinaryLoader;

impl AssetLoader<BinaryAsset> for BinaryLoader {
    async fn load(&self, path: &Path) -> Result<BinaryAsset, AssetError> {
        let data = std::fs::read(path)
            .map_err(|e| AssetError::LoadError(format!("Failed to read binary file: {}", e)))?;
        let name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown");
        Ok(BinaryAsset::new(name, data))
    }
}
