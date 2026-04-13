#![warn(missing_docs)]

//! 编译缓存模块
//! 提供编译产物的内存缓存、依赖追踪和磁盘持久化功能

use std::{collections::HashMap, path::Path};

use gg_core::{GError, GErrorKind, GResult};
use serde::{Deserialize, Serialize};

/// 缓存统计信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheStats {
    /// 缓存命中次数
    pub hits: u64,
    /// 缓存未命中次数
    pub misses: u64,
    /// 缓存条目总数
    pub entry_count: usize,
    /// 缓存数据占用的磁盘空间（字节）
    pub disk_size_bytes: u64,
}

impl Default for CacheStats {
    fn default() -> Self {
        Self { hits: 0, misses: 0, entry_count: 0, disk_size_bytes: 0 }
    }
}

/// 缓存条目，存储单个编译产物的缓存数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheEntry {
    /// 缓存条目键（由 ArtifactKey 派生）
    pub key: String,
    /// 缓存内容的哈希值
    pub content_hash: u64,
    /// 缓存的产物数据
    pub data: Vec<u8>,
    /// 依赖的缓存条目键列表
    pub dependencies: Vec<String>,
}

/// 编译缓存，支持内存缓存、依赖追踪和磁盘持久化
pub struct CompilationCache {
    /// 内存中的缓存条目
    entries: HashMap<String, CacheEntry>,
    /// 反向依赖图：键到依赖该键的条目列表的映射（用于级联失效）
    dependency_graph: HashMap<String, Vec<String>>,
    /// 缓存统计信息
    stats: CacheStats,
    /// 磁盘持久化路径
    #[allow(dead_code)]
    disk_path: Option<std::path::PathBuf>,
}

impl CompilationCache {
    /// 创建一个空的编译缓存
    pub fn new() -> Self {
        Self { entries: HashMap::new(), dependency_graph: HashMap::new(), stats: CacheStats::default(), disk_path: None }
    }

    /// 创建一个带有磁盘持久化路径的编译缓存
    pub fn with_disk_path(path: std::path::PathBuf) -> Self {
        Self { entries: HashMap::new(), dependency_graph: HashMap::new(), stats: CacheStats::default(), disk_path: Some(path) }
    }

    /// 插入一个缓存条目，同时更新依赖图
    pub fn insert(&mut self, key: &str, content_hash: u64, data: Vec<u8>, dependencies: Vec<String>) {
        if let Some(old) = self.entries.remove(key) {
            self.stats.entry_count -= 1;
            self.stats.disk_size_bytes -= old.data.len() as u64;

            for dep in &old.dependencies {
                if let Some(dependents) = self.dependency_graph.get_mut(dep) {
                    dependents.retain(|k| k != key);
                }
            }
        }

        let data_len = data.len() as u64;

        let entry = CacheEntry { key: key.to_string(), content_hash, data, dependencies: dependencies.clone() };

        self.stats.entry_count += 1;
        self.stats.disk_size_bytes += data_len;

        for dep in &dependencies {
            self.dependency_graph.entry(dep.clone()).or_default().push(key.to_string());
        }

        self.entries.insert(key.to_string(), entry);
    }

    /// 获取缓存数据，若哈希匹配则返回数据引用并更新统计信息
    pub fn get(&mut self, key: &str, content_hash: u64) -> Option<&Vec<u8>> {
        match self.entries.get(key) {
            Some(entry) if entry.content_hash == content_hash => {
                self.stats.hits += 1;
                Some(&entry.data)
            }
            _ => {
                self.stats.misses += 1;
                None
            }
        }
    }

    /// 使指定条目及其所有依赖该条目的下游条目失效（级联失效）
    pub fn invalidate(&mut self, key: &str) {
        let dependents = self.dependency_graph.get(key).cloned().unwrap_or_default();

        if let Some(entry) = self.entries.remove(key) {
            self.stats.entry_count -= 1;
            self.stats.disk_size_bytes -= entry.data.len() as u64;
        }

        self.dependency_graph.remove(key);

        for dependent in dependents {
            self.invalidate(&dependent);
        }
    }

