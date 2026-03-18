//! WAE Distributed - 分布式能力抽象层
//!
//! 提供统一的分布式系统能力抽象，包括分布式锁、功能开关、分布式ID生成等。
//!
//! 深度融合 tokio 运行时，所有 API 都是异步优先设计。
//! 微服务架构友好，支持高可用、高并发场景。

#![warn(missing_docs)]

use serde::{Deserialize, Serialize};
use std::{collections::HashSet, sync::Arc, time::Duration};
use wae_types::{WaeError, WaeResult};

pub use feature_flag::{FeatureFlag, FeatureFlagManager, FlagDefinition, Strategy, evaluate};
pub use id_generator::{IdGenerator, SnowflakeGenerator, UuidGenerator, UuidVersion};
pub use lock::{DistributedLock, InMemoryLock, InMemoryLockManager, LockOptions};

/// 功能开关模块
pub mod feature_flag {
    use super::*;
    use std::collections::HashMap;

    /// 功能开关策略
    #[derive(Debug, Clone, Default, Serialize, Deserialize)]
    pub enum Strategy {
        /// 始终开启
        On,
        /// 始终关闭
        #[default]
        Off,
        /// 百分比灰度
        Percentage(u32),
        /// 用户白名单
        UserList(Vec<String>),
    }

    /// 功能开关定义
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct FlagDefinition {
        /// 开关名称
        pub name: String,
        /// 开关描述
        pub description: String,
        /// 开关策略
        pub strategy: Strategy,
        /// 是否启用
        pub enabled: bool,
    }

    impl FlagDefinition {
        /// 创建新的功能开关定义
        pub fn new(name: impl Into<String>) -> Self {
            Self { name: name.into(), description: String::new(), strategy: Strategy::default(), enabled: false }
        }

        /// 设置描述
        pub fn with_description(mut self, description: impl Into<String>) -> Self {
            self.description = description.into();
            self
        }

        /// 设置策略
        pub fn with_strategy(mut self, strategy: Strategy) -> Self {
            self.strategy = strategy;
            self
        }

        /// 设置启用状态
        pub fn with_enabled(mut self, enabled: bool) -> Self {
            self.enabled = enabled;
            self
        }
    }

    /// 功能开关 trait (dyn 兼容)
    #[allow(async_fn_in_trait)]
    pub trait FeatureFlag: Send + Sync {
        /// 检查开关是否启用
        async fn is_enabled(&self, key: &str) -> bool;

        /// 检查开关是否启用 (带用户上下文)
        async fn is_enabled_for_user(&self, key: &str, user_id: &str) -> bool;

        /// 获取开关变体
        async fn get_variant(&self, key: &str) -> Option<String>;
    }

    /// 功能开关管理器
    pub struct FeatureFlagManager {
        flags: parking_lot::RwLock<HashMap<String, FlagDefinition>>,
    }

    impl FeatureFlagManager {
        /// 创建新的功能开关管理器
        pub fn new() -> Self {
            Self { flags: parking_lot::RwLock::new(HashMap::new()) }
        }

        /// 注册功能开关
        pub fn register(&self, flag: FlagDefinition) {
            let mut flags = self.flags.write();
            flags.insert(flag.name.clone(), flag);
        }

        /// 注销功能开关
        pub fn unregister(&self, name: &str) -> bool {
            let mut flags = self.flags.write();
            flags.remove(name).is_some()
        }

        /// 获取功能开关定义
        pub fn get(&self, name: &str) -> Option<FlagDefinition> {
            let flags = self.flags.read();
            flags.get(name).cloned()
        }

        /// 更新功能开关
        pub fn update(&self, name: &str, enabled: bool) -> bool {
            let mut flags = self.flags.write();
            if let Some(flag) = flags.get_mut(name) {
                flag.enabled = enabled;
                return true;
            }
            false
        }

        /// 列出所有功能开关
        pub fn list(&self) -> Vec<FlagDefinition> {
            let flags = self.flags.read();
            flags.values().cloned().collect()
        }
    }

