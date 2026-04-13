#![warn(missing_docs)]

//! GG 引擎缓存模块
//! 提供缓存驱动抽象与多种缓存后端实现

use gg_core::GResult;
use std::time::Duration;

/// 缓存值类型
///
/// 表示缓存中存储的值，支持多种基本数据类型。
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum CacheValue {
    /// 空值
    Null,
    /// 整数值
    Integer(i64),
    /// 浮点数值
    Real(f64),
    /// 文本字符串值
    Text(String),
    /// 二进制数据值
    Blob(Vec<u8>),
    /// 布尔值
    Bool(bool),
}

/// 缓存失效策略
///
/// 定义缓存条目的驱逐策略，用于在缓存空间不足时决定淘汰哪些条目。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CacheInvalidationStrategy {
    /// 基于生存时间的失效策略
    ///
    /// 条目在设定的 TTL 到期后自动失效。
    Ttl,
    /// 最近最少使用策略
    ///
    /// 当缓存空间不足时，优先淘汰最近最少被访问的条目。
    Lru,
    /// 最不经常使用策略
    ///
    /// 当缓存空间不足时，优先淘汰访问频率最低的条目。
    Lfu,
}

/// 缓存统计信息
///
/// 记录缓存操作的命中/未命中次数，用于监控缓存效率。
#[derive(Debug, Clone)]
pub struct CacheStats {
    /// 缓存命中次数
    pub hits: u64,
    /// 缓存未命中次数
    pub misses: u64,
}

impl CacheStats {
    /// 创建新的缓存统计实例
    ///
    /// 初始命中次数和未命中次数均为零。
    pub fn new() -> Self {
        Self { hits: 0, misses: 0 }
    }

    /// 记录一次缓存命中
    pub fn record_hit(&mut self) {
        self.hits += 1;
    }

    /// 记录一次缓存未命中
    pub fn record_miss(&mut self) {
        self.misses += 1;
    }

    /// 获取缓存命中率
    ///
    /// 返回命中次数占总访问次数的比例，范围 [0.0, 1.0]。
    /// 如果没有任何访问记录，返回 0.0。
    pub fn hit_rate(&self) -> f64 {
        let total = self.total_accesses();
        if total == 0 { 0.0 } else { self.hits as f64 / total as f64 }
    }

    /// 获取总访问次数
    ///
    /// 返回命中次数与未命中次数之和。
    pub fn total_accesses(&self) -> u64 {
        self.hits + self.misses
    }
}

impl Default for CacheStats {
    fn default() -> Self {
        Self::new()
    }
}

/// 缓存驱动 trait
///
/// 定义缓存操作的标准接口，所有缓存驱动都需要实现此 trait。
/// 基础方法必须实现，扩展方法提供默认实现。
pub trait CacheDriver {
    /// 获取缓存值
    ///
    /// 根据键名获取对应的缓存值，如果键不存在或已过期则返回 None。
    fn get(&mut self, key: &str) -> GResult<Option<CacheValue>>;

    /// 设置缓存值
    ///
    /// 将键值对存入缓存，可指定可选的生存时间（TTL）。
    fn set(&mut self, key: &str, value: CacheValue, ttl: Option<Duration>) -> GResult<()>;

    /// 删除缓存值
    ///
    /// 根据键名删除对应的缓存条目，返回是否成功删除。
    fn delete(&mut self, key: &str) -> GResult<bool>;

    /// 检查缓存键是否存在
    ///
    /// 检查指定键名是否存在且未过期。
    fn exists(&mut self, key: &str) -> GResult<bool>;

    /// 清空所有缓存
    ///
    /// 移除缓存中的所有条目。
    fn clear(&mut self) -> GResult<()>;

    /// 批量获取缓存值
    ///
    /// 根据多个键名获取对应的缓存值列表，
    /// 不存在的键对应位置返回 None。
    ///
    /// # 参数
    ///
    /// - `keys`: 键名列表
    fn get_many(&mut self, keys: &[&str]) -> GResult<Vec<Option<CacheValue>>> {
        keys.iter().map(|k| self.get(k)).collect()
    }

    /// 批量设置缓存值
    ///
    /// 将多个键值对存入缓存，所有条目使用相同的 TTL。
    ///
    /// # 参数
    ///
    /// - `entries`: 键值对列表
    /// - `ttl`: 可选的生存时间
    fn set_many(&mut self, entries: &[(&str, CacheValue)], ttl: Option<Duration>) -> GResult<()> {
        for (key, value) in entries {
            self.set(key, value.clone(), ttl)?;
        }
        Ok(())
    }

    /// 批量删除缓存值
    ///
    /// 根据多个键名删除对应的缓存条目，返回成功删除的数量。
    ///
    /// # 参数
    ///
    /// - `keys`: 键名列表
    fn delete_many(&mut self, keys: &[&str]) -> GResult<usize> {
        let mut deleted = 0;
        for key in keys {
            if self.delete(key)? {
                deleted += 1;
            }
        }
        Ok(deleted)
    }

    /// 递增缓存值
    ///
    /// 将指定键的整数值增加给定的步长。
    /// 如果键不存在，则将其初始化为步长值。
    ///
    /// # 参数
    ///
    /// - `key`: 键名
    /// - `delta`: 递增步长
    fn increment(&mut self, key: &str, delta: i64) -> GResult<i64> {
        let current = self.get(key)?;
        let new_value = match current {
            Some(CacheValue::Integer(n)) => n + delta,
            Some(_) => {
                return Err(gg_core::GError {
                    kind: gg_core::GErrorKind::Runtime,
                    message: format!("缓存键 '{}' 的值不是整数类型，无法递增", key),
                });
            }
            None => delta,
        };
        self.set(key, CacheValue::Integer(new_value), None)?;
        Ok(new_value)
    }

    /// 递减缓存值
    ///
    /// 将指定键的整数值减少给定的步长。
    /// 如果键不存在，则将其初始化为负步长值。
    ///
    /// # 参数
    ///
    /// - `key`: 键名
    /// - `delta`: 递减步长
    fn decrement(&mut self, key: &str, delta: i64) -> GResult<i64> {
        self.increment(key, -delta)
    }

    /// 刷新缓存条目的 TTL
    ///
    /// 重新设置指定键的生存时间，不影响缓存值本身。
    /// 如果键不存在则返回 false。
    ///
    /// # 参数
    ///
    /// - `key`: 键名
    /// - `ttl`: 新的生存时间
    fn touch(&mut self, key: &str, ttl: Duration) -> GResult<bool> {
        if !self.exists(key)? {
            return Ok(false);
        }
        let value = self.get(key)?;
        match value {
            Some(v) => {
                self.set(key, v, Some(ttl))?;
                Ok(true)
            }
            None => Ok(false),
        }
    }

    /// 获取缓存统计信息
    ///
    /// 返回当前缓存驱动的操作统计，包括命中次数和未命中次数。
    fn stats(&self) -> CacheStats;
}

/// 内存缓存驱动实现
pub mod memory;

/// 缓存统计模块
pub mod stats;

/// 文件缓存驱动实现
pub mod file;

/// Redis 缓存驱动实现
#[cfg(feature = "redis")]
pub mod redis;

/// 缓存管理器实现
pub mod manager;

pub use memory::MemoryCacheDriver;
pub use stats::CacheStatsCollector;
pub use file::FileCacheDriver;
#[cfg(feature = "redis")]
pub use redis::RedisCacheDriver;
pub use manager::CacheManager;
