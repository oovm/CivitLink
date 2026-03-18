//! GWG Engine ECS 核心封装
//!
//! 基于 bevy_ecs 提供简洁友好的实体组件系统 API。

pub use bevy_ecs::prelude::*;
pub use bevy_ecs::schedule::{ScheduleLabel, SystemSet, *};

/// 实体 ID，用于唯一标识游戏世界中的实体
pub type EntityId = Entity;

/// 世界容器，管理所有实体、组件和系统
pub struct GwgWorld {
    inner: bevy_ecs::world::World,
}

impl GwgWorld {
    /// 创建一个新的空世界
    pub fn new() -> Self {
        Self {
            inner: bevy_ecs::world::World::new(),
        }
    }

    /// 生成一个新实体并返回其可变引用
    pub fn spawn(&mut self) -> EntityWorldMut<'_> {
        self.inner.spawn_empty()
    }

    /// 根据 ID 获取实体的可变引用
    pub fn entity_mut(&mut self, entity: Entity) -> EntityWorldMut<'_> {
        self.inner.entity_mut(entity)
    }

    /// 根据 ID 获取实体的不可变引用
    pub fn entity(&self, entity: Entity) -> EntityRef<'_> {
        self.inner.entity(entity)
    }

    /// 销毁指定的实体
    pub fn despawn(&mut self, entity: Entity) {
        self.inner.despawn(entity);
    }

    /// 插入或替换全局资源
    pub fn insert_resource<T: Resource>(&mut self, resource: T) {
        self.inner.insert_resource(resource);
    }

    /// 获取全局资源的不可变引用
    pub fn get_resource<T: Resource>(&self) -> Option<&T> {
        self.inner.get_resource()
    }

    /// 获取全局资源的可变引用
    pub fn get_resource_mut<T: Resource>(&mut self) -> Option<Mut<'_, T>> {
        self.inner.get_resource_mut()
    }

    /// 移除并返回全局资源
    pub fn remove_resource<T: Resource>(&mut self) -> Option<T> {
        self.inner.remove_resource()
    }

    /// 获取内部的 bevy_ecs World 引用
    pub fn inner(&self) -> &bevy_ecs::world::World {
        &self.inner
    }

    /// 获取内部的 bevy_ecs World 可变引用
    pub fn inner_mut(&mut self) -> &mut bevy_ecs::world::World {
        &mut self.inner
    }
}

impl Default for GwgWorld {
    fn default() -> Self {
        Self::new()
    }
}

/// 默认的调度标签
#[derive(ScheduleLabel, Clone, Debug, PartialEq, Eq, Hash)]
pub struct Update;

/// 系统调度器，用于组织和执行系统
pub struct GwgSchedule {
    inner: bevy_ecs::schedule::Schedule,
}

impl GwgSchedule {
    /// 创建一个新的空调度器
    pub fn new() -> Self {
        Self {
            inner: bevy_ecs::schedule::Schedule::new(Update),
        }
    }

    /// 添加一个系统到调度器
    pub fn add_system<M>(&mut self, system: impl IntoSystemConfigs<M>) {
        self.inner.add_systems(system);
    }

    /// 运行调度器中的所有系统
    pub fn run(&mut self, world: &mut GwgWorld) {
        self.inner.run(world.inner_mut());
    }

    /// 获取内部的 bevy_ecs Schedule 引用
    pub fn inner(&self) -> &bevy_ecs::schedule::Schedule {
        &self.inner
    }

    /// 获取内部的 bevy_ecs Schedule 可变引用
    pub fn inner_mut(&mut self) -> &mut bevy_ecs::schedule::Schedule {
        &mut self.inner
    }
}

impl Default for GwgSchedule {
    fn default() -> Self {
        Self::new()
    }
}

/// 组件 trait 别名
pub use bevy_ecs::component::Component;

/// 资源 trait 别名
pub use bevy_ecs::system::Resource;

/// 系统 trait 别名
pub use bevy_ecs::system::System;

/// 查询类型，用于从世界中获取实体和组件
pub use bevy_ecs::system::Query;

/// 变更检测包装类型
pub use bevy_ecs::change_detection::Mut;

/// 实体引用
pub use bevy_ecs::world::EntityRef;

/// 实体可变引用
pub use bevy_ecs::world::EntityWorldMut;

/// 实体迭代器
pub use bevy_ecs::entity::Entities;

/// 系统配置
pub use bevy_ecs::schedule::IntoSystemConfigs;

pub mod prelude {
    //! ECS 核心的预导入模块

    pub use super::*;
}
