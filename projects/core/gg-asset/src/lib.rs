#![warn(missing_docs)]

//! GG 引擎资源管理模块
//! 提供异步资源加载、并发安全缓存和类型安全的资源句柄功能

use std::{
    any::{Any, TypeId},
    collections::HashMap,
    future::Future,
    marker::PhantomData,
    path::Path,
    pin::Pin,
    sync::{Arc, RwLock},
};

use dashmap::DashMap;

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

/// 资源加载状态
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LoadState {
    /// 未加载
    NotLoaded,
    /// 加载中
    Loading,
    /// 已加载
    Loaded,
    /// 加载失败，包含错误信息
    Failed(String),
}

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
        Self { id, path, _marker: PhantomData }
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
        Self { id: self.id, path: self.path.clone(), _marker: PhantomData }
    }
}

impl<T> std::fmt::Debug for Handle<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Handle").field("id", &self.id).field("path", &self.path.as_ref()).finish()
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
/// 使用并发安全的数据结构存储类型擦除的资源，
/// 并通过路径和唯一标识进行索引。
/// 支持类型安全的资源插入和查询，以及多线程并发访问。
pub struct AssetCache {
    /// 资源存储，按唯一标识索引
    assets: DashMap<u64, Arc<dyn Any + Send + Sync>>,
    /// 路径到唯一标识的映射
    handles: DashMap<Arc<str>, u64>,
    /// 下一个资源唯一标识，使用读写锁保证并发安全
    next_id: RwLock<u64>,
}

impl AssetCache {
    /// 创建空的资源缓存
    pub fn new() -> Self {
        Self { assets: DashMap::new(), handles: DashMap::new(), next_id: RwLock::new(0) }
    }

    /// 分配新的资源唯一标识
    pub fn allocate_id(&self) -> u64 {
        let mut next_id = self.next_id.write().unwrap();
        let id = *next_id;
        *next_id += 1;
        id
    }

    /// 插入资源并返回类型安全的句柄
    ///
    /// 如果路径已存在，则更新对应的资源并返回相同标识的句柄。
    pub fn insert<T: Send + Sync + 'static>(&self, path: &str, asset: T) -> Handle<T> {
        let path_arc: Arc<str> = Arc::from(path);

        if let Some(id) = self.handles.get(path).map(|r| *r.value()) {
            self.assets.insert(id, Arc::new(asset));
            return Handle::new(id, path_arc);
        }

        let id = self.allocate_id();
        self.assets.insert(id, Arc::new(asset));
        self.handles.insert(path_arc.clone(), id);

        Handle::new(id, path_arc)
    }

    /// 使用预分配的标识插入资源并返回类型安全的句柄
    ///
    /// 如果路径已存在，则更新对应的资源并返回相同标识的句柄，
    /// 预分配的标识将被忽略。
    pub fn insert_with_id<T: Send + Sync + 'static>(&self, id: u64, path: &str, asset: T) -> Handle<T> {
        let path_arc: Arc<str> = Arc::from(path);

        if let Some(existing_id) = self.handles.get(path).map(|r| *r.value()) {
            self.assets.insert(existing_id, Arc::new(asset));
            return Handle::new(existing_id, path_arc);
        }

        self.assets.insert(id, Arc::new(asset));
        self.handles.insert(path_arc.clone(), id);

        Handle::new(id, path_arc)
    }

    /// 根据句柄获取资源的共享引用
    ///
    /// 如果句柄对应的资源不存在或类型不匹配，返回 `None`。
    pub fn get<T: Send + Sync + 'static>(&self, handle: &Handle<T>) -> Option<Arc<T>> {
        self.assets.get(&handle.id).and_then(|ref_val| ref_val.value().clone().downcast::<T>().ok())
    }

    /// 根据路径获取类型安全的句柄
    ///
    /// 如果路径不存在于缓存中，返回 `None`。
    pub fn get_handle<T: Send + Sync + 'static>(&self, path: &str) -> Option<Handle<T>> {
        self.handles.get(path).map(|r| Handle::new(*r.value(), r.key().clone()))
    }

    /// 检查指定路径的资源是否存在于缓存中
    pub fn contains(&self, path: &str) -> bool {
        self.handles.contains_key(path)
    }

    /// 根据路径移除资源
    ///
    /// 如果路径存在则移除并返回 `true`，否则返回 `false`。
    pub fn remove(&self, path: &str) -> bool {
        if let Some((_, id)) = self.handles.remove(path) {
            self.assets.remove(&id);
            true
        }
        else {
            false
        }
    }

    /// 清空所有缓存资源
    pub fn clear(&self) {
        self.assets.clear();
        self.handles.clear();
    }
}

