//! 资产注册表模块
//! 提供 GUID 和路径双向索引的资产注册表，支持资产的注册、查询和移除

use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};

use crate::types::{AssetEntry, Guid};

/// 资产注册表，维护 GUID 和路径的双向索引
#[derive(Debug, Serialize, Deserialize)]
pub struct AssetRegistry {
    /// GUID 到资产条目的映射
    assets: HashMap<Guid, AssetEntry>,
    /// 路径到 GUID 的映射
    path_to_guid: HashMap<PathBuf, Guid>,
}

impl AssetRegistry {
    /// 创建一个空的资产注册表
    pub fn new() -> Self {
        Self { assets: HashMap::new(), path_to_guid: HashMap::new() }
    }

    /// 注册一个资产条目，如果 GUID 或路径已存在则替换
    pub fn register(&mut self, entry: AssetEntry) {
        if let Some(old) = self.assets.get(&entry.guid) {
            self.path_to_guid.remove(&old.path);
        }
        self.path_to_guid.insert(entry.path.clone(), entry.guid.clone());
        self.assets.insert(entry.guid.clone(), entry);
    }

    /// 根据 GUID 获取资产条目的引用
    pub fn get_by_guid(&self, guid: &Guid) -> Option<&AssetEntry> {
        self.assets.get(guid)
    }

    /// 根据路径获取资产条目的引用
    pub fn get_by_path(&self, path: &Path) -> Option<&AssetEntry> {
        self.path_to_guid.get(path).and_then(|guid| self.assets.get(guid))
    }

    /// 根据 GUID 移除并返回资产条目
    pub fn remove(&mut self, guid: &Guid) -> Option<AssetEntry> {
        let entry = self.assets.remove(guid);
        if let Some(ref e) = entry {
            self.path_to_guid.remove(&e.path);
        }
        entry
    }

    /// 检查注册表中是否包含指定 GUID 的资产
    pub fn contains_guid(&self, guid: &Guid) -> bool {
        self.assets.contains_key(guid)
    }

    /// 获取注册表中的资产数量
    pub fn len(&self) -> usize {
        self.assets.len()
    }

    /// 检查注册表是否为空
    pub fn is_empty(&self) -> bool {
        self.assets.is_empty()
    }

    /// 获取所有资产条目的迭代器
    pub fn entries(&self) -> impl Iterator<Item = &AssetEntry> {
        self.assets.values()
    }

    /// 更新指定 GUID 资产的哈希值，返回是否更新成功
    pub fn update_hash(&mut self, guid: &Guid, hash: &str) -> bool {
        if let Some(entry) = self.assets.get_mut(guid) {
            entry.hash = hash.to_string();
            true
        }
        else {
            false
        }
    }

    /// 克隆注册表供管线内部使用
    pub fn clone_for_pipeline(&self) -> Self {
        Self { assets: self.assets.clone(), path_to_guid: self.path_to_guid.clone() }
    }
}

impl Default for AssetRegistry {
    fn default() -> Self {
        Self::new()
    }
}
