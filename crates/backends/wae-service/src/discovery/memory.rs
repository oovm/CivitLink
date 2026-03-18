//! 内存服务发现实现
//!
//! 提供基于内存的服务发现，适用于单机测试场景。

use async_trait::async_trait;
use std::{collections::HashMap, sync::Arc};
use tokio::sync::RwLock;

use crate::{ServiceDiscovery, ServiceInstance, ServiceResult, ServiceSubscription};

/// 内存服务发现
pub struct MemoryServiceDiscovery {
    services: Arc<RwLock<HashMap<String, Vec<ServiceInstance>>>>,
}

impl MemoryServiceDiscovery {
    /// 创建新的内存服务发现实例
    pub fn new() -> Self {
        Self { services: Arc::new(RwLock::new(HashMap::new())) }
    }

    /// 添加服务实例 (用于测试)
    pub async fn add_instance(&self, instance: ServiceInstance) {
        let mut services = self.services.write().await;
        services.entry(instance.name.clone()).or_insert_with(Vec::new).push(instance);
    }

    /// 移除服务实例 (用于测试)
    pub async fn remove_instance(&self, service_name: &str, instance_id: &str) {
        let mut services = self.services.write().await;
        if let Some(instances) = services.get_mut(service_name) {
            instances.retain(|i| i.id != instance_id);
        }
    }
}

impl Default for MemoryServiceDiscovery {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl ServiceDiscovery for MemoryServiceDiscovery {
    async fn discover(&self, service_name: &str) -> ServiceResult<Vec<ServiceInstance>> {
        let services = self.services.read().await;
        Ok(services.get(service_name).cloned().unwrap_or_default())
    }

    async fn discover_one(&self, service_name: &str) -> ServiceResult<Option<ServiceInstance>> {
        let services = self.services.read().await;
        Ok(services.get(service_name).and_then(|instances| instances.first().cloned()))
    }

    async fn subscribe(&self, service_name: &str) -> ServiceResult<Box<dyn ServiceSubscription>> {
        Ok(Box::new(MemoryServiceSubscription { service_name: service_name.to_string(), services: self.services.clone() }))
    }
}

/// 内存服务订阅
struct MemoryServiceSubscription {
    service_name: String,
    services: Arc<RwLock<HashMap<String, Vec<ServiceInstance>>>>,
}

#[async_trait]
impl ServiceSubscription for MemoryServiceSubscription {
    async fn instances(&self) -> ServiceResult<Vec<ServiceInstance>> {
        let services = self.services.read().await;
        Ok(services.get(&self.service_name).cloned().unwrap_or_default())
    }

    async fn wait_for_change(&mut self) -> ServiceResult<Vec<ServiceInstance>> {
        tokio::time::sleep(tokio::time::Duration::from_secs(30)).await;
        self.instances().await
    }
}