/// 类型擦除的资源加载器 trait
///
/// 用于在 `AssetServer` 中存储不同类型的资源加载器，
/// 隐藏具体的资源类型信息。
trait ErasedAssetLoader: Send + Sync {
    /// 以类型擦除的方式异步加载指定路径的资源
    fn load_erased<'a>(
        &'a self,
        path: &'a Path,
    ) -> Pin<Box<dyn Future<Output = Result<Box<dyn Any + Send + Sync>, AssetError>> + Send + 'a>>;
}

/// 类型化资源加载器包装
///
/// 将具体的 `AssetLoader` 实现包装为类型擦除的 `ErasedAssetLoader`，
/// 通过泛型参数保留资源类型信息。
struct TypedAssetLoader<T: Asset, L: AssetLoader<T>> {
    /// 具体的资源加载器
    loader: L,
    /// 资源类型标记
    _marker: PhantomData<T>,
}

impl<T: Asset, L: AssetLoader<T> + Send + Sync> ErasedAssetLoader for TypedAssetLoader<T, L> {
    fn load_erased<'a>(
        &'a self,
        path: &'a Path,
    ) -> Pin<Box<dyn Future<Output = Result<Box<dyn Any + Send + Sync>, AssetError>> + Send + 'a>> {
        Box::pin(async move {
            let asset = self.loader.load(path).await?;
            Ok(Box::new(asset) as Box<dyn Any + Send + Sync>)
        })
    }
}

/// 资源服务器
///
/// 提供资源管理的上层接口，内部封装并发安全的资源缓存，
/// 支持异步资源加载和加载状态追踪。
pub struct AssetServer {
    /// 内部资源缓存
    cache: AssetCache,
    /// 已注册的资源加载器，按资源类型索引
    loaders: HashMap<TypeId, Box<dyn ErasedAssetLoader>>,
    /// 资源加载状态，按资源唯一标识索引
    load_states: DashMap<u64, LoadState>,
}

impl AssetServer {
    /// 创建新的资源服务器
    pub fn new() -> Self {
        Self { cache: AssetCache::new(), loaders: HashMap::new(), load_states: DashMap::new() }
    }