    /// 清除所有缓存条目
    pub fn invalidate_all(&mut self) {
        self.entries.clear();
        self.dependency_graph.clear();
        self.stats.entry_count = 0;
        self.stats.disk_size_bytes = 0;
    }

    /// 检查指定条目是否存在且哈希匹配
    pub fn is_valid(&self, key: &str, content_hash: u64) -> bool {
        match self.entries.get(key) {
            Some(entry) => entry.content_hash == content_hash,
            None => false,
        }
    }

    /// 获取缓存统计信息的引用
    pub fn stats(&self) -> &CacheStats {
        &self.stats
    }

    /// 将所有缓存条目序列化到磁盘
    pub fn persist_to_disk(&self, path: &Path) -> GResult<()> {
        let entries: Vec<&CacheEntry> = self.entries.values().collect();

        let data = bincode::serde::encode_to_vec(&entries, bincode::config::standard())
            .map_err(|e| GError { kind: GErrorKind::Io, message: format!("缓存序列化失败: {}", e) })?;

        std::fs::write(path, data)?;

        Ok(())
    }

    /// 从磁盘反序列化缓存条目并验证哈希
    pub fn load_from_disk(&mut self, path: &Path) -> GResult<()> {
        let data = std::fs::read(path)?;

        let (entries, _): (Vec<CacheEntry>, _) = bincode::serde::decode_from_slice(&data, bincode::config::standard())
            .map_err(|e| GError { kind: GErrorKind::Io, message: format!("缓存反序列化失败: {}", e) })?;

        for entry in entries {
            let key = entry.key.clone();
            let content_hash = entry.content_hash;
            let data = entry.data;
            let dependencies = entry.dependencies;
            self.insert(&key, content_hash, data, dependencies);
        }

        Ok(())
    }

    /// 获取缓存条目数量
    pub fn entry_count(&self) -> usize {
        self.entries.len()
    }
}

impl Default for CompilationCache {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_insert_and_get() {
        let mut cache = CompilationCache::new();

        cache.insert("key1", 12345, vec![1, 2, 3], vec![]);

        let result = cache.get("key1", 12345);
        assert!(result.is_some());
        assert_eq!(result.unwrap(), &vec![1, 2, 3]);

        let result = cache.get("nonexistent", 12345);
        assert!(result.is_none());
    }

    #[test]
    fn test_hash_validation_miss() {
        let mut cache = CompilationCache::new();

        cache.insert("key1", 12345, vec![1, 2, 3], vec![]);

        let result = cache.get("key1", 99999);
        assert!(result.is_none());

        let result = cache.get("key1", 12345);
        assert!(result.is_some());
    }

    #[test]
    fn test_invalidate_single_entry() {
        let mut cache = CompilationCache::new();

        cache.insert("key1", 100, vec![1], vec![]);
        cache.insert("key2", 200, vec![2], vec![]);

        assert!(cache.is_valid("key1", 100));
        assert!(cache.is_valid("key2", 200));

        cache.invalidate("key1");

        assert!(!cache.is_valid("key1", 100));
        assert!(cache.is_valid("key2", 200));
        assert_eq!(cache.entry_count(), 1);
    }

    #[test]
    fn test_cascade_invalidation() {
        let mut cache = CompilationCache::new();

        cache.insert("a", 100, vec![1], vec![]);
        cache.insert("b", 200, vec![2], vec!["a".to_string()]);
        cache.insert("c", 300, vec![3], vec!["b".to_string()]);

        assert!(cache.is_valid("a", 100));
        assert!(cache.is_valid("b", 200));
        assert!(cache.is_valid("c", 300));

        cache.invalidate("a");

        assert!(!cache.is_valid("a", 100));
        assert!(!cache.is_valid("b", 200));
        assert!(!cache.is_valid("c", 300));
        assert_eq!(cache.entry_count(), 0);
    }

    #[test]
    fn test_stats_tracking() {
        let mut cache = CompilationCache::new();

        cache.insert("key1", 100, vec![1, 2, 3], vec![]);
        cache.insert("key2", 200, vec![4, 5, 6], vec![]);

        assert_eq!(cache.stats().entry_count, 2);
        assert_eq!(cache.stats().disk_size_bytes, 6);
        assert_eq!(cache.stats().hits, 0);
        assert_eq!(cache.stats().misses, 0);

        let _ = cache.get("key1", 100);
        assert_eq!(cache.stats().hits, 1);

        let _ = cache.get("key1", 999);
        assert_eq!(cache.stats().misses, 1);

        let _ = cache.get("nonexistent", 100);
        assert_eq!(cache.stats().misses, 2);
    }

