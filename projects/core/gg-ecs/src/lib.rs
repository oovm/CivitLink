#![warn(missing_docs)]

//! GG 引擎 ECS 核心模块
//!
//! 提供实体-组件-系统架构，基于 Archetype 存储实现高性能的实体管理、组件存储和查询功能。

extern crate self as gg_ecs;

pub mod archetype;
pub mod entity;
pub mod query;
pub mod storage;

use std::{
    any::{Any, TypeId},
    collections::HashMap,
    marker::PhantomData,
};

use archetype::{ArchetypeGraph, ArchetypeId};
pub use entity::{Entity, EntityAllocator, EntityLocation};
use gg_error::{GError, GErrorKind, GResult};
pub use gg_macros::{Component, Resource};

pub use entity::Entity as EntityType;

/// 组件 trait，所有组件必须实现此 trait
///
/// 使用 blanket impl，任何满足 `Any + Send + Sync` 的类型自动成为组件。
pub trait Component: Any + Send + Sync {}

impl<T: Any + Send + Sync> Component for T {}

/// 资源 trait，全局单例数据必须实现此 trait
///
/// 使用 blanket impl，任何满足 `Any + Send + Sync` 的类型自动成为资源。
pub trait Resource: Any + Send + Sync {}

impl<T: Any + Send + Sync> Resource for T {}

/// 系统 trait，所有系统必须实现此 trait
pub trait System {
    /// 获取系统名称
    fn name(&self) -> &str;

    /// 执行系统逻辑
    fn execute(&mut self, world: &mut World) -> GResult<()>;
}

/// 查询过滤器 trait，用于在查询时过滤实体
pub trait QueryFilter {
    /// 检查实体是否满足过滤条件
    fn matches(world: &World, entity: Entity) -> bool;
}

/// 无过滤器的查询，匹配所有实体
pub struct NoneFilter;

impl QueryFilter for NoneFilter {
    fn matches(_world: &World, _entity: Entity) -> bool {
        true
    }
}

/// 查询过滤器：仅匹配包含指定组件的实体
pub struct With<T: Component> {
    _marker: PhantomData<T>,
}

impl<T: Component> QueryFilter for With<T> {
    fn matches(world: &World, entity: Entity) -> bool {
        world.get_component::<T>(entity).is_some()
    }
}

/// 查询过滤器：仅匹配不包含指定组件的实体
pub struct Without<T: Component> {
    _marker: PhantomData<T>,
}

impl<T: Component> QueryFilter for Without<T> {
    fn matches(world: &World, entity: Entity) -> bool {
        world.get_component::<T>(entity).is_none()
    }
}

/// 变更检测过滤器：仅匹配自上次 tick 以来组件值发生变化的实体
pub struct Changed<T: Component> {
    _marker: PhantomData<T>,
}

impl<T: Component> QueryFilter for Changed<T> {
    fn matches(world: &World, entity: Entity) -> bool {
        if let Some(location) = world.entity_allocator.get_location(entity) {
            let archetype_id = ArchetypeId::new(location.archetype_id);
            if let Some(archetype) = world.archetype_graph.get(archetype_id) {
                let type_id = TypeId::of::<T>();
                if archetype.has_component(type_id) {
                    return world.changed_ticks.get(&(entity, type_id)).map_or(false, |&tick| tick == world.tick);
                }
            }
        }
        false
    }
}

/// 组件查询迭代器，用于遍历拥有指定组件类型且满足过滤条件的所有实体
pub struct Query<'a, T: Component, F: QueryFilter = NoneFilter> {
    /// 底层组件列迭代器
    inner: Option<storage::ComponentColumnIter<'a, T>>,
    /// 世界引用，用于过滤检查
    world: &'a World,
    /// 过滤器类型标记
    _filter: PhantomData<F>,
    /// 组件类型标记
    _marker: PhantomData<&'a T>,
}

impl<'a, T: Component, F: QueryFilter> Iterator for Query<'a, T, F> {
    type Item = (Entity, &'a T);

    fn next(&mut self) -> Option<Self::Item> {
        let iter = self.inner.as_mut()?;
        loop {
            let (entity, component) = iter.next()?;
            if F::matches(self.world, entity) {
                return Some((entity, component));
            }
        }
    }
}

/// 实体构建器，用于链式创建实体并添加组件
pub struct EntityBuilder<'a> {
    /// 世界可变引用
    world: &'a mut World,
    /// 实体 ID
    entity: Entity,
}

