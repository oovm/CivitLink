//! GG 引擎资源管理系统
//! 
//! 提供统一的资源加载、管理和热更新功能，支持编辑器UI和游戏UI的资源分离

#![warn(missing_docs)]

use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};

use gg_error::{GError, GErrorKind, GResult};

/// 资源类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AssetType {
    /// 纹理
    Texture,
    /// 字体
    Font,
    /// 材质
    Material,
    /// 模型
    Model,
    /// 音频
    Audio,
    /// 脚本
    Script,
    /// UI资源
    Ui,
    /// 编辑器UI资源
    EditorUi,
    /// 其他资源
    Other,
}

/// 资源状态
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AssetState {
    /// 未加载
    Unloaded,
    /// 加载中
    Loading,
    /// 已加载
    Loaded,
    /// 加载失败
    Failed,
}

/// 资源句柄
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Handle<T: Asset> {
    /// 资源ID
    id: u64,
    /// 类型标记
    _marker: std::marker::PhantomData<T>,
}

impl<T: Asset> Handle<T> {
    /// 创建新的资源句柄
    pub fn new(id: u64) -> Self {
        Self {
            id,
            _marker: std::marker::PhantomData,
        }
    }

    /// 获取资源ID
    pub fn id(&self) -> u64 {
        self.id
    }
}

/// 资源特质
pub trait Asset: Send + Sync + 'static {
    /// 获取资源类型
    fn asset_type() -> AssetType;

    /// 加载资源
    fn load(path: &Path) -> GResult<Self>;

    /// 卸载资源
    fn unload(&mut self);

    /// 获取资源大小
    fn size(&self) -> usize;
}

/// 资源元数据
#[derive(Debug, Clone)]
pub struct AssetMeta {
    /// 资源路径
    pub path: PathBuf,
    /// 资源类型
    pub asset_type: AssetType,
    /// 资源大小
    pub size: usize,
    /// 最后修改时间
    pub last_modified: Instant,
    /// 依赖资源
    pub dependencies: HashSet<u64>,
}

/// 资源项
struct AssetItem {
    /// 资源数据
    asset: Box<dyn Any + Send + Sync>,
    /// 资源元数据
    meta: AssetMeta,
    /// 资源状态
    state: AssetState,
    /// 引用计数
    ref_count: usize,
    /// 最后访问时间
    last_accessed: Instant,
}

/// 资源服务器
pub struct AssetServer {
    /// 资源存储
    assets: HashMap<u64, AssetItem>,
    /// 路径到资源ID的映射
    path_to_id: HashMap<PathBuf, u64>,
    /// 资源ID分配器
    next_id: u64,
    /// 编辑器UI资源ID集合
    editor_ui_assets: HashSet<u64>,
    /// 游戏UI资源ID集合
    game_ui_assets: HashSet<u64>,
    /// 资源加载线程池
    // loader_threads: ThreadPool,
    /// 热更新监控
    hot_reload_enabled: bool,
    /// 热更新检查间隔
    hot_reload_interval: Duration,
    /// 上次热更新检查时间
    last_hot_reload_check: Instant,
}

impl AssetServer {
    /// 创建新的资源服务器
    pub fn new() -> Self {
        Self {
            assets: HashMap::new(),
            path_to_id: HashMap::new(),
            next_id: 1,
            editor_ui_assets: HashSet::new(),
            game_ui_assets: HashSet::new(),
            hot_reload_enabled: false,
            hot_reload_interval: Duration::from_secs(1),
            last_hot_reload_check: Instant::now(),
        }
    }

    /// 加载资源
    pub fn load<T: Asset>(&mut self, path: &Path, is_editor_ui: bool) -> GResult<Handle<T>> {
        // 检查资源是否已加载
        if let Some(&id) = self.path_to_id.get(path) {
            // 增加引用计数
            if let Some(item) = self.assets.get_mut(&id) {
                item.ref_count += 1;
                item.last_accessed = Instant::now();
                return Ok(Handle::new(id));
            }
        }

        // 分配新的资源ID
        let id = self.next_id;
        self.next_id += 1;

        // 加载资源
        let asset = T::load(path)?;
        let size = asset.size();

        // 创建资源项
        let item = AssetItem {
            asset: Box::new(asset),
            meta: AssetMeta {
                path: path.to_path_buf(),
                asset_type: T::asset_type(),
                size,
                last_modified: Instant::now(),
                dependencies: HashSet::new(),
            },
            state: AssetState::Loaded,
            ref_count: 1,
            last_accessed: Instant::now(),
        };

        // 存储资源
        self.assets.insert(id, item);
        self.path_to_id.insert(path.to_path_buf(), id);

        // 分类资源
        if is_editor_ui {
            self.editor_ui_assets.insert(id);
        } else {
            self.game_ui_assets.insert(id);
        }

        Ok(Handle::new(id))
    }

    /// 获取资源
    pub fn get<T: Asset>(&self, handle: &Handle<T>) -> Option<&T> {
        self.assets.get(&handle.id()).and_then(|item| {
            // 更新最后访问时间
            // 注意：这里不能修改，因为是不可变引用
            item.asset.downcast_ref::<T>()
        })
    }

    /// 获取资源可变引用
    pub fn get_mut<T: Asset>(&mut self, handle: &Handle<T>) -> Option<&mut T> {
        self.assets.get_mut(&handle.id()).and_then(|item| {
            item.last_accessed = Instant::now();
            item.asset.downcast_mut::<T>()
        })
    }

    /// 释放资源
    pub fn release<T: Asset>(&mut self, handle: Handle<T>) {
        if let Some(item) = self.assets.get_mut(&handle.id()) {
            item.ref_count -= 1;
            if item.ref_count <= 0 {
                // 卸载资源
                if let Some(asset) = item.asset.downcast_mut::<T>() {
                    asset.unload();
                }
                // 从存储中移除
                self.assets.remove(&handle.id());
                self.path_to_id.remove(&item.meta.path);
                self.editor_ui_assets.remove(&handle.id());
                self.game_ui_assets.remove(&handle.id());
            }
        }
    }

    /// 启用热更新
    pub fn enable_hot_reload(&mut self, enabled: bool) {
        self.hot_reload_enabled = enabled;
    }

    /// 检查热更新
    pub fn check_hot_reload(&mut self) -> GResult<Vec<u64>> {
        if !self.hot_reload_enabled {
            return Ok(Vec::new());
        }

        let now = Instant::now();
        if now.duration_since(self.last_hot_reload_check) < self.hot_reload_interval {
            return Ok(Vec::new());
        }

        self.last_hot_reload_check = now;
        let mut reloaded_assets = Vec::new();

        // 检查所有资源
        for (id, item) in &mut self.assets {
            let path = &item.meta.path;
            if let Ok(metadata) = std::fs::metadata(path) {
                if let Ok(modified) = metadata.modified() {
                    let modified