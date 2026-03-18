//! 资源管理
//!
//! 提供全局资源的存储和访问机制。

use std::any::{Any, TypeId};
use std::collections::HashMap;

use gwg_types::prelude::*;

/// 资源引用
pub struct ResourceRef<'a, T: Resource> {
    inner: &'a T,
}

impl<'a, T: Resource> ResourceRef<'a, T> {
    /// 创建新的资源引用
    pub fn new(inner: &'a T) -> Self {
        Self { inner }
    }

    /// 获取内部引用
    pub fn get(&self) -> &T {
        self.inner
    }
}

impl<'a, T: Resource> std::ops::Deref for ResourceRef<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.inner
    }
}

/// 资源可变引用
pub struct ResourceRefMut<'a, T: Resource> {
    inner: &'a mut T,
}

impl<'a, T: Resource> ResourceRefMut<'a, T> {
    /// 创建新的资源可变引用
    pub fn new(inner: &'a mut T) -> Self {
        Self { inner }
    }

    /// 获取内部可变引用
    pub fn get_mut(&mut self) -> &mut T {
        self.inner
    }
}

impl<'a, T: Resource> std::ops::Deref for ResourceRefMut<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.inner
    }
}

impl<'a, T: Resource> std::ops::DerefMut for ResourceRefMut<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.inner
    }
}

/// 资源容器
struct ResourceEntry {
    /// 资源数据
    data: Box<dyn Any + Send + Sync>,
}

/// 资源管理器
pub struct Resources {
    /// 资源映射
    resources: HashMap<TypeId, ResourceEntry>,
}

impl Resources {
    /// 创建新的资源管理器
    pub fn new() -> Self {
        Self {
            resources: HashMap::new(),
        }
    }

    /// 插入资源
    pub fn insert<T: Resource>(&mut self, resource: T) {
        let type_id = TypeId::of::<T>();
        self.resources.insert(
            type_id,
            ResourceEntry {
                data: Box::new(resource),
            },
        );
    }

    /// 获取资源引用
    pub fn get<T: Resource>(&self) -> Option<&T> {
        let type_id = TypeId::of::<T>();
        self.resources
            .get(&type_id)
            .and_then(|entry| entry.data.downcast_ref::<T>())
    }

    /// 获取资源可变引用
    pub fn get_mut<T: Resource>(&mut self) -> Option<&mut T> {
        let type_id = TypeId::of::<T>();
        self.resources
            .get_mut(&type_id)
            .and_then(|entry| entry.data.downcast_mut::<T>())
    }

    /// 移除资源
    pub fn remove<T: Resource>(&mut self) -> Option<T> {
        let type_id = TypeId::of::<T>();
        self.resources
            .remove(&type_id)
            .and_then(|entry| entry.data.downcast::<T>().ok())
            .map(|boxed| *boxed)
    }

    /// 检查是否包含资源
    pub fn contains<T: Resource>(&self) -> bool {
        self.resources.contains_key(&TypeId::of::<T>())
    }

    /// 检查是否为空
    pub fn is_empty(&self) -> bool {
        self.resources.is_empty()
    }

    /// 获取资源数量
    pub fn len(&self) -> usize {
        self.resources.len()
    }

    /// 清空所有资源
    pub fn clear(&mut self) {
        self.resources.clear();
    }
}

impl Default for Resources {
    fn default() -> Self {
        Self::new()
    }
}
