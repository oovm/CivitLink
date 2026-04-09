//! 服务注册表

use std::any::{Any, TypeId};
use std::collections::HashMap;

/// 类型擦除的服务容器
///
/// 使用 `TypeId` 作为键存储任意实现了 `Any + Send + Sync` 的服务实例，
/// 支持按类型注册和检索服务。
pub struct ServiceRegistry {
    services: HashMap<TypeId, Box<dyn Any + Send + Sync>>,
}

impl ServiceRegistry {
    /// 创建空的服务注册表
    pub fn new() -> Self {
        Self {
            services: HashMap::new(),
        }
    }

    /// 注册服务
    ///
    /// 如果相同类型的服务已存在，将被替换。
    pub fn register<T: Any + Send + Sync>(&mut self, service: T) {
        self.services.insert(TypeId::of::<T>(), Box::new(service));
    }

    /// 获取服务引用
    pub fn get<T: Any>(&self) -> Option<&T> {
        self.services
            .get(&TypeId::of::<T>())
            .and_then(|s| s.downcast_ref::<T>())
    }

    /// 获取服务可变引用
    pub fn get_mut<T: Any>(&mut self) -> Option<&mut T> {
        self.services
            .get_mut(&TypeId::of::<T>())
            .and_then(|s| s.downcast_mut::<T>())
    }
}

impl Default for ServiceRegistry {
    fn default() -> Self {
        Self::new()
    }
}