    impl Default for FeatureFlagManager {
        fn default() -> Self {
            Self::new()
        }
    }

    impl FeatureFlag for FeatureFlagManager {
        async fn is_enabled(&self, key: &str) -> bool {
            let flags = self.flags.read();
            if let Some(flag) = flags.get(key) {
                return flag.enabled && matches!(flag.strategy, Strategy::On);
            }
            false
        }

        async fn is_enabled_for_user(&self, key: &str, user_id: &str) -> bool {
            let flags = self.flags.read();
            if let Some(flag) = flags.get(key) {
                return evaluate(&flag.strategy, user_id);
            }
            false
        }

        async fn get_variant(&self, _key: &str) -> Option<String> {
            None
        }
    }

    /// 评估开关状态
    pub fn evaluate(strategy: &Strategy, user_id: &str) -> bool {
        match strategy {
            Strategy::On => true,
            Strategy::Off => false,
            Strategy::Percentage(pct) => {
                let hash = calculate_hash(user_id);
                let bucket = hash % 100;
                bucket < *pct as u64
            }
            Strategy::UserList(users) => users.contains(&user_id.to_string()),
        }
    }

    fn calculate_hash(s: &str) -> u64 {
        let mut hash: u64 = 0;
        for c in s.chars() {
            hash = hash.wrapping_mul(31).wrapping_add(c as u64);
        }
        hash
    }
}

/// ID 生成器模块
pub mod id_generator {
    use parking_lot::Mutex;
    use std::time::{SystemTime, UNIX_EPOCH};

    /// ID 生成器 trait (dyn 兼容)
    #[allow(async_fn_in_trait)]
    pub trait IdGenerator: Send + Sync {
        /// 生成单个 ID
        async fn generate(&self) -> String;

        /// 批量生成 ID
        async fn generate_batch(&self, count: usize) -> Vec<String>;
    }

    /// 雪花算法 ID 生成器
    pub struct SnowflakeGenerator {
        worker_id: u64,
        datacenter_id: u64,
        sequence: Mutex<u64>,
        last_timestamp: Mutex<u64>,
    }

    impl SnowflakeGenerator {
        const EPOCH: u64 = 1704067200000;
        const WORKER_ID_BITS: u64 = 5;
        const DATACENTER_ID_BITS: u64 = 5;
        const SEQUENCE_BITS: u64 = 12;
        const MAX_WORKER_ID: u64 = (1 << Self::WORKER_ID_BITS) - 1;
        const MAX_DATACENTER_ID: u64 = (1 << Self::DATACENTER_ID_BITS) - 1;
        const SEQUENCE_MASK: u64 = (1 << Self::SEQUENCE_BITS) - 1;
        const WORKER_ID_SHIFT: u64 = Self::SEQUENCE_BITS;
        const DATACENTER_ID_SHIFT: u64 = Self::SEQUENCE_BITS + Self::WORKER_ID_BITS;
        const TIMESTAMP_SHIFT: u64 = Self::SEQUENCE_BITS + Self::WORKER_ID_BITS + Self::DATACENTER_ID_BITS;

        /// 创建新的雪花算法 ID 生成器
        pub fn new(worker_id: u64, datacenter_id: u64) -> Result<Self, String> {
            if worker_id > Self::MAX_WORKER_ID {
                return Err(format!("Worker ID must be between 0 and {}", Self::MAX_WORKER_ID));
            }
            if datacenter_id > Self::MAX_DATACENTER_ID {
                return Err(format!("Datacenter ID must be between 0 and {}", Self::MAX_DATACENTER_ID));
            }
            Ok(Self { worker_id, datacenter_id, sequence: Mutex::new(0), last_timestamp: Mutex::new(0) })
        }

        fn current_timestamp() -> u64 {
            SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis() as u64
        }

        fn til_next_millis(last_timestamp: u64) -> u64 {
            let mut timestamp = Self::current_timestamp();
            while timestamp <= last_timestamp {
                timestamp = Self::current_timestamp();
            }
            timestamp
        }

