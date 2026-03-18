//! WAE Service - 服务发现与注册模块
//!
//! 提供统一的服务发现与注册抽象，支持多种注册中心。
//!
//! 深度融合 tokio 运行时，支持：
//! - 服务注册与注销
//! - 服务发现与健康检查
//! - 负载均衡策略
//! - 服务元数据管理
#![warn(missing_docs)]

pub mod discovery;
pub mod registry;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, net::SocketAddr, time::Duration};

pub use wae_types::{WaeError, WaeResult};

/// 服务发现结果类型
pub type ServiceResult<T> = WaeResult<T>;

/// 服务发现错误类型
pub type ServiceError = WaeError;

/// 服务实例信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceInstance {
    /// 服务唯一标识
    pub id: String,
    /// 服务名称
    pub name: String,
    /// 服务地址
    pub addr: SocketAddr,
    /// 服务元数据
    pub metadata: HashMap<String, String>,
    /// 服务标签
    pub tags: Vec<String>,
    /// 注册时间
    pub registered_at: DateTime<Utc>,
    /// 最后心跳时间
    pub last_heartbeat: DateTime<Utc>,
    /// 服务权重 (用于负载均衡)
    pub weight: u32,
    /// 服务状态
    pub status: ServiceStatus,
}

/// 服务状态
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ServiceStatus {
    /// 健康
    Healthy,
    /// 不健康
    Unhealthy,
    /// 维护中
    Maintenance,
}

/// 服务注册配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceRegistration {
    /// 服务名称
    pub name: String,
    /// 服务地址
    pub addr: SocketAddr,
    /// 服务元数据
    pub metadata: HashMap<String, String>,
    /// 服务标签
    pub tags: Vec<String>,
    /// 服务权重
    pub weight: u32,
    /// 心跳间隔
    pub heartbeat_interval: Duration,
    /// 健康检查配置
    pub health_check: Option<HealthCheckConfig>,
}

impl Default for ServiceRegistration {
    fn default() -> Self {
        Self {
            name: "unnamed-service".to_string(),
            addr: "0.0.0.0:3000".parse().unwrap(),
            metadata: HashMap::new(),
            tags: Vec::new(),
            weight: 1,
            heartbeat_interval: Duration::from_secs(10),
            health_check: None,
        }
    }
}

/// 健康检查配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheckConfig {
    /// 健康检查路径
    pub path: String,
    /// 健康检查间隔
    pub interval: Duration,
    /// 健康检查超时
    pub timeout: Duration,
    /// 失败阈值
    pub failure_threshold: u32,
}

impl Default for HealthCheckConfig {
    fn default() -> Self {
        Self {
            path: "/health".to_string(),
            interval: Duration::from_secs(30),
            timeout: Duration::from_secs(5),
            failure_threshold: 3,
        }
    }
}

/// 服务注册中心 Trait
#[async_trait]
pub trait ServiceRegistry: Send + Sync {
    /// 注册服务
    async fn register(&self, instance: &ServiceInstance) -> ServiceResult<()>;

    /// 注销服务
    async fn deregister(&self, service_id: &str) -> ServiceResult<()>;

    /// 发送心跳
    async fn heartbeat(&self, service_id: &str) -> ServiceResult<()>;

    /// 更新服务状态
    async fn update_status(&self, service_id: &str, status: ServiceStatus) -> ServiceResult<()>;
}

/// 服务发现 Trait
#[async_trait]
pub trait ServiceDiscovery: Send + Sync {
    /// 发现服务实例
    async fn discover(&self, service_name: &str) -> ServiceResult<Vec<ServiceInstance>>;

    /// 发现单个服务实例 (根据负载均衡策略)
    async fn discover_one(&self, service_name: &str) -> ServiceResult<Option<ServiceInstance>>;

    /// 订阅服务变更
    async fn subscribe(&self, service_name: &str) -> ServiceResult<Box<dyn ServiceSubscription>>;
}

/// 服务订阅 Trait
///
/// 使用 `async_trait` 宏使其兼容 dyn。
#[async_trait]
pub trait ServiceSubscription: Send + Sync {
    /// 获取服务实例列表
    async fn instances(&self) -> ServiceResult<Vec<ServiceInstance>>;

    /// 等待服务变更
    async fn wait_for_change(&mut self) -> ServiceResult<Vec<ServiceInstance>>;
}

/// 负载均衡策略
#[derive(Debug, Clone, Copy, Default)]
pub enum LoadBalanceStrategy {
    /// 轮询
    #[default]
    RoundRobin,
    /// 随机
    Random,
    /// 加权轮询
    WeightedRoundRobin,
    /// 最少连接
    LeastConnections,
}

/// 负载均衡器
pub struct LoadBalancer {
    strategy: LoadBalanceStrategy,
    current_index: std::sync::atomic::AtomicUsize,
}

impl LoadBalancer {
    /// 创建负载均衡器
    pub fn new(strategy: LoadBalanceStrategy) -> Self {
        Self { strategy, current_index: std::sync::atomic::AtomicUsize::new(0) }
    }

    /// 选择服务实例
    pub fn select(&self, instances: &[ServiceInstance]) -> Option<ServiceInstance> {
        if instances.is_empty() {
            return None;
        }

        match self.strategy {
            LoadBalanceStrategy::RoundRobin => self.round_robin(instances),
            LoadBalanceStrategy::Random => self.random(instances),
            LoadBalanceStrategy::WeightedRoundRobin => self.weighted_round_robin(instances),
            LoadBalanceStrategy::LeastConnections => self.least_connections(instances),
        }
    }

    fn round_robin(&self, instances: &[ServiceInstance]) -> Option<ServiceInstance> {
        let index = self.current_index.fetch_add(1, std::sync::atomic::Ordering::Relaxed) % instances.len();
        instances.get(index).cloned()
    }

    fn random(&self, instances: &[ServiceInstance]) -> Option<ServiceInstance> {
        use std::{
            collections::hash_map::RandomState,
            hash::{BuildHasher, Hasher},
        };

        let state = RandomState::new();
        let mut hasher = state.build_hasher();
        hasher.write_u64(std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos() as u64);
        let index = hasher.finish() as usize % instances.len();
        instances.get(index).cloned()
    }

    fn weighted_round_robin(&self, instances: &[ServiceInstance]) -> Option<ServiceInstance> {
        let total_weight: u32 = instances.iter().map(|i| i.weight).sum();
        if total_weight == 0 {
            return self.round_robin(instances);
        }

        let mut random_value =
            (self.current_index.fetch_add(1, std::sync::atomic::Ordering::Relaxed) % total_weight as usize) as u32;

        for instance in instances {
            if random_value < instance.weight {
                return Some(instance.clone());
            }
            random_value -= instance.weight;
        }

        instances.last().cloned()
    }

    fn least_connections(&self, instances: &[ServiceInstance]) -> Option<ServiceInstance> {
        instances.iter().min_by_key(|i| i.weight).cloned()
    }
}

impl Default for LoadBalancer {
    fn default() -> Self {
        Self::new(LoadBalanceStrategy::default())
    }
}
