//! Session 存储模块
//!
//! 提供 Session 存储的 trait 定义和内存实现。

use async_trait::async_trait;
use dashmap::DashMap;
use serde::{Serialize, de::DeserializeOwned};
use std::{sync::Arc, time::Duration};
use tokio::time::Instant;

/// Session 存储接口
///
/// 定义了 Session 存储的基本操作，支持异步操作。
/// 实现此 trait 可以创建自定义的 Session 存储后端。
#[async_trait]
pub trait SessionStore: Clone + Send + Sync + 'static {
    /// 获取 Session 数据
    async fn get(&self, session_id: &str) -> Option<String>;

    /// 设置 Session 数据
    async fn set(&self, session_id: &str, data: &str, ttl: Duration);

    /// 删除 Session
    async fn remove(&self, session_id: &str);

    /// 检查 Session 是否存在
    async fn exists(&self, session_id: &str) -> bool;

    /// 刷新 Session 过期时间
    async fn refresh(&self, session_id: &str, ttl: Duration);

    /// 获取类型化的 Session 数据
    async fn get_typed<T: DeserializeOwned + Send + Sync>(&self, session_id: &str) -> Option<T> {
        let data = self.get(session_id).await?;
        serde_json::from_str(&data).ok()
    }

    /// 设置类型化的 Session 数据
    async fn set_typed<T: Serialize + Send + Sync>(&self, session_id: &str, data: &T, ttl: Duration) -> bool {
        match serde_json::to_string(data) {
            Ok(json) => {
                self.set(session_id, &json, ttl).await;
                true
            }
            Err(_) => false,
        }
    }
}

/// Session 条目
#[derive(Debug, Clone)]
struct SessionEntry {
    /// Session 数据
    data: String,
    /// 过期时间
    expires_at: Instant,
}

/// 内存 Session 存储
///
/// 基于内存的 Session 存储实现，使用 DashMap 提供高并发访问。
/// 支持自动过期清理。
#[derive(Debug, Clone)]
pub struct MemorySessionStore {
    /// 存储映射
    storage: Arc<DashMap<String, SessionEntry>>,
    /// 清理间隔
    cleanup_interval: Duration,
    /// 是否启用自动清理
    #[allow(dead_code)]
    auto_cleanup: bool,
}

impl MemorySessionStore {
    /// 创建新的内存 Session 存储
    pub fn new() -> Self {
        Self::with_config(Duration::from_secs(60), true)
    }

    /// 使用配置创建内存 Session 存储
    pub fn with_config(cleanup_interval: Duration, auto_cleanup: bool) -> Self {
        let store = Self { storage: Arc::new(DashMap::new()), cleanup_interval, auto_cleanup };

        if auto_cleanup {
            store.start_cleanup_task();
        }

        store
    }

    /// 启动清理任务
    fn start_cleanup_task(&self) {
        let storage = Arc::clone(&self.storage);
        let interval = self.cleanup_interval;

        tokio::spawn(async move {
            loop {
                tokio::time::sleep(interval).await;
                let now = Instant::now();

                storage.retain(|_, entry| entry.expires_at > now);
            }
        });
    }

    /// 手动清理过期 Session
    pub fn cleanup_expired(&self) {
        let now = Instant::now();
        self.storage.retain(|_, entry| entry.expires_at > now);
    }

    /// 获取 Session 数量
    pub fn len(&self) -> usize {
        self.storage.len()
    }

    /// 检查是否为空
    pub fn is_empty(&self) -> bool {
        self.storage.is_empty()
    }

    /// 清空所有 Session
    pub fn clear(&self) {
        self.storage.clear();
    }
}

impl Default for MemorySessionStore {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl SessionStore for MemorySessionStore {
    async fn get(&self, session_id: &str) -> Option<String> {
        self.storage.get(session_id).filter(|entry| entry.expires_at > Instant::now()).map(|entry| entry.data.clone())
    }

    async fn set(&self, session_id: &str, data: &str, ttl: Duration) {
        let entry = SessionEntry { data: data.to_string(), expires_at: Instant::now() + ttl };
        self.storage.insert(session_id.to_string(), entry);
    }

    async fn remove(&self, session_id: &str) {
        self.storage.remove(session_id);
    }

    async fn exists(&self, session_id: &str) -> bool {
        self.storage.get(session_id).map(|entry| entry.expires_at > Instant::now()).unwrap_or(false)
    }

    async fn refresh(&self, session_id: &str, ttl: Duration) {
        if let Some(mut entry) = self.storage.get_mut(session_id) {
            entry.expires_at = Instant::now() + ttl;
        }
    }
}
