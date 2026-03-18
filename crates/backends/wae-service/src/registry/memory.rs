//! 内存服务注册实现
//!
//! 提供基于内存的服务注册，适用于单机测试场景。

use chrono::Utc;
use std::{collections::HashMap, sync::Arc};
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::{ServiceInstance, ServiceRegistry, ServiceResult, ServiceStatus};

/// 内存服务注册中心
pub struct MemoryServiceRegistry {
    services: Arc<RwLock<HashMap<String, ServiceInstance>>>,
}

impl MemoryServiceRegistry {
    /// 创建新的内存服务注册中心
    pub fn new() -> Self {
        Self { services: Arc::new(RwLock::new(HashMap::new())) }
    }

    /// 获取所有服务实例
    pub async fn get_all(&self) -> Vec<ServiceInstance> {
        let services = self.services.read().await;
        services.values().cloned().collect()
    }

    /// 获取服务实例
    pub async fn get(&self, service_id: &str) -> Option<ServiceInstance> {
        let services = self.services.read().await;
        services.get(service_id).cloned()
    }
}

impl Default for MemoryServiceRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait::async_trait]
impl ServiceRegistry for MemoryServiceRegistry {
    async fn register(&self, instance: &ServiceInstance) -> ServiceResult<()> {
        let mut services = self.services.write().await;
        services.insert(instance.id.clone(), instance.clone());
        Ok(())
    }

    async fn deregister(&self, service_id: &str) -> ServiceResult<()> {
        let mut services = self.services.write().await;
        services.remove(service_id);
        Ok(())
    }

    async fn heartbeat(&self, service_id: &str) -> ServiceResult<()> {
        let mut services = self.services.write().await;
        if let Some(instance) = services.get_mut(service_id) {
            instance.last_heartbeat = Utc::now();
        }
        Ok(())
    }

    async fn update_status(&self, service_id: &str, status: ServiceStatus) -> ServiceResult<()> {
        let mut services = self.services.write().await;
        if let Some(instance) = services.get_mut(service_id) {
            instance.status = status;
        }
        Ok(())
    }
}

/// 服务实例构建器
#[allow(dead_code)]
pub struct ServiceInstanceBuilder {
    name: String,
    addr: std::net::SocketAddr,
    metadata: HashMap<String, String>,
    tags: Vec<String>,
    weight: u32,
}

impl ServiceInstanceBuilder {
    #[allow(dead_code)]
    /// 创建新的服务实例构建器
    pub fn new(name: impl Into<String>, addr: std::net::SocketAddr) -> Self {
        Self { name: name.into(), addr, metadata: HashMap::new(), tags: Vec::new(), weight: 1 }
    }

    #[allow(dead_code)]
    /// 添加元数据
    pub fn metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }

    #[allow(dead_code)]
    /// 添加标签
    pub fn tag(mut self, tag: impl Into<String>) -> Self {
        self.tags.push(tag.into());
        self
    }

    #[allow(dead_code)]
    /// 设置权重
    pub fn weight(mut self, weight: u32) -> Self {
        self.weight = weight;
        self
    }

    #[allow(dead_code)]
    /// 构建服务实例
    pub fn build(self) -> ServiceInstance {
        let now = Utc::now();
        ServiceInstance {
            id: Uuid::new_v4().to_string(),
            name: self.name,
            addr: self.addr,
            metadata: self.metadata,
            tags: self.tags,
            registered_at: now,
            last_heartbeat: now,
            weight: self.weight,
            status: ServiceStatus::Healthy,
        }
    }
}