        fn generate_id(&self) -> u64 {
            let mut sequence = self.sequence.lock();
            let mut last_timestamp = self.last_timestamp.lock();

            let timestamp = Self::current_timestamp();

            if timestamp < *last_timestamp {
                panic!("Clock moved backwards!");
            }

            if timestamp == *last_timestamp {
                *sequence = (*sequence + 1) & Self::SEQUENCE_MASK;
                if *sequence == 0 {
                    *last_timestamp = Self::til_next_millis(*last_timestamp);
                }
            }
            else {
                *sequence = 0;
            }

            *last_timestamp = timestamp;

            ((timestamp - Self::EPOCH) << Self::TIMESTAMP_SHIFT)
                | (self.datacenter_id << Self::DATACENTER_ID_SHIFT)
                | (self.worker_id << Self::WORKER_ID_SHIFT)
                | *sequence
        }
    }

    impl IdGenerator for SnowflakeGenerator {
        async fn generate(&self) -> String {
            self.generate_id().to_string()
        }

        async fn generate_batch(&self, count: usize) -> Vec<String> {
            (0..count).map(|_| self.generate_id().to_string()).collect()
        }
    }

    /// UUID 版本
    #[derive(Debug, Clone, Copy, Default)]
    pub enum UuidVersion {
        /// V4 随机 UUID
        #[default]
        V4,
        /// V7 时间排序 UUID
        V7,
    }

    /// UUID 生成器
    pub struct UuidGenerator {
        version: UuidVersion,
    }

    impl UuidGenerator {
        /// 创建新的 UUID 生成器
        pub fn new(version: UuidVersion) -> Self {
            Self { version }
        }

        /// 创建 V4 UUID 生成器
        pub fn v4() -> Self {
            Self::new(UuidVersion::V4)
        }

        /// 创建 V7 UUID 生成器
        pub fn v7() -> Self {
            Self::new(UuidVersion::V7)
        }
    }

    impl Default for UuidGenerator {
        fn default() -> Self {
            Self::v4()
        }
    }

    impl IdGenerator for UuidGenerator {
        async fn generate(&self) -> String {
            match self.version {
                UuidVersion::V4 => uuid::Uuid::new_v4().to_string(),
                UuidVersion::V7 => {
                    let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis() as u64;
                    let random_bytes: [u8; 10] = {
                        let mut bytes = [0u8; 10];
                        for byte in &mut bytes {
                            *byte = rand_byte();
                        }
                        bytes
                    };

                    let mut uuid_bytes = [0u8; 16];
                    uuid_bytes[0..6].copy_from_slice(&now.to_be_bytes()[2..8]);
                    uuid_bytes[6..16].copy_from_slice(&random_bytes);

                    uuid_bytes[6] = (uuid_bytes[6] & 0x0F) | 0x70;
                    uuid_bytes[8] = (uuid_bytes[8] & 0x3F) | 0x80;

                    uuid::Uuid::from_bytes(uuid_bytes).to_string()
                }
            }
        }

        async fn generate_batch(&self, count: usize) -> Vec<String> {
            let mut result = Vec::with_capacity(count);
            for _ in 0..count {
                result.push(self.generate().await);
            }
            result
        }
    }

    fn rand_byte() -> u8 {
        use std::{
            collections::hash_map::RandomState,
            hash::{BuildHasher, Hasher},
        };
        let state = RandomState::new();
        let mut hasher = state.build_hasher();
        hasher.write_u64(SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos() as u64);
        (hasher.finish() & 0xFF) as u8
    }
}

/// 分布式锁模块
pub mod lock {
    use super::*;

    /// 锁选项
    #[derive(Debug, Clone)]
    pub struct LockOptions {
        /// 锁的生存时间
        pub ttl: Duration,
        /// 等待获取锁的超时时间
        pub wait_timeout: Duration,
    }

    impl Default for LockOptions {
        fn default() -> Self {
            Self { ttl: Duration::from_secs(30), wait_timeout: Duration::from_secs(10) }
        }
    }

