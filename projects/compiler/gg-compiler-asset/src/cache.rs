//! 资产缓存模块
//! 提供编译产物的内存缓存，支持基于内容哈希的增量有效性验证、
//! LRU 淘汰策略和磁盘持久化

use std::{collections::HashMap, path::Path};

use serde::{Deserialize, Serialize};

use crate::{dependency_graph::DependencyGraph, error::AssetPipelineError, types::Guid};

/// 缓存的编译产物条目
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CachedArtifact {
    /// 产物的全局唯一标识符
    pub guid: Guid,
    /// 产物的二进制数据
    pub data: Vec<u8>,
    /// 产物内容的 SHA1 哈希值
    pub content_hash: String,
    /// 缓存创建时间戳（UNIX 秒）
    pub timestamp: u64,
    /// 最近访问时间戳（UNIX 秒），用于 LRU 淘汰
    pub last_accessed: u64,
}

/// 资产缓存，存储已编译的产物数据，支持 LRU 淘汰和磁盘持久化
#[derive(Debug, Serialize, Deserialize)]
pub struct AssetCache {
    /// GUID 到缓存条目的映射
    entries: HashMap<Guid, CachedArtifact>,
    /// 最大缓存条目数，超过时按 LRU 策略淘汰
    max_capacity: Option<usize>,
}

impl AssetCache {
    /// 创建一个无容量限制的资产缓存
    pub fn new() -> Self {
        Self { entries: HashMap::new(), max_capacity: None }
    }

    /// 创建一个有容量限制的资产缓存
    pub fn with_capacity(max_capacity: usize) -> Self {
        Self { entries: HashMap::new(), max_capacity: Some(max_capacity) }
    }

    /// 插入或替换一个缓存条目，超出容量时按 LRU 策略淘汰
    pub fn insert(&mut self, guid: Guid, data: Vec<u8>, hash: String) {
        let now = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_secs();

        if let Some(max) = self.max_capacity {
            if self.entries.len() >= max && !self.entries.contains_key(&guid) {
                self.evict_lru();
            }
        }

        self.entries
            .insert(guid.clone(), CachedArtifact { guid, data, content_hash: hash, timestamp: now, last_accessed: now });
    }

    /// 根据 GUID 获取缓存条目的引用，同时更新访问时间
    pub fn get(&mut self, guid: &Guid) -> Option<&CachedArtifact> {
        if let Some(entry) = self.entries.get_mut(guid) {
            let now = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_secs();
            entry.last_accessed = now;
        }
        self.entries.get(guid)
    }

    /// 根据 GUID 获取缓存条目的不可变引用（不更新访问时间）
    pub fn peek(&self, guid: &Guid) -> Option<&CachedArtifact> {
        self.entries.get(guid)
    }

    /// 根据 GUID 移除并返回缓存条目
    pub fn remove(&mut self, guid: &Guid) -> Option<CachedArtifact> {
        self.entries.remove(guid)
    }

    /// 检查缓存中是否包含指定 GUID 的条目
    pub fn contains(&self, guid: &Guid) -> bool {
        self.entries.contains_key(guid)
    }

    /// 检查缓存条目是否与期望的哈希值匹配
    pub fn is_valid(&self, guid: &Guid, expected_hash: &str) -> bool {
        self.entries.get(guid).is_some_and(|entry| entry.content_hash == expected_hash)
    }

    /// 使指定 GUID 的缓存条目失效（移除）
    pub fn invalidate(&mut self, guid: &Guid) {
        self.entries.remove(guid);
    }

    /// 根据依赖图级联失效缓存条目
    ///
    /// 将指定资产及其所有传递性下游依赖的缓存条目标记为失效并移除
    pub fn invalidate_dependents(&mut self, guid: &Guid, dependency_graph: &DependencyGraph) {
        let dependents = dependency_graph.invalidate_dependents(guid);

        self.entries.remove(guid);

        for dependent_guid in &dependents {
            self.entries.remove(dependent_guid);
        }
    }

    /// 清空所有缓存条目
    pub fn clear(&mut self) {
        self.entries.clear();
    }

    /// 获取缓存中条目的总数
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// 检查缓存是否为空
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// 计算缓存中所有产物数据的总大小（字节）
    pub fn total_size(&self) -> usize {
        self.entries.values().map(|entry| entry.data.len()).sum()
    }

    /// 将缓存持久化到磁盘文件
    ///
    /// 使用 JSON 格式序列化所有缓存条目，写入指定路径
    pub fn persist_to_disk(&self, path: &Path) -> Result<(), AssetPipelineError> {
        let json = serde_json::to_string_pretty(&self.entries)?;
        std::fs::write(path, json)?;
        Ok(())
    }

    /// 从磁盘文件加载缓存
    ///
    /// 读取 JSON 格式的缓存文件，反序列化为缓存条目
    pub fn load_from_disk(&mut self, path: &Path) -> Result<(), AssetPipelineError> {
        let json = std::fs::read_to_string(path)?;
        let entries: HashMap<Guid, CachedArtifact> = serde_json::from_str(&json)?;
        self.entries = entries;
        Ok(())
    }

    /// 校验缓存条目的有效性
    ///
    /// 检查每个缓存条目的内容哈希是否与当前文件哈希一致，
    /// 移除不一致的缓存条目
    pub fn validate_entries<F>(&mut self, hash_fn: F) -> Vec<Guid>
    where
        F: Fn(&Guid) -> Option<String>,
    {
        let mut invalidated = Vec::new();

        let guids_to_remove: Vec<Guid> = self
            .entries
            .iter()
            .filter_map(|(guid, entry)| match hash_fn(guid) {
                Some(current_hash) if current_hash == entry.content_hash => None,
                _ => Some(guid.clone()),
            })
            .collect();

        for guid in &guids_to_remove {
            self.entries.remove(guid);
            invalidated.push(guid.clone());
        }

        invalidated
    }

    /// 克隆缓存供管线内部使用
    pub fn clone_for_pipeline(&self) -> Self {
        Self { entries: self.entries.clone(), max_capacity: self.max_capacity }
    }

    /// 按 LRU 策略淘汰最久未访问的缓存条目
    fn evict_lru(&mut self) {
        if self.entries.is_empty() {
            return;
        }

        let oldest_key = self.entries.iter().min_by_key(|(_, entry)| entry.last_accessed).map(|(guid, _)| guid.clone());

        if let Some(key) = oldest_key {
            self.entries.remove(&key);
        }
    }
}

impl Default for AssetCache {
    fn default() -> Self {
        Self::new()
    }
}
