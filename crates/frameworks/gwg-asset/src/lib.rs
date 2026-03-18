//! GWG Engine 资源管理系统
//!
//! 提供异步资源加载、缓存和 Handle 系统。

use std::fmt;
use std::path::Path;
use std::sync::Arc;

use anyhow::Result;
use dashmap::DashMap;
use tokio::sync::RwLock;

/// 资源加载错误
#[derive(thiserror::Error, Debug)]
pub enum AssetError {
    /// 资源未找到
    #[error("Asset not found: {0}")]
    NotFound(String),
    /// 资源加载失败
    #[error("Failed to load asset: {0}")]
    LoadError(String),
    /// 资源类型不匹配
    #[error("Asset type mismatch")]
    TypeMismatch,
}

/// 资源句柄，用于安全地引用资源
#[derive(Clone, PartialEq, Eq, Hash)]
pub struct Handle<T> {
    id: u64,
    path: Arc<str>,
    _marker: std::marker::PhantomData<T>,
}

impl<T> Handle<T> {
    /// 创建一个新的资源句柄
    pub fn new(id: u64, path: impl Into<Arc<str>>) -> Self {
        Self {
            id,
            path: path.into(),
            _marker: std::marker::PhantomData,
        }
    }

    /// 获取资源 ID
    pub fn id(&self) -> u64 {
        self.id
    }

    /// 获取资源路径
    pub fn path(&self) -> &str {
        &self.path
    }
}

impl<T> fmt::Debug for Handle<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Handle")
            .field("id", &self.id)
            .field("path", &self.path)
            .finish()
    }
}

/// 资源 trait，所有可加载的资源都需要实现此 trait
pub trait Asset: Send + Sync + 'static {
    /// 资源的类型名称
    fn type_name() -> &'static str;
}

/// 资源加载器 trait，负责加载特定类型的资源
pub trait AssetLoader<T: Asset>: Send + Sync + 'static {
    /// 从路径加载资源
    async fn load(&self, path: &Path) -> Result<T, AssetError>;
}

/// 资源缓存，存储已加载的资源
pub struct AssetCache {
    assets: DashMap<u64, Arc<dyn std::any::Any + Send + Sync>>,
    handles: DashMap<Arc<str>, u64>,
    next_id: RwLock<u64>,
}

impl AssetCache {
    /// 创建一个新的空资源缓存
    pub fn new() -> Self {
        Self {
            assets: DashMap::new(),
            handles: DashMap::new(),
            next_id: RwLock::new(1),
        }
    }

    /// 插入资源到缓存中
    pub async fn insert<T: Asset>(&self, path: impl Into<Arc<str>>, asset: T) -> Handle<T> {
        let path = path.into();
        let mut next_id = self.next_id.write().await;
        let id = *next_id;
        *next_id += 1;
        drop(next_id);

        self.assets.insert(id, Arc::new(asset));
        self.handles.insert(path.clone(), id);

        Handle::new(id, path)
    }

    /// 根据句柄获取资源
    pub fn get<T: Asset>(&self, handle: &Handle<T>) -> Option<Arc<T>> {
        self.assets.get(&handle.id).and_then(|asset| {
            asset.downcast_ref::<T>().map(|_| {
                let arc: Arc<dyn std::any::Any + Send + Sync> = Arc::clone(&asset);
                arc.downcast::<T>().unwrap()
            })
        })
    }

    /// 根据路径获取资源句柄
    pub fn get_handle<T: Asset>(&self, path: &str) -> Option<Handle<T>> {
        self.handles.get(path).map(|id| Handle::new(*id, path))
    }

    /// 检查资源是否已加载
    pub fn contains(&self, path: &str) -> bool {
        self.handles.contains_key(path)
    }

    /// 从缓存中移除资源
    pub fn remove(&self, path: &str) {
        if let Some((_, id)) = self.handles.remove(path) {
            self.assets.remove(&id);
        }
    }

    /// 清空缓存
    pub fn clear(&self) {
        self.assets.clear();
        self.handles.clear();
    }
}

impl Default for AssetCache {
    fn default() -> Self {
        Self::new()
    }
}

/// 资源管理器，负责资源的加载和管理
pub struct AssetServer {
    cache: AssetCache,
}

impl AssetServer {
    /// 创建一个新的资源服务器
    pub fn new() -> Self {
        Self {
            cache: AssetCache::new(),
        }
    }

    /// 直接将资源添加到缓存中
    pub async fn add_asset<T: Asset>(&self, path: impl Into<Arc<str>>, asset: T) -> Handle<T> {
        self.cache.insert(path, asset).await
    }

    /// 获取资源缓存的引用
    pub fn cache(&self) -> &AssetCache {
        &self.cache
    }
}

impl Default for AssetServer {
    fn default() -> Self {
        Self::new()
    }
}

pub mod prelude {
    //! 资源管理系统的预导入模块

    pub use super::{Asset, AssetCache, AssetError, AssetLoader, AssetServer, Handle};
}
