//! 缓存管理器实现
//!
//! 提供多级缓存管理，支持 L1（内存）+ L2（持久化）两级缓存架构
//! 和基于标签的缓存失效机制。

use crate::{CacheDriver, CacheStats, CacheValue};
use gg_core::GResult;
use std::{
    collections::{HashMap, HashSet},
    time::Duration,
};

/// 标签映射
///
/// 维护缓存键与标签之间的双向映射关系，
/// 支持通过标签批量失效相关缓存。
struct TagMapping {
    /// 标签到键集合的映射
    tag_to_keys: HashMap<String, HashSet<String>>,
    /// 键到标签集合的映射
    key_to_tags: HashMap<String, HashSet<String>>,
}

impl TagMapping {
    /// 创建新的标签映射
    fn new() -> Self {
        Self { tag_to_keys: HashMap::new(), key_to_tags: HashMap::new() }
    }

    /// 为键添加标签
    fn add_tags(&mut self, key: &str, tags: &[&str]) {
        let key_tags = self.key_to_tags.entry(key.to_string()).or_default();
        for tag in tags {
            key_tags.insert(tag.to_string());
            self.tag_to_keys.entry(tag.to_string()).or_default().insert(key.to_string());
        }
    }

    /// 获取标签关联的所有键
    fn keys_for_tag(&self, tag: &str) -> Vec<String> {
        self.tag_to_keys.get(tag).map(|keys| keys.iter().cloned().collect()).unwrap_or_default()
    }

    /// 移除键的所有标签映射
    fn remove_key(&mut self, key: &str) {
        if let Some(tags) = self.key_to_tags.remove(key) {
            for tag in &tags {
                if let Some(keys) = self.tag_to_keys.get_mut(tag) {
                    keys.remove(key);
                    if keys.is_empty() {
                        self.tag_to_keys.remove(tag);
                    }
                }
            }
        }
    }

    /// 清空所有映射
    fn clear(&mut self) {
        self.tag_to_keys.clear();
        self.key_to_tags.clear();
    }
}

/// 缓存管理器
///
/// 提供多级缓存管理功能，支持 L1（内存缓存）+ L2（持久化缓存）两级架构，
/// 以及基于标签的缓存失效机制。
///
/// L1 缓存用于高频访问的热数据，L2 缓存用于持久化存储。
/// 读取时先查 L1，命中则直接返回；未命中则查 L2，命中后回填 L1。
/// 写入时同时写入 L1 和 L2。
pub struct CacheManager {
    /// L1 缓存（内存缓存）
    l1: Box<dyn CacheDriver>,
    /// L2 缓存（持久化缓存）
    l2: Option<Box<dyn CacheDriver>>,
    /// 标签映射
    tag_mapping: TagMapping,
}

impl CacheManager {
    /// 创建仅 L1 缓存的管理器
    ///
    /// # 参数
    ///
    /// - `l1`: L1 内存缓存驱动
    pub fn new(l1: Box<dyn CacheDriver>) -> Self {
        Self { l1, l2: None, tag_mapping: TagMapping::new() }
    }

    /// 创建 L1 + L2 两级缓存的管理器
    ///
    /// # 参数
    ///
    /// - `l1`: L1 内存缓存驱动
    /// - `l2`: L2 持久化缓存驱动
    pub fn with_l2(l1: Box<dyn CacheDriver>, l2: Box<dyn CacheDriver>) -> Self {
        Self { l1, l2: Some(l2), tag_mapping: TagMapping::new() }
    }

    /// 获取缓存值
    ///
    /// 先查 L1 缓存，命中则直接返回；
    /// 未命中则查 L2 缓存，命中后回填 L1。
    pub fn get(&mut self, key: &str) -> GResult<Option<CacheValue>> {
        let l1_result = self.l1.get(key)?;
        if l1_result.is_some() {
            return Ok(l1_result);
        }

        if let Some(ref mut l2) = self.l2 {
            let l2_result = l2.get(key)?;
            if let Some(ref value) = l2_result {
                let _ = self.l1.set(key, value.clone(), None);
            }
            Ok(l2_result)
        }
        else {
            Ok(None)
        }
    }

    /// 设置缓存值
    ///
    /// 同时写入 L1 和 L2 缓存。
    pub fn set(&mut self, key: &str, value: CacheValue, ttl: Option<Duration>) -> GResult<()> {
        self.l1.set(key, value.clone(), ttl)?;
        if let Some(ref mut l2) = self.l2 {
            l2.set(key, value, ttl)?;
        }
        Ok(())
    }

    /// 设置带标签的缓存值
    ///
    /// 将缓存值与标签关联，支持后续通过标签批量失效。
    ///
    /// # 参数
    ///
    /// - `key`: 缓存键
    /// - `value`: 缓存值
    /// - `ttl`: 可选的生存时间
    /// - `tags`: 关联的标签列表
    pub fn set_with_tags(&mut self, key: &str, value: CacheValue, ttl: Option<Duration>, tags: &[&str]) -> GResult<()> {
        self.set(key, value, ttl)?;
        self.tag_mapping.add_tags(key, tags);
        Ok(())
    }

    /// 删除缓存值
    ///
    /// 同时从 L1 和 L2 缓存中删除，并移除标签映射。
    pub fn delete(&mut self, key: &str) -> GResult<bool> {
        self.tag_mapping.remove_key(key);
        let l1_deleted = self.l1.delete(key)?;
        let l2_deleted = if let Some(ref mut l2) = self.l2 { l2.delete(key)? } else { false };
        Ok(l1_deleted || l2_deleted)
    }

    /// 通过标签失效缓存
    ///
    /// 删除与指定标签关联的所有缓存条目。
    ///
    /// # 参数
    ///
    /// - `tag`: 要失效的标签
    pub fn invalidate_tag(&mut self, tag: &str) -> GResult<usize> {
        let keys = self.tag_mapping.keys_for_tag(tag);
        let count = keys.len();
        for key in &keys {
            self.l1.delete(key)?;
            if let Some(ref mut l2) = self.l2 {
                l2.delete(key)?;
            }
            self.tag_mapping.remove_key(key);
        }
        Ok(count)
    }

    /// 失效所有缓存
    ///
    /// 清空 L1 和 L2 缓存中的所有条目，并重置标签映射。
    pub fn invalidate_all(&mut self) -> GResult<()> {
        self.l1.clear()?;
        if let Some(ref mut l2) = self.l2 {
            l2.clear()?;
        }
        self.tag_mapping.clear();
        Ok(())
    }

    /// 获取 L1 缓存统计
    pub fn l1_stats(&self) -> CacheStats {
        self.l1.stats()
    }

    /// 获取 L2 缓存统计
    pub fn l2_stats(&self) -> Option<CacheStats> {
        self.l2.as_ref().map(|l2| l2.stats())
    }
}