/// 多组件查询迭代器，支持元组查询
///
/// 遍历所有匹配 WorldQuery 条件的 Archetype，
/// 逐行提取实体和组件数据。
pub struct MultiQuery<'a, Q: query::WorldQuery> {
    /// 世界引用
    world: &'a World,
    /// 匹配的 Archetype 列表及其查询状态
    matching_archetypes: Vec<(ArchetypeId, Q::State)>,
    /// 当前 Archetype 索引
    current_archetype_idx: usize,
    /// 当前行索引
    current_row: usize,
    /// 查询类型标记
    _marker: PhantomData<Q>,
}

impl<'a, Q: query::WorldQuery> MultiQuery<'a, Q> {
    /// 使用 for_each 回调模式遍历所有匹配的实体和组件数据
    ///
    /// 由于 Rust 借用检查的限制，多组件查询使用回调模式而非迭代器。
    pub fn for_each<F, R>(&mut self, mut f: F)
    where
        F: FnMut(Entity, R),
        R: query::QueryResult<'a>,
    {
        while self.current_archetype_idx < self.matching_archetypes.len() {
            let (archetype_id, _state) = &self.matching_archetypes[self.current_archetype_idx];
            let archetype_id = *archetype_id;
            if let Some(archetype) = self.world.archetype_graph.get(archetype_id) {
                while self.current_row < archetype.len() {
                    if let Some(entity) = archetype.get_entity(self.current_row) {
                        if let Some(result) = R::fetch(archetype, self.current_row) {
                            f(entity, result);
                        }
                    }
                    self.current_row += 1;
                }
            }
            self.current_archetype_idx += 1;
            self.current_row = 0;
        }
    }
}

impl<'a> EntityBuilder<'a> {
    /// 向实体插入组件，返回自身以支持链式调用
    pub fn insert<T: Component>(self, component: T) -> Self {
        let _ = self.world.add_component(self.entity, component);
        self
    }

    /// 获取当前构建实体的 ID
    pub fn id(&self) -> Entity {
        self.entity
    }
}

/// GG 引擎世界结构，管理所有实体、组件、资源和系统
///
/// 基于 Archetype 存储架构，将具有相同组件集合的实体组织在一起，
/// 实现缓存友好的行式存储。
pub struct World {
    /// 实体分配器
    entity_allocator: EntityAllocator,
    /// Archetype 图
    archetype_graph: ArchetypeGraph,
    /// 全局资源存储
    resources: HashMap<TypeId, Box<dyn Any + Send + Sync>>,
    /// 已注册的系统列表
    systems: Vec<Box<dyn System>>,
    /// 全局时钟，用于变更检测
    tick: u32,
    /// 变更检测记录：(Entity, TypeId) -> tick
    changed_ticks: HashMap<(Entity, TypeId), u32>,
}

impl World {
    /// 创建新的世界实例
    pub fn new() -> Self {
        Self {
            entity_allocator: EntityAllocator::new(),
            archetype_graph: ArchetypeGraph::new(),
            resources: HashMap::new(),
            systems: Vec::new(),
            tick: 0,
            changed_ticks: HashMap::new(),
        }
    }

    /// 创建新实体并返回实体构建器，用于链式添加组件
    pub fn spawn(&mut self) -> EntityBuilder<'_> {
        let entity = self.entity_allocator.allocate();
        let archetype_id = self.archetype_graph.empty_archetype();
        let archetype = self.archetype_graph.get_mut(archetype_id).unwrap();
        let row = archetype.add_entity(entity);

        let location = EntityLocation::new(archetype_id.0, row);
        self.entity_allocator.set_location(entity, location);

