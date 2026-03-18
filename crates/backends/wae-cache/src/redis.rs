//! Redis 缓存后端实现

use crate::{CacheBackend, CacheConfig, CacheResult};
use async_trait::async_trait;
use deadpool_redis::{Config, Pool, Runtime};
use redis::AsyncCommands;
use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Redis 连接配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RedisConfig {
    /// Redis 服务器地址
    pub url: String,
    /// 连接池大小
    pub pool_size: usize,
    /// 连接超时时间
    pub connection_timeout: Option<Duration>,
    /// 操作超时时间
    pub operation_timeout: Option<Duration>,
    /// 连接重试次数
    pub retry_count: usize,
    /// 连接重试间隔
    pub retry_interval: Duration,
}

impl Default for RedisConfig {
    fn default() -> Self {
        Self {
            url: "redis://127.0.0.1:6379".to_string(),
            pool_size: 10,
            connection_timeout: Some(Duration::from_secs(5)),
            operation_timeout: Some(Duration::from_secs(3)),
            retry_count: 3,
            retry_interval: Duration::from_millis(100),
        }
    }
}

impl RedisConfig {
    /// 创建新的 Redis 配置
    pub fn new(url: String) -> Self {
        Self { url, ..Default::default() }
    }

    /// 设置连接池大小
    pub fn with_pool_size(mut self, pool_size: usize) -> Self {
        self.pool_size = pool_size;
        self
    }

    /// 设置连接超时
    pub fn with_connection_timeout(mut self, timeout: Duration) -> Self {
        self.connection_timeout = Some(timeout);
        self
    }

    /// 设置操作超时
    pub fn with_operation_timeout(mut self, timeout: Duration) -> Self {
        self.operation_timeout = Some(timeout);
        self
    }
}

/// Redis 缓存后端
pub struct RedisCacheBackend {
    pool: Pool,
    config: CacheConfig,
}

impl RedisCacheBackend {
    /// 创建新的 Redis 缓存后端
    pub fn new(redis_config: RedisConfig, cache_config: CacheConfig) -> CacheResult<Self> {
        let cfg = Config::from_url(redis_config.url);
        let pool = cfg
            .create_pool(Some(Runtime::Tokio1))
            .map_err(|e| wae_types::WaeError::internal(format!("Failed to create Redis pool: {}", e)))?;

        Ok(Self { pool, config: cache_config })
    }

    fn build_key(&self, key: &str) -> String {
        if self.config.key_prefix.is_empty() { key.to_string() } else { format!("{}:{}", self.config.key_prefix, key) }
    }
}

#[async_trait]
impl CacheBackend for RedisCacheBackend {
    async fn get_bytes(&self, key: &str) -> CacheResult<Option<Vec<u8>>> {
        let full_key = self.build_key(key);
        let mut conn = self
            .pool
            .get()
            .await
            .map_err(|e| wae_types::WaeError::internal(format!("Failed to get Redis connection: {}", e)))?;

        let result: Option<Vec<u8>> =
            conn.get(full_key).await.map_err(|e| wae_types::WaeError::internal(format!("Redis GET failed: {}", e)))?;

        Ok(result)
    }

    async fn set_bytes(&self, key: &str, value: &[u8], ttl: Option<Duration>) -> CacheResult<()> {
        let full_key = self.build_key(key);
        let effective_ttl = ttl.or(self.config.default_ttl);
        let mut conn = self
            .pool
            .get()
            .await
            .map_err(|e| wae_types::WaeError::internal(format!("Failed to get Redis connection: {}", e)))?;

        if let Some(ttl) = effective_ttl {
            let ttl_secs = ttl.as_secs() as u64;
            let _: () = conn
                .set_ex(full_key, value, ttl_secs)
                .await
                .map_err(|e| wae_types::WaeError::internal(format!("Redis SETEX failed: {}", e)))?;
        }
        else {
            let _: () = conn
                .set(full_key, value)
                .await
                .map_err(|e| wae_types::WaeError::internal(format!("Redis SET failed: {}", e)))?;
        }

        Ok(())
    }

    async fn delete(&self, key: &str) -> CacheResult<bool> {
        let full_key = self.build_key(key);
        let mut conn = self
            .pool
            .get()
            .await
            .map_err(|e| wae_types::WaeError::internal(format!("Failed to get Redis connection: {}", e)))?;

        let count: u64 =
            conn.del(full_key).await.map_err(|e| wae_types::WaeError::internal(format!("Redis DEL failed: {}", e)))?;

        Ok(count > 0)
    }

    async fn exists(&self, key: &str) -> CacheResult<bool> {
        let full_key = self.build_key(key);
        let mut conn = self
            .pool
            .get()
            .await
            .map_err(|e| wae_types::WaeError::internal(format!("Failed to get Redis connection: {}", e)))?;

        let count: u32 =
            conn.exists(full_key).await.map_err(|e| wae_types::WaeError::internal(format!("Redis EXISTS failed: {}", e)))?;

        Ok(count > 0)
    }

