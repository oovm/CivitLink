//! 缓存统计收集器实现
//!
//! 提供线程安全的缓存统计信息收集功能。

use crate::CacheStats;

/// 缓存统计收集器
///
/// 提供对缓存统计信息的收集和查询功能，
/// 支持记录命中和未命中事件。
pub struct CacheStatsCollector {
    /// 内部统计信息
    inner: CacheStats,
}

impl CacheStatsCollector {
    /// 创建新的统计收集器
    pub fn new() -> Self {
        Self { inner: CacheStats::new() }
    }

    /// 记录一次缓存命中
    pub fn record_hit(&mut self) {
        self.inner.record_hit();
    }

    /// 记录一次缓存未命中
    pub fn record_miss(&mut self) {
        self.inner.record_miss();
    }

    /// 获取当前统计快照
    pub fn snapshot(&self) -> CacheStats {
        self.inner.clone()
    }

    /// 获取当前命中率
    pub fn hit_rate(&self) -> f64 {
        self.inner.hit_rate()
    }

    /// 重置统计信息
    pub fn reset(&mut self) {
        self.inner = CacheStats::new();
    }
}

impl Default for CacheStatsCollector {
    fn default() -> Self {
        Self::new()
    }
}