        EntityBuilder { world: self, entity }
    }

    /// 销毁指定实体及其所有组件
    pub fn despawn(&mut self, entity: Entity) -> GResult<()> {
        if !self.entity_allocator.is_alive(entity) {
            return Err(GError { kind: GErrorKind::Ecs, message: format!("Entity {} does not exist", entity) });
        }

        if let Some(location) = self.entity_allocator.destroy(entity) {
            let archetype_id = ArchetypeId::new(location.archetype_id);
            if let Some(archetype) = self.archetype_graph.get_mut(archetype_id) {
                if let Some(swapped) = archetype.remove_entity(location.row) {
                    self.entity_allocator.set_location(swapped.entity, EntityLocation::new(archetype_id.0, swapped.new_row));
                }
            }
        }

        self.changed_ticks.retain(|(e, _), _| *e != entity);
        Ok(())
    }

    /// 向指定实体添加组件
    ///
    /// 如果实体已有该类型组件，则覆盖。
    /// 添加组件会导致实体迁移到新的 Archetype。
    pub fn add_component<T: Component>(&mut self, entity: Entity, component: T) -> GResult<()> {
        let location = self
            .entity_allocator
            .get_location(entity)
            .ok_or_else(|| GError { kind: GErrorKind::Ecs, message: format!("Entity {} does not exist", entity) })?;

        let old_archetype_id = ArchetypeId::new(location.archetype_id);
        let type_id = TypeId::of::<T>();

        let has_component = self.archetype_graph.get(old_archetype_id).map(|a| a.has_component(type_id)).unwrap_or(false);

        if has_component {
            let archetype = self.archetype_graph.get_mut(old_archetype_id).unwrap();
            archetype.add_component(location.row, component);
        }
        else {
            let mut new_types =
                self.archetype_graph.get(old_archetype_id).map(|a| a.component_types().clone()).unwrap_or_default();
            new_types.insert(type_id);

            let new_archetype_id = self.archetype_graph.get_or_create(new_types);

            self.migrate_entity(entity, location, old_archetype_id, new_archetype_id, &[]);

            let archetype = self.archetype_graph.get_mut(new_archetype_id).unwrap();
            let new_location = self.entity_allocator.get_location(entity).unwrap();
            archetype.add_component(new_location.row, component);
        }

        self.mark_changed(entity, type_id);
        Ok(())
    }

    /// 添加类型擦除的组件
    ///
    /// 与 add_component 类似，但接受已装箱的类型擦除组件。
    /// 主要用于 PluginRegistrar 在 apply 阶段插入资源。
    pub fn add_component_raw(&mut self, entity: Entity, component: Box<dyn Any + Send + Sync>, type_id: TypeId) -> GResult<()> {
        let location = self
            .entity_allocator
            .get_location(entity)
            .ok_or_else(|| GError { kind: GErrorKind::Ecs, message: format!("Entity {} does not exist", entity) })?;

        let old_archetype_id = ArchetypeId::new(location.archetype_id);

        let has_component = self.archetype_graph.get(old_archetype_id).map(|a| a.has_component(type_id)).unwrap_or(false);

        if has_component {
            let archetype = self.archetype_graph.get_mut(old_archetype_id).unwrap();
            archetype.add_component_raw(location.row, component, type_id);
        }
        else {
            let mut new_types =
                self.archetype_graph.get(old_archetype_id).map(|a| a.component_types().clone()).unwrap_or_default();
            new_types.insert(type_id);

            let new_archetype_id = self.archetype_graph.get_or_create(new_types);
            self.migrate_entity(entity, location, old_archetype_id, new_archetype_id, &[]);

            let new_location = self.entity_allocator.get_location(entity).unwrap();
            let archetype = self.archetype_graph.get_mut(new_archetype_id).unwrap();
            archetype.add_component_raw(new_location.row, component, type_id);
        }

        self.mark_changed(entity, type_id);
        Ok(())
    }

    /// 获取指定实体的组件不可变引用
    pub fn get_component<T: Component>(&self, entity: Entity) -> Option<&T> {
        let location = self.entity_allocator.get_location(entity)?;
        let archetype_id = ArchetypeId::new(location.archetype_id);
        let archetype = self.archetype_graph.get(archetype_id)?;
        archetype.get_component(location.row)
    }

    /// 获取指定实体的组件可变引用
    pub fn get_component_mut<T: Component>(&mut self, entity: Entity) -> Option<&mut T> {
        let location = self.entity_allocator.get_location(entity)?;
        let archetype_id = ArchetypeId::new(location.archetype_id);
        let archetype = self.archetype_graph.get_mut(archetype_id)?;
        let result = archetype.get_component_mut(location.row);
        if result.is_some() {
            self.changed_ticks.insert((entity, TypeId::of::<T>()), self.tick);
        }
        result
    }

    /// 移除指定实体的组件并返回被移除的组件
    pub fn remove_component<T: Component>(&mut self, entity: Entity) -> Option<Box<T>> {
        let location = self.entity_allocator.get_location(entity)?;
        let old_archetype_id = ArchetypeId::new(location.archetype_id);
        let type_id = TypeId::of::<T>();

        {
            let archetype = self.archetype_graph.get(old_archetype_id)?;
            if !archetype.has_component(type_id) {
                return None;
            }
        }

        let component = {
            let archetype = self.archetype_graph.get_mut(old_archetype_id)?;
            archetype.get_component_mut::<T>(location.row).map(|c| unsafe { std::ptr::read(c as *const T) })
        };

        let new_types = {
            let archetype = self.archetype_graph.get(old_archetype_id)?;
            let mut types = archetype.component_types().clone();
            types.remove(&type_id);
            types
        };

        let new_archetype_id = self.archetype_graph.get_or_create(new_types);
        self.migrate_entity(entity, location, old_archetype_id, new_archetype_id, &[type_id]);

        self.changed_ticks.remove(&(entity, type_id));
        component.map(Box::new)
    }

    /// 插入全局资源，若已存在则覆盖
    pub fn insert_resource<T: Resource>(&mut self, resource: T) {
        let type_id = TypeId::of::<T>();
        self.resources.insert(type_id, Box::new(resource));
    }

    /// 插入类型擦除的全局资源
    ///
    /// 与 insert_resource 类似，但接受已装箱的类型擦除资源。
    /// 主要用于 PluginRegistrar 在 apply 阶段插入资源。
    pub fn insert_resource_raw(&mut self, resource: Box<dyn Any + Send + Sync>, type_id: TypeId) {
        self.resources.insert(type_id, resource);
    }

    /// 获取全局资源的不可变引用
    pub fn get_resource<T: Resource>(&self) -> Option<&T> {
        let type_id = TypeId::of::<T>();
        self.resources.get(&type_id).and_then(|r| r.downcast_ref::<T>())
    }

    /// 获取全局资源的可变引用
    pub fn get_resource_mut<T: Resource>(&mut self) -> Option<&mut T> {
        let type_id = TypeId::of::<T>();
        self.resources.get_mut(&type_id).and_then(|r| r.downcast_mut::<T>())
    }

    /// 移除全局资源并返回被移除的资源
    pub fn remove_resource<T: Resource + 'static>(&mut self) -> Option<Box<dyn Any + Send + Sync>> {
        let type_id = TypeId::of::<T>();
        self.resources.remove(&type_id)
    }

    /// 查询拥有指定组件类型的所有实体，返回查询迭代器
    pub fn query<T: Component>(&self) -> Query<'_, T, NoneFilter> {
        let type_id = TypeId::of::<T>();
        let iter = self.collect_component_iter(type_id);
        Query { inner: iter, world: self, _filter: PhantomData, _marker: PhantomData }
    }

    /// 查询拥有指定组件类型且满足过滤条件的所有实体
    pub fn query_filtered<T: Component, F: QueryFilter>(&self) -> Query<'_, T, F> {
        let type_id = TypeId::of::<T>();
        let iter = self.collect_component_iter(type_id);
        Query { inner: iter, world: self, _filter: PhantomData, _marker: PhantomData }
    }

    /// 多组件查询，使用 WorldQuery trait 支持元组查询
    ///
    /// 返回所有匹配 Archetype 中满足查询条件的实体和组件数据。
    pub fn query_multi<Q: query::WorldQuery>(&self) -> MultiQuery<'_, Q> {
        let mut matching_archetypes: Vec<(ArchetypeId, Q::State)> = Vec::new();
        for archetype in self.archetype_graph.iter() {
            let state = Q::init_state(archetype);
            if Q::matches_archetype(&state, archetype) {
                matching_archetypes.push((archetype.id(), state));
            }
        }
        MultiQuery { world: self, matching_archetypes, current_archetype_idx: 0, current_row: 0, _marker: PhantomData }
    }

    /// 注册系统到世界
    pub fn register_system(&mut self, system: Box<dyn System>) {
        self.systems.push(system);
    }

    /// 按注册顺序执行所有系统
    pub fn run_systems(&mut self) -> GResult<()> {
        let mut systems = std::mem::take(&mut self.systems);
        for system in &mut systems {
            system.execute(self)?;
        }
        self.systems = systems;
        self.increment_tick();
        Ok(())
    }

    /// 获取所有存活实体的集合
    pub fn entities(&self) -> Vec<Entity> {
        self.entity_allocator.iter_alive().collect()
    }

    /// 检查实体是否存在
    pub fn contains_entity(&self, entity: Entity) -> bool {
        self.entity_allocator.is_alive(entity)
    }

    /// 获取当前时钟值
    pub fn tick(&self) -> u32 {
        self.tick
    }

    /// 递增时钟
    pub fn increment_tick(&mut self) {
        self.tick = self.tick.wrapping_add(1);
    }

    /// 获取实体分配器引用
    pub fn entity_allocator(&self) -> &EntityAllocator {
        &self.entity_allocator
    }

    /// 获取 Archetype 图引用
    pub fn archetype_graph(&self) -> &ArchetypeGraph {
        &self.archetype_graph
    }

    /// 获取 Archetype 图可变引用
    pub fn archetype_graph_mut(&mut self) -> &mut ArchetypeGraph {
        &mut self.archetype_graph
    }

    /// 将实体从旧 Archetype 迁移到新 Archetype
    ///
    /// 迁移过程中会正确拷贝所有现有组件数据到新 Archetype，
    /// 并更新被 swap-remove 影响的实体的 EntityLocation。
    fn migrate_entity(
        &mut self,
        entity: Entity,
        old_location: EntityLocation,
        old_archetype_id: ArchetypeId,
        new_archetype_id: ArchetypeId,
        exclude_types: &[TypeId],
    ) {
        if old_archetype_id == new_archetype_id {
            return;
        }

        let old_archetype = self.archetype_graph.get_mut(old_archetype_id).unwrap();
        let old_row = old_location.row;

        let mut component_data: Vec<(TypeId, Vec<u8>)> = Vec::new();
        let mut column_meta: Vec<(TypeId, &'static str, usize, usize, Option<unsafe fn(*mut u8, usize)>)> = Vec::new();
        for column in old_archetype.storage().iter_columns() {
            let type_id = column.type_id();
            if exclude_types.contains(&type_id) {
                continue;
            }
            column_meta.push((type_id, column.type_name(), column.size(), column.align(), column.drop_fn()));
            if let Some((ptr, size)) = unsafe { column.read_raw(old_row) } {
                let mut data = vec![0u8; size];
                unsafe {
                    std::ptr::copy_nonoverlapping(ptr, data.as_mut_ptr(), size);
                }
                component_data.push((type_id, data));
            }
        }

        if let Some(swapped) = old_archetype.remove_entity(old_row) {
            self.entity_allocator.set_location(swapped.entity, EntityLocation::new(old_archetype_id.0, swapped.new_row));
        }

        let new_archetype = self.archetype_graph.get_mut(new_archetype_id).unwrap();

        for (type_id, type_name, size, align, drop_fn) in column_meta {
            if !new_archetype.storage().contains(type_id) {
                new_archetype.storage_mut().add_column_raw(type_id, type_name, size, align, drop_fn);
            }
        }

        let new_row = new_archetype.add_entity(entity);

        for (type_id, data) in component_data {
            if let Some(column) = new_archetype.storage_mut().get_column_mut(type_id) {
                column.ensure_len(new_row + 1);
                unsafe {
                    let dst = column.data_as_mut_ptr().add(new_row * data.len());
                    std::ptr::copy_nonoverlapping(data.as_ptr(), dst, data.len());
                }
            }
        }

        let new_location = EntityLocation::new(new_archetype_id.0, new_row);
        self.entity_allocator.set_location(entity, new_location);
    }

    /// 标记组件已变更
    fn mark_changed(&mut self, entity: Entity, type_id: TypeId) {
        self.changed_ticks.insert((entity, type_id), self.tick);
    }

    /// 收集所有 Archetype 中指定类型组件的迭代器
    fn collect_component_iter<T: Component>(&self, type_id: TypeId) -> Option<storage::ComponentColumnIter<'_, T>> {
        let mut entries: Vec<(Entity, *const T)> = Vec::new();
        for archetype in self.archetype_graph.iter() {
            if !archetype.has_component(type_id) {
                continue;
            }
            if let Some(column) = archetype.storage().get_column(type_id) {
                for row in 0..archetype.len() {
                    if let Some(entity) = archetype.get_entity(row) {
                        if let Some(component) = column.get::<T>(row) {
                            entries.push((entity, component as *const T));
                        }
                    }
                }
            }
        }
        Some(storage::ComponentColumnIter::new(entries))
    }
}

impl Default for World {
    fn default() -> Self {
        Self::new()
    }
}

/// 预导入模块，包含 ECS 核心类型
pub mod prelude {
    pub use crate::{
        Changed, Component, EntityBuilder, MultiQuery, NoneFilter, Query, QueryFilter, Resource, System, With, Without, World,
        archetype::{ArchetypeGraph, ArchetypeId},
        entity::{Entity, EntityAllocator, EntityLocation},
        query::{QueryResult, WorldQuery},
        storage::{ComponentColumn, ComponentStorage},
    };
}
