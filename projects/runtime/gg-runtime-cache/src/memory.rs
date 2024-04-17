//! 内存缓存驱动实现
//!
//! 提供基于 HashMap 的内存缓存驱动，支持 TTL 过期、LRU/LFU 驱逐策略和统计追踪。

use crate::{CacheDriver, CacheInvalidationStrategy, CacheStats, CacheValue};
use gg_core::GResult;
use std::{
    collections::HashMap,
    time::{Duration, Instant},
};

/// 缓存条目
///
/// 存储缓存值及其过期时间，并追踪访问信息用于驱逐策略。
pub struct CacheEntry {
    /// 缓存值
    pub value: CacheValue,
    /// 过期时间点，None 表示永不过期
    pub expires_at: Option<Instant>,
    /// 最后访问时间
    pub last_accessed: Instant,
    /// 访问次数
    pub access_count: u64,
}

impl CacheEntry {
    /// 创建新的缓存条目
    ///
    /// # 参数
    ///
    /// - `value`: 缓存值
    /// - `expires_at`: 过期时间点
    pub fn new(value: CacheValue, expires_at: Option<Instant>) -> Self {
        Self { value, expires_at, last_accessed: Instant::now(), access_count: 0 }
    }

    /// 检查条目是否已过期
    ///
    /// 如果设置了过期时间且已超过当前时间，则返回 true。
    pub fn is_expired(&self) -> bool {
        match self.expires_at {
            Some(expires_at) => Instant::now() >= expires_at,
            None => false,
        }
    }

    /// 记录一次访问
    ///
    /// 更新最后访问时间和访问计数。
    pub fn touch(&mut self) {
        self.last_accessed = Instant::now();
        self.access_count += 1;
    }

    /// 估算条目占用的内存大小（字节）
    pub fn estimated_size(&self) -> usize {
        let value_size = match &self.value {
            CacheValue::Null => 0,
            CacheValue::Integer(_) => 8,
            CacheValue::Real(_) => 8,
            CacheValue::Text(s) => s.len(),
            CacheValue::Blob(b) => b.len(),
            CacheValue::Bool(_) => 1,
        };
        value_size + std::mem::size_of::<Option<Instant>>() + std::mem::size_of::<Instant>()
            + std::mem::size_of::<u64>()
    }
}

/// 内存缓存驱动
///
/// 基于 HashMap 的内存缓存实现，支持 TTL 过期策略、LRU/LFU 驱逐策略和统计追踪。
pub struct MemoryCacheDriver {
    /// 缓存条目存储
    entries: HashMap<String, CacheEntry>,
    /// 驱逐策略
    strategy: CacheInvalidationStrategy,
    /// 最大容量（0 表示无限制）
    max_capacity: usize,
    /// 命中次数
    hits: u64,
    /// 未命中次数
    misses: u64,
}

impl MemoryCacheDriver {
    /// 创建新的内存缓存驱动
    ///
    /// 默认使用 TTL 驱逐策略，无容量限制。
    pub fn new() -> Self {
        Self {
            entries: HashMap::new(),
            strategy: CacheInvalidationStrategy::Ttl,
            max_capacity: 0,
            hits: 0,
            misses: 0,
        }
    }

    /// 使用指定的驱逐策略创建内存缓存驱动
    ///
    /// # 参数
    ///
    /// - `strategy`: 缓存驱逐策略
    pub fn with_strategy(strategy: CacheInvalidationStrategy) -> Self {
        Self {
            entries: HashMap::new(),
            strategy,
            max_capacity: 0,
            hits: 0,
            misses: 0,
        }
    }

    /// 设置最大容量
    ///
    /// 当缓存条目数超过此限制时，将根据驱逐策略淘汰条目。
    /// 设置为 0 表示无容量限制。
    ///
    /// # 参数
    ///
    /// - `capacity`: 最大容量
    pub fn with_max_capacity(mut self, capacity: usize) -> Self {
        self.max_capacity = capacity;
        self
    }

    /// 驱逐所有过期条目
    ///
    /// 遍历缓存中的所有条目，移除已过期的条目。
    pub fn evict_expired(&mut self) {
        self.entries.retain(|_, entry| !entry.is_expired());
    }

    /// 根据驱逐策略淘汰条目
    ///
    /// 当缓存容量达到上限时，根据配置的策略选择淘汰条目。
    /// 仅在设置了最大容量限制时生效。
    pub fn evict_by_strategy(&mut self) {
        if self.max_capacity == 0 || self.entries.len() <= self.max_capacity {
            return;
        }

        let evict_count = self.entries.len() - self.max_capacity;
        match self.strategy {
            CacheInvalidationStrategy::Ttl => {
                self.evict_expired();
            }
            CacheInvalidationStrategy::Lru => {
                let mut entries_with_time: Vec<(String, Instant)> = self
                    .entries
                    .iter()
                    .map(|(k, v)| (k.clone(), v.last_accessed))
                    .collect();
                entries_with_time.sort_by_key(|(_, t)| *t);
                for (key, _) in entries_with_time.into_iter().take(evict_count) {
                    self.entries.remove(&key);
                }
            }
            CacheInvalidationStrategy::Lfu => {
                let mut entries_with_count: Vec<(String, u64)> = self
                    .entries
                    .iter()
                    .map(|(k, v)| (k.clone(), v.access_count))
                    .collect();
                entries_with_count.sort_by_key(|(_, c)| *c);
                for (key, _) in entries_with_count.into_iter().take(evict_count) {
                    self.entries.remove(&key);
                }
            }
        }
    }

    /// 获取缓存条目数
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// 判断缓存是否为空
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

impl Default for MemoryCacheDriver {
    fn default() -> Self {
        Self::new()
    }
}

impl CacheDriver for MemoryCacheDriver {
    fn get(&mut self, key: &str) -> GResult<Option<CacheValue>> {
        match self.entries.get_mut(key) {
            Some(entry) => {
                if entry.is_expired() {
                    self.entries.remove(key);
                    self.misses += 1;
                    Ok(None)
                }
                else {
                    entry.touch();
                    self.hits += 1;
                    Ok(Some(entry.value.clone()))
                }
            }
            None => {
                self.misses += 1;
                Ok(None)
            }
        }
    }

    fn set(&mut self, key: &str, value: CacheValue, ttl: Option<Duration>) -> GResult<()> {
        let expires_at = ttl.map(|duration| Instant::now() + duration);
        let entry = CacheEntry::new(value, expires_at);
        self.entries.insert(key.to_string(), entry);
        self.evict_by_strategy();
        Ok(())
    }

    fn delete(&mut self, key: &str) -> GResult<bool> {
        Ok(self.entries.remove(key).is_some())
    }

    fn exists(&mut self, key: &str) -> GResult<bool> {
        match self.entries.get_mut(key) {
            Some(entry) => {
                if entry.is_expired() {
                    self.entries.remove(key);
                    Ok(false)
                }
                else {
                    entry.touch();
                    Ok(true)
                }
            }
            None => Ok(false),
        }
    }

    fn clear(&mut self) -> GResult<()> {
        self.entries.clear();
        Ok(())
    }

    fn stats(&self) -> CacheStats {
        CacheStats { hits: self.hits, misses: self.misses }
    }
}
