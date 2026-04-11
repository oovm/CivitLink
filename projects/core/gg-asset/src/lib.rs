#![warn(missing_docs)]
#![allow(clippy::type_complexity)]

//! GG 引擎资源管理模块
//! 提供异步资源加载、并发安全缓存和类型安全的资源句柄功能

use std::{
    any::{Any, TypeId},
    collections::HashMap,
    future::Future,
    marker::PhantomData,
    path::Path,
    pin::Pin,
    sync::{Arc, Mutex, RwLock},
};

use dashmap::DashMap;
use notify::{Config, EventKind, RecommendedWatcher, RecursiveMode, Watcher};

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

impl Default for AssetCache {
    fn default() -> Self {
        Self::new()
    }
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

    /// 根据路径获取资源唯一标识
    ///
    /// 如果路径不存在于缓存中，返回 `None`。
    pub fn get_id(&self, path: &str) -> Option<u64> {
        self.handles.get(path).map(|r| *r.value())
    }

    /// 使用类型擦除的资源数据更新指定路径的缓存
    ///
    /// 如果路径已存在，更新对应的资源并返回其唯一标识。
    /// 如果路径不存在，返回 `None`。
    pub fn update_erased(&self, path: &str, asset: Arc<dyn Any + Send + Sync>) -> Option<u64> {
        if let Some(id) = self.handles.get(path).map(|r| *r.value()) {
            self.assets.insert(id, asset);
            Some(id)
        }
        else {
            None
        }
    }
}

/// 资源变更事件
#[derive(Debug, Clone)]
pub struct AssetChangeEvent {
    /// 变更的资源路径
    pub path: String,
    /// 变更类型
    pub kind: AssetChangeKind,
}

/// 资源变更类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AssetChangeKind {
    /// 文件被修改
    Modified,
    /// 文件被创建
    Created,
    /// 文件被删除
    Deleted,
}

/// 资源文件监视器
///
/// 基于 notify crate 实现异步文件变更检测，
/// 当监视目录中的资源文件发生修改时，通过事件通道通知 AssetServer。
pub struct AssetWatcher {
    /// 内部文件监视器
    watcher: Option<RecommendedWatcher>,
    /// 文件变更事件接收端
    rx: Option<tokio::sync::mpsc::UnboundedReceiver<AssetChangeEvent>>,
    /// 正在监视的目录列表
    watched_dirs: Vec<String>,
}

impl AssetWatcher {
    /// 创建空的资源文件监视器
    pub fn new() -> Self {
        Self { watcher: None, rx: None, watched_dirs: Vec::new() }
    }

    /// 开始监视指定目录
    ///
    /// 使用 notify 的 RecommendedWatcher 递归监视目录中的文件变更，
    /// 变更事件通过内部通道发送，可通过 `poll_changes` 方法获取。
    /// 如果监视器尚未创建，将自动创建。
    /// 如果指定目录已在监视列表中，则跳过。
    pub fn watch(&mut self, path: &str) -> Result<(), AssetError> {
        if self.watched_dirs.iter().any(|d| d == path) {
            return Ok(());
        }

        if self.watcher.is_none() {
            let (tx, rx) = tokio::sync::mpsc::unbounded_channel();

            let watcher = RecommendedWatcher::new(
                move |res: Result<notify::Event, notify::Error>| {
                    if let Ok(event) = res {
                        for file_path in &event.paths {
                            let kind = match event.kind {
                                EventKind::Create(_) => AssetChangeKind::Created,
                                EventKind::Modify(_) => AssetChangeKind::Modified,
                                EventKind::Remove(_) => AssetChangeKind::Deleted,
                                _ => continue,
                            };
                            let path_str = file_path.to_string_lossy().to_string();
                            let _ = tx.send(AssetChangeEvent { path: path_str, kind });
                        }
                    }
                },
                Config::default(),
            )
            .map_err(|e| AssetError::LoadError(format!("Failed to create file watcher: {}", e)))?;

            self.watcher = Some(watcher);
            self.rx = Some(rx);
        }

        if let Some(watcher) = &mut self.watcher {
            watcher
                .watch(Path::new(path), RecursiveMode::Recursive)
                .map_err(|e| AssetError::LoadError(format!("Failed to watch directory '{}': {}", path, e)))?;
        }

        self.watched_dirs.push(path.to_string());
        Ok(())
    }

