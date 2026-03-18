//! 速率限制服务实现

use chrono::Utc;
use std::{collections::HashMap, sync::Arc};
use tokio::sync::RwLock;

use crate::rate_limit::{RateLimitConfig, RateLimitError, RateLimitKey, RateLimitResult};

/// 请求记录
#[derive(Debug, Clone)]
struct RequestRecord {
    /// 请求时间戳
    timestamps: Vec<i64>,
}

/// 速率限制服务
#[derive(Debug, Clone)]
pub struct RateLimitService {
    config: RateLimitConfig,
    requests: Arc<RwLock<HashMap<RateLimitKey, RequestRecord>>>,
}

impl RateLimitService {
    /// 创建新的速率限制服务
    ///
    /// # Arguments
    /// * `config` - 速率限制配置
    pub fn new(config: RateLimitConfig) -> Self {
        Self { config, requests: Arc::new(RwLock::new(HashMap::new())) }
    }

    /// 使用默认配置创建速率限制服务
    pub fn default() -> Self {
        Self::new(RateLimitConfig::default())
    }

    /// 检查并记录请求
    ///
    /// # Arguments
    /// * `key` - 速率限制键
    pub async fn check_and_record(&self, key: RateLimitKey) -> RateLimitResult<()> {
        let now = Utc::now().timestamp();
        let window_start = now - self.config.window_seconds as i64;

        let mut requests = self.requests.write().await;

        let record = requests.entry(key).or_insert_with(|| RequestRecord { timestamps: Vec::new() });

        record.timestamps.retain(|&ts| ts > window_start);

        if record.timestamps.len() >= self.config.max_requests as usize {
            let oldest_ts = record.timestamps.first().copied().unwrap_or(now);
            let retry_after = (oldest_ts + self.config.window_seconds as i64 - now).max(0) as u64;
            return Err(RateLimitError::RateLimitExceeded { retry_after });
        }

        record.timestamps.push(now);

        self.cleanup_old_records().await;

        Ok(())
    }

    /// 重置指定键的速率限制
    ///
    /// # Arguments
    /// * `key` - 速率限制键
    pub async fn reset(&self, key: &RateLimitKey) {
        self.requests.write().await.remove(key);
    }

    /// 获取当前请求计数
    ///
    /// # Arguments
    /// * `key` - 速率限制键
    pub async fn get_count(&self, key: &RateLimitKey) -> u32 {
        let now = Utc::now().timestamp();
        let window_start = now - self.config.window_seconds as i64;

        let requests = self.requests.read().await;

        requests.get(key).map(|record| record.timestamps.iter().filter(|&&ts| ts > window_start).count() as u32).unwrap_or(0)
    }

    /// 清理过期记录
    async fn cleanup_old_records(&self) {
        let now = Utc::now().timestamp();
        let window_start = now - self.config.window_seconds as i64;

        let mut requests = self.requests.write().await;
        requests.retain(|_, record| {
            record.timestamps.retain(|&ts| ts > window_start);
            !record.timestamps.is_empty()
        });
    }

    /// 获取配置
    pub fn config(&self) -> &RateLimitConfig {
        &self.config
    }
}

impl Default for RateLimitService {
    fn default() -> Self {
        Self::new(RateLimitConfig::default())
    }
}
