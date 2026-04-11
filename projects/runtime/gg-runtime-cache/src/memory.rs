//! 内存缓存驱动实现
//!
//! 提供基于 HashMap 的内存缓存驱动，支持 TTL 过期和主动驱逐。

use crate::{CacheDriver, CacheValue};
use gg_core::GResult;
use std::{
    collections::HashMap,
    time::{Duration, Instant},
};

/// 缓存条目
///
/// 存储缓存值及其过期时间。
pub struct CacheEntry {
    /// 缓存值
    pub value: CacheValue,
    /// 过期时间点，None 表示永不过期
    pub expires_at: Option<Instant>,
}

impl CacheEntry {
    /// 检查条目是否已过期
    ///
    /// 如果设置了过期时间且已超过当前时间，则返回 true。
    pub fn is_expired(&self) -> bool {
        match self.expires_at {
            Some(expires_at) => Instant::now() >= expires_at,
            None => false,
        }
    }
}

/// 内存缓存驱动
///
/// 基于 HashMap 的内存缓存实现，支持 TTL 过期策略和主动驱逐过期条目。
pub struct MemoryCacheDriver {
    /// 缓存条目存储
    entries: HashMap<String, CacheEntry>,
}

impl MemoryCacheDriver {
    /// 创建新的内存缓存驱动
    pub fn new() -> Self {
        Self { entries: HashMap::new() }
    }

    /// 驱逐所有过期条目
    ///
    /// 遍历缓存中的所有条目，移除已过期的条目。
    pub fn evict_expired(&mut self) {
        self.entries.retain(|_, entry| !entry.is_expired());
    }
}

impl Default for MemoryCacheDriver {
    fn default() -> Self {
        Self::new()
    }
}

impl CacheDriver for MemoryCacheDriver {
    fn get(&mut self, key: &str) -> GResult<Option<CacheValue>> {
        match self.entries.get(key) {
            Some(entry) => {
                if entry.is_expired() {
                    self.entries.remove(key);
                    Ok(None)
                }
                else {
                    Ok(Some(entry.value.clone()))
                }
            }
            None => Ok(None),
        }
    }

    fn set(&mut self, key: &str, value: CacheValue, ttl: Option<Duration>) -> GResult<()> {
        let expires_at = ttl.map(|duration| Instant::now() + duration);
        let entry = CacheEntry { value, expires_at };
        self.entries.insert(key.to_string(), entry);
        Ok(())
    }

    fn delete(&mut self, key: &str) -> GResult<bool> {
        Ok(self.entries.remove(key).is_some())
    }

    fn exists(&mut self, key: &str) -> GResult<bool> {
        match self.entries.get(key) {
            Some(entry) => {
                if entry.is_expired() {
                    self.entries.remove(key);
                    Ok(false)
                }
                else {
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
}