    impl LockOptions {
        /// 创建新的锁选项
        pub fn new() -> Self {
            Self::default()
        }

        /// 设置锁的生存时间
        pub fn with_ttl(mut self, ttl: Duration) -> Self {
            self.ttl = ttl;
            self
        }

        /// 设置等待获取锁的超时时间
        pub fn with_wait_timeout(mut self, timeout: Duration) -> Self {
            self.wait_timeout = timeout;
            self
        }
    }

    /// 分布式锁 trait (dyn 兼容)
    #[allow(async_fn_in_trait)]
    pub trait DistributedLock: Send + Sync {
        /// 获取锁 (阻塞直到获取成功或超时)
        async fn lock(&self) -> WaeResult<()>;

        /// 尝试获取锁 (非阻塞)
        async fn try_lock(&self) -> WaeResult<bool>;

        /// 释放锁
        async fn unlock(&self) -> WaeResult<()>;

        /// 带超时的获取锁
        async fn lock_with_timeout(&self, timeout: Duration) -> WaeResult<()>;

        /// 获取锁的键名
        fn key(&self) -> &str;

        /// 检查锁是否被持有
        async fn is_locked(&self) -> bool;
    }

    /// 内存锁管理器
    pub struct InMemoryLockManager {
        locks: parking_lot::RwLock<HashSet<String>>,
    }

    impl InMemoryLockManager {
        /// 创建新的内存锁管理器
        pub fn new() -> Self {
            Self { locks: parking_lot::RwLock::new(HashSet::new()) }
        }

        /// 创建锁实例
        pub fn create_lock(&self, key: impl Into<String>) -> InMemoryLock {
            InMemoryLock::new(key, Arc::new(self.clone()))
        }

        async fn acquire_lock(&self, key: &str, _ttl: Duration) -> WaeResult<bool> {
            let mut locks = self.locks.write();
            if locks.contains(key) {
                return Ok(false);
            }
            locks.insert(key.to_string());
            Ok(true)
        }

        async fn release_lock(&self, key: &str) -> WaeResult<()> {
            let mut locks = self.locks.write();
            if locks.remove(key) {
                return Ok(());
            }
            Err(WaeError::not_found("Lock", key))
        }

        async fn is_locked(&self, key: &str) -> bool {
            self.locks.read().contains(key)
        }
    }

    impl Default for InMemoryLockManager {
        fn default() -> Self {
            Self::new()
        }
    }

    impl Clone for InMemoryLockManager {
        fn clone(&self) -> Self {
            Self { locks: parking_lot::RwLock::new(self.locks.read().clone()) }
        }
    }

    /// 内存锁实现
    pub struct InMemoryLock {
        key: String,
        manager: Arc<InMemoryLockManager>,
    }

    impl InMemoryLock {
        /// 创建新的内存锁
        pub fn new(key: impl Into<String>, manager: Arc<InMemoryLockManager>) -> Self {
            Self { key: key.into(), manager }
        }
    }

    impl DistributedLock for InMemoryLock {
        async fn lock(&self) -> WaeResult<()> {
            self.lock_with_timeout(Duration::from_secs(30)).await
        }

        async fn try_lock(&self) -> WaeResult<bool> {
            self.manager.acquire_lock(&self.key, Duration::from_secs(30)).await
        }

        async fn unlock(&self) -> WaeResult<()> {
            self.manager.release_lock(&self.key).await
        }

        async fn lock_with_timeout(&self, timeout: Duration) -> WaeResult<()> {
            let start = std::time::Instant::now();
            loop {
                if self.manager.acquire_lock(&self.key, Duration::from_secs(30)).await? {
                    return Ok(());
                }
                if start.elapsed() >= timeout {
                    return Err(WaeError::operation_timeout(format!("Lock key: {}", self.key), timeout.as_millis() as u64));
                }
                tokio::time::sleep(Duration::from_millis(50)).await;
            }
        }

        fn key(&self) -> &str {
            &self.key
        }

        async fn is_locked(&self) -> bool {
            self.manager.is_locked(&self.key).await
        }
    }
}