    #[test]
    fn test_persist_and_load_round_trip() {
        let dir = std::env::temp_dir().join("gg_compiler_cache_test_round_trip");
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("cache.bin");

        let mut cache = CompilationCache::new();
        cache.insert("key1", 100, vec![1, 2, 3], vec![]);
        cache.insert("key2", 200, vec![4, 5, 6], vec!["key1".to_string()]);

        cache.persist_to_disk(&path).unwrap();

        let mut loaded_cache = CompilationCache::new();
        loaded_cache.load_from_disk(&path).unwrap();

        assert!(loaded_cache.is_valid("key1", 100));
        assert!(loaded_cache.is_valid("key2", 200));
        assert_eq!(loaded_cache.entry_count(), 2);
        assert_eq!(loaded_cache.stats().disk_size_bytes, 6);

        let data = loaded_cache.get("key1", 100);
        assert!(data.is_some());
        assert_eq!(data.unwrap(), &vec![1, 2, 3]);

        let data = loaded_cache.get("key2", 200);
        assert!(data.is_some());
        assert_eq!(data.unwrap(), &vec![4, 5, 6]);

        let _ = std::fs::remove_file(&path);
        let _ = std::fs::remove_dir(&dir);
    }

    #[test]
    fn test_invalidate_all() {
        let mut cache = CompilationCache::new();

        cache.insert("key1", 100, vec![1], vec![]);
        cache.insert("key2", 200, vec![2], vec!["key1".to_string()]);
        cache.insert("key3", 300, vec![3], vec!["key2".to_string()]);

        assert_eq!(cache.entry_count(), 3);

        cache.invalidate_all();

        assert_eq!(cache.entry_count(), 0);
        assert!(!cache.is_valid("key1", 100));
        assert!(!cache.is_valid("key2", 200));
        assert!(!cache.is_valid("key3", 300));
        assert_eq!(cache.stats().entry_count, 0);
        assert_eq!(cache.stats().disk_size_bytes, 0);
    }

    #[test]
    fn test_replace_entry() {
        let mut cache = CompilationCache::new();

        cache.insert("key1", 100, vec![1, 2], vec![]);
        assert_eq!(cache.stats().disk_size_bytes, 2);
        assert_eq!(cache.stats().entry_count, 1);

        cache.insert("key1", 200, vec![3, 4, 5], vec![]);
        assert_eq!(cache.stats().disk_size_bytes, 3);
        assert_eq!(cache.stats().entry_count, 1);

        let result = cache.get("key1", 200);
        assert!(result.is_some());
        assert_eq!(result.unwrap(), &vec![3, 4, 5]);

        let result = cache.get("key1", 100);
        assert!(result.is_none());
    }

    #[test]
    fn test_dependency_graph_update_on_replace() {
        let mut cache = CompilationCache::new();

        cache.insert("a", 100, vec![1], vec![]);
        cache.insert("b", 200, vec![2], vec!["a".to_string()]);

        cache.invalidate("a");
        assert!(!cache.is_valid("b", 200));

        cache.insert("a", 101, vec![10], vec![]);
        cache.insert("b", 201, vec![20], vec!["a".to_string()]);

        assert!(cache.is_valid("a", 101));
        assert!(cache.is_valid("b", 201));

        cache.invalidate("a");
        assert!(!cache.is_valid("b", 201));
    }

    #[test]
    fn test_with_disk_path() {
        let path = std::path::PathBuf::from("/tmp/test_cache");
        let cache = CompilationCache::with_disk_path(path.clone());
        assert_eq!(cache.disk_path, Some(path));
    }

    #[test]
    fn test_default() {
        let cache = CompilationCache::default();
        assert_eq!(cache.entry_count(), 0);
        assert_eq!(cache.stats().hits, 0);
        assert_eq!(cache.stats().misses, 0);
    }
}