    /// 注册资源加载器
    ///
    /// 为指定资源类型 `T` 注册加载器 `L`，
    /// 后续可通过 `load` 方法异步加载该类型的资源。
    pub fn register_loader<T: Asset, L: AssetLoader<T> + Send + Sync + 'static>(&mut self, loader: L) {
        self.loaders.insert(TypeId::of::<T>(), Box::new(TypedAssetLoader { loader, _marker: PhantomData }));
    }

    /// 异步加载指定路径的资源
    ///
    /// 如果资源已在缓存中，直接返回现有句柄。
    /// 否则使用已注册的加载器异步加载资源，
    /// 加载过程中会追踪资源状态。
    pub async fn load<T: Asset>(&self, path: &str) -> Result<Handle<T>, AssetError> {
        if let Some(handle) = self.cache.get_handle::<T>(path) {
            return Ok(handle);
        }

        let type_id = TypeId::of::<T>();
        let loader = self
            .loaders
            .get(&type_id)
            .ok_or_else(|| AssetError::LoadError(format!("No loader registered for type {:?}", type_id)))?;

        let id = self.cache.allocate_id();
        self.load_states.insert(id, LoadState::Loading);

        match loader.load_erased(Path::new(path)).await {
            Ok(asset_box) => {
                let asset = asset_box.downcast::<T>().map_err(|_| AssetError::TypeMismatch)?;
                let handle = self.cache.insert_with_id(id, path, *asset);
                self.load_states.insert(id, LoadState::Loaded);
                Ok(handle)
            }
            Err(e) => {
                self.load_states.insert(id, LoadState::Failed(e.to_string()));
                Err(e)
            }
        }
    }

    /// 添加资源到服务器
    ///
    /// 将资源同步插入内部缓存并返回类型安全的句柄，
    /// 同时将加载状态设置为已加载。
    pub fn add_asset<T: Asset + 'static>(&self, path: &str, asset: T) -> Handle<T> {
        let handle = self.cache.insert(path, asset);
        self.load_states.insert(handle.id, LoadState::Loaded);
        handle
    }

    /// 查询资源的加载状态
    ///
    /// 根据句柄查询对应资源的当前加载状态，
    /// 如果资源未被追踪则返回未加载状态。
    pub fn load_state<T: Send + Sync + 'static>(&self, handle: &Handle<T>) -> LoadState {
        self.load_states.get(&handle.id).map(|s| s.value().clone()).unwrap_or(LoadState::NotLoaded)
    }

    /// 获取内部资源缓存的引用
    pub fn cache(&self) -> &AssetCache {
        &self.cache
    }

    /// 获取内部资源缓存的可变引用
    ///
    /// 由于内部缓存使用 `DashMap` 实现并发安全，
    /// 通过共享引用即可进行大部分修改操作，
    /// 此方法用于需要独占访问的场景。
    pub fn cache_mut(&mut self) -> &mut AssetCache {
        &mut self.cache
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
    fn load(&self, path: &Path) -> impl std::future::Future<Output = Result<T, AssetError>> + Send;
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
        Self { content, name: name.to_string() }
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
        let content = tokio::fs::read_to_string(path)
            .await
            .map_err(|e| AssetError::LoadError(format!("Failed to read text file: {}", e)))?;
        let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("unknown");
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
        Self { data, name: name.to_string() }
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
        let data =
            tokio::fs::read(path).await.map_err(|e| AssetError::LoadError(format!("Failed to read binary file: {}", e)))?;
        let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("unknown");
        Ok(BinaryAsset::new(name, data))
    }
}

/// 图像资源
///
/// 封装图像的 RGBA 像素数据及其尺寸信息。
pub struct ImageAsset {
    /// 图像宽度（像素）
    width: u32,
    /// 图像高度（像素）
    height: u32,
    /// RGBA 像素数据
    data: Vec<u8>,
}

impl ImageAsset {
    /// 创建新的图像资源
    pub fn new(width: u32, height: u32, data: Vec<u8>) -> Self {
        Self { width, height, data }
    }

    /// 获取图像宽度
    pub fn width(&self) -> u32 {
        self.width
    }

    /// 获取图像高度
    pub fn height(&self) -> u32 {
        self.height
    }

    /// 获取 RGBA 像素数据
    pub fn data(&self) -> &[u8] {
        &self.data
    }
}

impl Asset for ImageAsset {
    fn type_name() -> &'static str
    where
        Self: Sized,
    {
        "ImageAsset"
    }
}

/// 图像资源加载器
///
/// 从文件系统异步加载 PNG/JPG 图像文件为 `ImageAsset`。
pub struct ImageLoader;

impl AssetLoader<ImageAsset> for ImageLoader {
    async fn load(&self, path: &Path) -> Result<ImageAsset, AssetError> {
        let data = tokio::fs::read(path)
            .await
            .map_err(|e| AssetError::LoadError(format!("Failed to read image file: {}", e)))?;

        let img = image::load_from_memory(&data)
            .map_err(|e| AssetError::LoadError(format!("Failed to decode image: {}", e)))?;

        let rgba = img.to_rgba8();
        let dimensions = rgba.dimensions();

        Ok(ImageAsset::new(dimensions.0, dimensions.1, rgba.into_raw()))
    }
}

/// 预导入模块
pub mod prelude {
    /// 重新导出 gg-asset 核心类型
    pub use crate::{
        Asset, AssetCache, AssetError, AssetLoader, AssetServer, BinaryAsset, BinaryLoader, Handle, ImageAsset,
        ImageLoader, LoadState, TextAsset, TextLoader,
    };
}
