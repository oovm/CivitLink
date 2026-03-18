//! World 实现
//!
//! World 是 ECS 的核心容器，管理所有实体、组件和资源。

use std::any::TypeId;
use std::collections::HashSet;

use gwg_types::prelude::*;

use crate::archetype::{ArchetypeGraph, ArchetypeId};
use crate::entity::{EntityAllocator, EntityLocation};
use crate::query::{Query, QueryMut, WorldQuery};
use crate::resource::Resources;

/// 实体引用
pub struct EntityRef<'w> {
    world: &'w World,
    entity: Entity,
    location: EntityLocation,
}

impl<'w> EntityRef<'w> {
    /// 获取实体 ID
    pub fn id(&self) -> Entity {
        self.entity
    }

    /// 获取组件
    pub fn get<T: Component>(&self) -> Option<&'w T> {
        let archetype = self.world.archetype_graph.get(ArchetypeId::new(self.location.archetype_id))?;
        archetype.get_component(self.location.row as usize)
    }

    /// 检查是否包含组件
    pub fn contains<T: Component>(&self) -> bool {
        match self.world.archetype_graph.get(ArchetypeId::new(self.location.archetype_id)) {
            Some(archetype) => archetype.has_component(TypeId::of::<T>()),
            None => false,
        }
    }
}

/// 实体可变引用
pub struct EntityMut<'w> {
    world: &'w mut World,
    entity: Entity,
    location: EntityLocation,
}

impl<'w> EntityMut<'w> {
    /// 获取实体 ID
    pub fn id(&self) -> Entity {
        self.entity
    }

    /// 获取组件
    pub fn get<T: Component>(&self) -> Option<&T> {
        let archetype = self.world.archetype_graph.get(ArchetypeId::new(self.location.archetype_id))?;
        archetype.get_component(self.location.row as usize)
    }

    /// 获取组件可变引用
    pub fn get_mut<T: Component>(&mut self) -> Option<&mut T> {
        let archetype = self.world.archetype_graph.get_mut(ArchetypeId::new(self.location.archetype_id))?;
        archetype.get_component_mut(self.location.row as usize)
    }

    /// 检查是否包含组件
    pub fn contains<T: Component>(&self) -> bool {
        match self.world.archetype_graph.get(ArchetypeId::new(self.location.archetype_id)) {
            Some(archetype) => archetype.has_component(TypeId::of::<T>()),
            None => false,
        }
    }

    /// 添加组件
    pub fn insert<T: Component>(&mut self, component: T) -> &mut Self {
        let old_archetype_id = ArchetypeId::new(self.location.archetype_id);
        
        let mut new_types = if let Some(old_archetype) = self.world.archetype_graph.get(old_archetype_id) {
            old_archetype.component_types().clone()
        } else {
            HashSet::new()
        };
        new_types.insert(TypeId::of::<T>());

        let new_archetype_id = self.world.archetype_graph.get_or_create(new_types);
        
        if old_archetype_id != new_archetype_id {
            self.move_to_archetype(old_archetype_id, new_archetype_id);
        }

        let archetype = self.world.archetype_graph.get_mut(new_archetype_id).unwrap();
        archetype.add_component(self.location.row as usize, component);
        self.location.archetype_id = new_archetype_id.0;
        
        self.world.entities.set_location(self.entity, self.location);
        
        self
    }

    /// 移动到新 Archetype
    fn move_to_archetype(&mut self, old_id: ArchetypeId, new_id: ArchetypeId) {
        let row = self.location.row as usize;
        
        let old_archetype = self.world.archetype_graph.get_mut(old_id).unwrap();
        old_archetype.remove_entity(row);

        let new_archetype = self.world.archetype_graph.get_mut(new_id).unwrap();
        let new_row = new_archetype.add_entity(self.entity);
        self.location.row = new_row as u32;
        self.location.archetype_id = new_id.0;
    }

    /// 移除组件
    pub fn remove<T: Component>(&mut self) -> Option<T> {
        let archetype_id = ArchetypeId::new(self.location.archetype_id);
        let archetype = self.world.archetype_graph.get_mut(archetype_id)?;
        
        if !archetype.has_component(TypeId::of::<T>()) {
            return None;
        }

        let component = archetype.get_component_mut::<T>(self.location.row as usize).map(|c| unsafe {
            std::ptr::read(c as *const T)
        });

        let mut new_types = archetype.component_types().clone();
        new_types.remove(&TypeId::of::<T>());

        let new_archetype_id = self.world.archetype_graph.get_or_create(new_types);
        self.move_to_archetype(archetype_id, new_archetype_id);

        component
    }
}