    async fn expire(&self, key: &str, ttl: Duration) -> CacheResult<bool> {
        let full_key = self.build_key(key);
        let mut conn = self
            .pool
            .get()
            .await
            .map_err(|e| wae_types::WaeError::internal(format!("Failed to get Redis connection: {}", e)))?;

        let ttl_secs = ttl.as_secs() as i64;
        let result: bool = conn
            .expire(full_key, ttl_secs)
            .await
            .map_err(|e| wae_types::WaeError::internal(format!("Redis EXPIRE failed: {}", e)))?;

        Ok(result)
    }

    async fn ttl(&self, key: &str) -> CacheResult<Option<Duration>> {
        let full_key = self.build_key(key);
        let mut conn = self
            .pool
            .get()
            .await
            .map_err(|e| wae_types::WaeError::internal(format!("Failed to get Redis connection: {}", e)))?;

        let ttl_secs: i64 =
            conn.ttl(full_key).await.map_err(|e| wae_types::WaeError::internal(format!("Redis TTL failed: {}", e)))?;

        if ttl_secs < 0 { Ok(None) } else { Ok(Some(Duration::from_secs(ttl_secs as u64))) }
    }

    async fn mget_bytes(&self, keys: &[&str]) -> CacheResult<Vec<Option<Vec<u8>>>> {
        let full_keys: Vec<String> = keys.iter().map(|k| self.build_key(k)).collect();
        let mut conn = self
            .pool
            .get()
            .await
            .map_err(|e| wae_types::WaeError::internal(format!("Failed to get Redis connection: {}", e)))?;

        let result: Vec<Option<Vec<u8>>> =
            conn.mget(full_keys).await.map_err(|e| wae_types::WaeError::internal(format!("Redis MGET failed: {}", e)))?;

        Ok(result)
    }

    async fn mset_bytes(&self, items: &[(&str, &[u8])], ttl: Option<Duration>) -> CacheResult<()> {
        let effective_ttl = ttl.or(self.config.default_ttl);
        let mut conn = self
            .pool
            .get()
            .await
            .map_err(|e| wae_types::WaeError::internal(format!("Failed to get Redis connection: {}", e)))?;

        let full_items: Vec<(String, &[u8])> = items.iter().map(|(k, v)| (self.build_key(k), *v)).collect();

        let mut pipe = redis::pipe();
        pipe.mset(&full_items);

        if let Some(ttl) = effective_ttl {
            let ttl_secs = ttl.as_secs() as i64;
            for (k, _) in full_items {
                pipe.expire(k, ttl_secs);
            }
        }

        let _: () = pipe
            .query_async(&mut conn)
            .await
            .map_err(|e| wae_types::WaeError::internal(format!("Redis MSET failed: {}", e)))?;

        Ok(())
    }

    async fn mdelete(&self, keys: &[&str]) -> CacheResult<u64> {
        let full_keys: Vec<String> = keys.iter().map(|k| self.build_key(k)).collect();
        let mut conn = self
            .pool
            .get()
            .await
            .map_err(|e| wae_types::WaeError::internal(format!("Failed to get Redis connection: {}", e)))?;

        let count: u64 =
            conn.del(full_keys).await.map_err(|e| wae_types::WaeError::internal(format!("Redis MDEL failed: {}", e)))?;

        Ok(count)
    }

    async fn incr(&self, key: &str, delta: i64) -> CacheResult<i64> {
        let full_key = self.build_key(key);
        let mut conn = self
            .pool
            .get()
            .await
            .map_err(|e| wae_types::WaeError::internal(format!("Failed to get Redis connection: {}", e)))?;

        let result: i64 = conn
            .incr(full_key, delta)
            .await
            .map_err(|e| wae_types::WaeError::internal(format!("Redis INCRBY failed: {}", e)))?;

        Ok(result)
    }

    async fn decr(&self, key: &str, delta: i64) -> CacheResult<i64> {
        let full_key = self.build_key(key);
        let mut conn = self
            .pool
            .get()
            .await
            .map_err(|e| wae_types::WaeError::internal(format!("Failed to get Redis connection: {}", e)))?;

        let result: i64 = conn
            .decr(full_key, delta)
            .await
            .map_err(|e| wae_types::WaeError::internal(format!("Redis DECRBY failed: {}", e)))?;

        Ok(result)
    }

    async fn clear(&self) -> CacheResult<()> {
        let mut conn = self
            .pool
            .get()
            .await
            .map_err(|e| wae_types::WaeError::internal(format!("Failed to get Redis connection: {}", e)))?;

        if self.config.key_prefix.is_empty() {
            let _: () = redis::cmd("FLUSHDB")
                .query_async(&mut conn)
                .await
                .map_err(|e| wae_types::WaeError::internal(format!("Redis FLUSHDB failed: {}", e)))?;
        }
        else {
            let pattern = format!("{}:*", self.config.key_prefix);
            let keys: Vec<String> = redis::cmd("KEYS")
                .arg(pattern)
                .query_async(&mut conn)
                .await
                .map_err(|e| wae_types::WaeError::internal(format!("Redis KEYS failed: {}", e)))?;

            if !keys.is_empty() {
                let _: u64 =
                    conn.del(keys).await.map_err(|e| wae_types::WaeError::internal(format!("Redis DEL failed: {}", e)))?;
            }
        }

        Ok(())
    }

    fn config(&self) -> &CacheConfig {
        &self.config
    }
}