    /// 停止监视所有目录
    ///
    /// 停止所有已注册的目录监视，并释放文件监视器资源。
    pub fn unwatch(&mut self) {
        if let Some(mut watcher) = self.watcher.take() {
            for dir in &self.watched_dirs {
                let _ = watcher.unwatch(Path::new(dir));
            }
        }
        self.rx = None;
        self.watched_dirs.clear();
    }

    /// 非阻塞地获取待处理的文件变更事件
    ///
    /// 返回自上次调用以来积累的所有变更事件列表。
    /// 如果没有待处理的事件，返回空列表。
    pub fn poll_changes(&mut self) -> Vec<AssetChangeEvent> {
        let mut changes = Vec::new();
        if let Some(rx) = &mut self.rx {
            while let Ok(event) = rx.try_recv() {
                changes.push(event);
            }
        }
        changes
    }
}

impl Default for AssetWatcher {
    fn default() -> Self {
        Self::new()
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
    pub load_states: DashMap<u64, LoadState>,
    /// 资源加载完成回调，按资源类型索引
    on_load_callbacks: HashMap<TypeId, Vec<Box<dyn Fn(u64) + Send + Sync>>>,
    /// 资源重载完成回调，按资源类型索引
    on_reload_callbacks: HashMap<TypeId, Vec<Box<dyn Fn(&str) + Send + Sync>>>,
    /// 资源文件监视器
    watcher: AssetWatcher,
    /// 资源路径到类型的映射
    path_types: DashMap<String, TypeId>,
    /// 待处理的资源重载队列
    pending_reloads: Mutex<Vec<(String, TypeId)>>,
}

impl Default for AssetServer {
    fn default() -> Self {
        Self::new()
    }
}

impl AssetServer {
    /// 创建新的资源服务器
    pub fn new() -> Self {
        Self {
            cache: AssetCache::new(),
            loaders: HashMap::new(),
            load_states: DashMap::new(),
            on_load_callbacks: HashMap::new(),
            on_reload_callbacks: HashMap::new(),
            watcher: AssetWatcher::new(),
            path_types: DashMap::new(),
            pending_reloads: Mutex::new(Vec::new()),
        }
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
    /// 加载过程中会追踪资源状态，
    /// 加载成功后会调用所有已注册的 `on_load` 回调。
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
                self.path_types.insert(path.to_string(), type_id);

                if let Some(callbacks) = self.on_load_callbacks.get(&type_id) {
                    for callback in callbacks {
                        callback(handle.id);
                    }
                }

                Ok(handle)
            }
            Err(e) => {
                self.load_states.insert(id, LoadState::Failed(e.to_string()));
                Err(e)
            }
        }
    }

    /// 注册资源加载完成回调
    ///
    /// 当指定类型 `T` 的资源通过 `load` 方法异步加载成功后，
    /// 将调用所有已注册的回调，传入资源句柄的唯一标识。
    /// 通过 `add_asset` 同步添加的资源不会触发回调。
    pub fn on_load<T: Asset>(&mut self, callback: impl Fn(u64) + Send + Sync + 'static) {
        let type_id = TypeId::of::<T>();
        self.on_load_callbacks.entry(type_id).or_default().push(Box::new(callback));
    }

    /// 添加资源到服务器
    ///
    /// 将资源同步插入内部缓存并返回类型安全的句柄，
    /// 同时将加载状态设置为已加载。
    pub fn add_asset<T: Asset + 'static>(&self, path: &str, asset: T) -> Handle<T> {
        let handle = self.cache.insert(path, asset);
        self.load_states.insert(handle.id, LoadState::Loaded);
        self.path_types.insert(path.to_string(), TypeId::of::<T>());
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

    /// 异步重新加载指定路径的资源
    ///
    /// 使用已注册的加载器重新加载资源并更新缓存。
    /// 如果资源不在缓存中，行为与 `load` 相同。
    /// 重载成功后触发所有已注册的 `on_reload` 回调。
    pub async fn reload<T: Asset>(&self, path: &str) -> Result<Handle<T>, AssetError> {
        if !self.cache.contains(path) {
            return self.load::<T>(path).await;
        }

        let type_id = TypeId::of::<T>();
        let loader = self
            .loaders
            .get(&type_id)
            .ok_or_else(|| AssetError::LoadError(format!("No loader registered for type {:?}", type_id)))?;

        let handle = self
            .cache
            .get_handle::<T>(path)
            .ok_or_else(|| AssetError::NotFound(path.to_string()))?;

        self.load_states.insert(handle.id, LoadState::Loading);

        match loader.load_erased(Path::new(path)).await {
            Ok(asset_box) => {
                let asset = asset_box.downcast::<T>().map_err(|_| AssetError::TypeMismatch)?;
                let handle = self.cache.insert(path, *asset);
                self.load_states.insert(handle.id, LoadState::Loaded);
                self.path_types.insert(path.to_string(), type_id);

                if let Some(callbacks) = self.on_reload_callbacks.get(&type_id) {
                    for callback in callbacks {
                        callback(path);
                    }
                }

                Ok(handle)
            }
            Err(e) => {
                self.load_states.insert(handle.id, LoadState::Failed(e.to_string()));
                Err(e)
            }
        }
    }

    /// 注册资源重载完成回调
    ///
    /// 当指定类型 `T` 的资源通过 `reload` 方法重新加载成功后，
    /// 将调用所有已注册的回调，传入资源路径。
    pub fn on_reload<T: Asset>(&mut self, callback: impl Fn(&str) + Send + Sync + 'static) {
        let type_id = TypeId::of::<T>();
        self.on_reload_callbacks.entry(type_id).or_default().push(Box::new(callback));
    }

    /// 开始监视指定目录的资源文件变更
    ///
    /// 当目录中的文件发生修改时，变更事件将被记录到待处理队列，
    /// 需要调用 `process_watcher_events` 和 `process_pending_reloads` 来处理。
    pub fn watch_directory(&mut self, path: &str) -> Result<(), AssetError> {
        self.watcher.watch(path)
    }

    /// 停止监视所有目录
    pub fn unwatch(&mut self) {
        self.watcher.unwatch();
    }

    /// 处理文件监视器事件
    ///
    /// 非阻塞地轮询文件监视器的变更事件，
    /// 将需要重载的资源路径加入待处理队列。
    /// 对于已删除的资源，直接从缓存中移除。
    /// 需要后续调用 `process_pending_reloads` 异步处理队列中的重载请求。
    pub fn process_watcher_events(&mut self) {
        let changes = self.watcher.poll_changes();
        for change in changes {
            match change.kind {
                AssetChangeKind::Modified | AssetChangeKind::Created => {
                    if let Some(type_id) = self.path_types.get(&change.path).map(|r| *r.value()) {
                        let mut pending = self.pending_reloads.lock().unwrap();
                        pending.push((change.path.clone(), type_id));
                    }
                }
                AssetChangeKind::Deleted => {
                    self.cache.remove(&change.path);
                    self.path_types.remove(&change.path);
                }
            }
        }
    }

    /// 异步处理待重载队列
    ///
    /// 处理由 `process_watcher_events` 产生的待重载资源队列，
    /// 使用类型擦除的方式重新加载每个资源。
    pub async fn process_pending_reloads(&self) {
        let reloads: Vec<(String, TypeId)> = {
            let mut pending = self.pending_reloads.lock().unwrap();
            std::mem::take(&mut *pending)
        };

        for (path, type_id) in reloads {
            let _ = self.reload_erased(&path, type_id).await;
        }
    }

    /// 使用类型擦除的方式异步重新加载指定路径的资源
    async fn reload_erased(&self, path: &str, type_id: TypeId) -> Result<(), AssetError> {
        let loader = self
            .loaders
            .get(&type_id)
            .ok_or_else(|| AssetError::LoadError(format!("No loader registered for type {:?}", type_id)))?;

        let id = self
            .cache
            .get_id(path)
            .ok_or_else(|| AssetError::NotFound(path.to_string()))?;

        self.load_states.insert(id, LoadState::Loading);

        match loader.load_erased(Path::new(path)).await {
            Ok(asset_box) => {
                if self.cache.update_erased(path, Arc::from(asset_box)).is_some() {
                    self.load_states.insert(id, LoadState::Loaded);

                    if let Some(callbacks) = self.on_reload_callbacks.get(&type_id) {
                        for callback in callbacks {
                            callback(path);
                        }
                    }

                    Ok(())
                }
                else {
                    self.load_states.insert(id, LoadState::Failed("Asset path no longer in cache".to_string()));
                    Err(AssetError::NotFound(path.to_string()))
                }
            }
            Err(e) => {
                self.load_states.insert(id, LoadState::Failed(e.to_string()));
                Err(e)
            }
        }
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

/// 音频格式
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AudioFormat {
    /// WAV 格式
    Wav,
    /// OGG 格式
    Ogg,
    /// MP3 格式
    Mp3,
    /// FLAC 格式
    Flac,
}

impl AudioFormat {
    /// 根据文件扩展名获取音频格式
    pub fn from_extension(ext: &str) -> Option<AudioFormat> {
        match ext.to_lowercase().as_str() {
            "wav" => Some(AudioFormat::Wav),
            "ogg" => Some(AudioFormat::Ogg),
            "mp3" => Some(AudioFormat::Mp3),
            "flac" => Some(AudioFormat::Flac),
            _ => None,
        }
    }
}

/// 音频资源
///
/// 封装音频原始数据、格式及基本信息。
pub struct AudioAsset {
    /// 音频原始数据
    data: Vec<u8>,
    /// 音频格式
    format: AudioFormat,
    /// 声道数
    channels: u16,
    /// 采样率
    sample_rate: u32,
}

impl AudioAsset {
    /// 创建新的音频资源
    pub fn new(data: Vec<u8>, format: AudioFormat, channels: u16, sample_rate: u32) -> Self {
        Self { data, format, channels, sample_rate }
    }

    /// 获取音频原始数据
    pub fn data(&self) -> &[u8] {
        &self.data
    }

    /// 获取音频格式
    pub fn format(&self) -> AudioFormat {
        self.format
    }

    /// 获取声道数
    pub fn channels(&self) -> u16 {
        self.channels
    }

    /// 获取采样率
    pub fn sample_rate(&self) -> u32 {
        self.sample_rate
    }
}

impl Asset for AudioAsset {
    fn type_name() -> &'static str
    where
        Self: Sized,
    {
        "AudioAsset"
    }
}

/// 音频资源加载器
///
/// 从文件系统异步加载音频文件为 `AudioAsset`，
/// 根据文件扩展名判断音频格式，声道数和采样率使用默认值。
pub struct AudioLoader;

impl AssetLoader<AudioAsset> for AudioLoader {
    async fn load(&self, path: &Path) -> Result<AudioAsset, AssetError> {
        let data = tokio::fs::read(path)
            .await
            .map_err(|e| AssetError::LoadError(format!("Failed to read audio file: {}", e)))?;

        let extension = path.extension().and_then(|e| e.to_str()).unwrap_or("");
        let format = AudioFormat::from_extension(extension)
            .ok_or_else(|| AssetError::LoadError(format!("Unsupported audio format: {}", extension)))?;

        Ok(AudioAsset::new(data, format, 2, 44100))
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

/// 字体资源
///
/// 封装字体原始数据和解析后的字体对象。
pub struct FontAsset {
    /// 字体原始数据
    data: Vec<u8>,
    /// 解析后的字体对象
    font: ab_glyph::FontArc,
}

impl FontAsset {
    /// 创建新的字体资源
    ///
    /// 如果字体数据解析失败，返回 `AssetError::LoadError`。
    pub fn new(data: Vec<u8>) -> Result<Self, AssetError> {
        let font = ab_glyph::FontArc::try_from_vec(data.clone())
            .map_err(|e| AssetError::LoadError(format!("Failed to parse font: {}", e)))?;
        Ok(Self { data, font })
    }

    /// 获取字体原始数据
    pub fn data(&self) -> &[u8] {
        &self.data
    }

    /// 获取字体对象的引用
    pub fn font(&self) -> &ab_glyph::FontArc {
        &self.font
    }
}

impl Asset for FontAsset {
    fn type_name() -> &'static str
    where
        Self: Sized,
    {
        "FontAsset"
    }
}

/// 字体资源加载器
///
/// 从文件系统异步加载 TTF/OTF 字体文件为 `FontAsset`，
/// 使用 `ab_glyph` 解析字体数据。
pub struct FontLoader;

impl AssetLoader<FontAsset> for FontLoader {
    async fn load(&self, path: &Path) -> Result<FontAsset, AssetError> {
        let data = tokio::fs::read(path)
            .await
            .map_err(|e| AssetError::LoadError(format!("Failed to read font file: {}", e)))?;

        FontAsset::new(data)
    }
}

/// 预导入模块
pub mod prelude {
    /// 重新导出 gg-asset 核心类型
    pub use crate::{
        Asset, AssetCache, AssetChangeKind, AssetChangeEvent, AssetError, AssetLoader, AssetServer,
        AssetWatcher, AudioAsset, AudioFormat, AudioLoader, BinaryAsset, BinaryLoader, FontAsset,
        FontLoader, Handle, ImageAsset, ImageLoader, LoadState, TextAsset, TextLoader,
    };
}