/// World，ECS 的核心容器
pub struct World {
    /// 实体分配器
    entities: EntityAllocator,
    /// Archetype 图
    archetype_graph: ArchetypeGraph,
    /// 资源管理器
    resources: Resources,
    /// 全局时钟
    tick: u32,
}

impl World {
    /// 创建新的世界
    pub fn new() -> Self {
        Self {
            entities: EntityAllocator::new(),
            archetype_graph: ArchetypeGraph::new(),
            resources: Resources::new(),
            tick: 0,
        }
    }

    /// 生成一个空实体
    pub fn spawn_empty(&mut self) -> Entity {
        let entity = self.entities.allocate();
        let archetype_id = self.archetype_graph.empty_archetype();
        let archetype = self.archetype_graph.get_mut(archetype_id).unwrap();
        let row = archetype.add_entity(entity);
        
        let location = EntityLocation::new(archetype_id.0, row as u32);
        self.entities.set_location(entity, location);
        
        entity
    }

    /// 销毁实体
    pub fn despawn(&mut self, entity: Entity) -> bool {
        if let Some(location) = self.entities.destroy(entity) {
            let archetype_id = ArchetypeId::new(location.archetype_id);
            if let Some(archetype) = self.archetype_graph.get_mut(archetype_id) {
                archetype.remove_entity(location.row as usize);
            }
            true
        } else {
            false
        }
    }

    /// 获取实体引用
    pub fn entity(&self, entity: Entity) -> Option<EntityRef<'_>> {
        let location = self.entities.get_location(entity)?;
        Some(EntityRef {
            world: self,
            entity,
            location,
        })
    }

    /// 获取实体可变引用
    pub fn entity_mut(&mut self, entity: Entity) -> Option<EntityMut<'_>> {
        let location = self.entities.get_location(entity)?;
        Some(EntityMut {
            world: self,
            entity,
            location,
        })
    }

    /// 检查实体是否存在
    pub fn contains_entity(&self, entity: Entity) -> bool {
        self.entities.is_alive(entity)
    }

    /// 插入资源
    pub fn insert_resource<T: Resource>(&mut self, resource: T) {
        self.resources.insert(resource);
    }

    /// 获取资源
    pub fn get_resource<T: Resource>(&self) -> Option<&T> {
        self.resources.get::<T>()
    }

    /// 获取资源可变引用
    pub fn get_resource_mut<T: Resource>(&mut self) -> Option<&mut T> {
        self.resources.get_mut::<T>()
    }

    /// 移除资源
    pub fn remove_resource<T: Resource>(&mut self) -> Option<T> {
        self.resources.remove::<T>()
    }

    /// 检查是否包含资源
    pub fn contains_resource<T: Resource>(&self) -> bool {
        self.resources.contains::<T>()
    }

    /// 创建查询
    pub fn query<Q: WorldQuery>(&self) -> Query<'_, Q> {
        Query::new(self)
    }

    /// 创建可变查询
    pub fn query_mut<Q: WorldQuery>(&mut self) -> QueryMut<'_, Q> {
        QueryMut::new(self)
    }

    /// 获取实体分配器
    pub fn entities(&self) -> &EntityAllocator {
        &self.entities
    }

    /// 获取 Archetype 图
    pub fn archetype_graph(&self) -> &ArchetypeGraph {
        &self.archetype_graph
    }

    /// 获取 Archetype 图可变引用
    pub fn archetype_graph_mut(&mut self) -> &mut ArchetypeGraph {
        &mut self.archetype_graph
    }

    /// 获取资源管理器
    pub fn resources(&self) -> &Resources {
        &self.resources
    }

    /// 获取资源管理器可变引用
    pub fn resources_mut(&mut self) -> &mut Resources {
        &mut self.resources
    }

    /// 获取当前时钟
    pub fn tick(&self) -> u32 {
        self.tick
    }

    /// 增加时钟
    pub fn increment_tick(&mut self) {
        self.tick = self.tick.wrapping_add(1);
    }

    /// 获取实体数量
    pub fn len(&self) -> usize {
        self.entities.len()
    }

    /// 检查是否为空
    pub fn is_empty(&self) -> bool {
        self.entities.is_empty()
    }

    /// 清空世界
    pub fn clear(&mut self) {
        self.entities = EntityAllocator::new();
        self.archetype_graph = ArchetypeGraph::new();
        self.resources.clear();
        self.tick = 0;
    }
}

impl Default for World {
    fn default() -> Self {
        Self::new()
    }
}

unsafe impl Send for World {}
unsafe impl Sync for World {}
