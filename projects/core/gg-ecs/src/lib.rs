#![warn(missing_docs)]

//! GG 引擎 ECS 核心模块
//! 提供实体-组件-系统架构，包含实体管理、组件存储、资源管理和查询功能

use gg_error::{GError, GErrorKind, GResult};
use std::{
    any::{Any, TypeId},
    collections::{HashMap, HashSet},
    marker::PhantomData,
};

/// 实体 ID 类型
pub type Entity = u64;

/// World 类型别名
pub type World = GgWorld;

/// 组件 trait，所有组件必须实现此 trait
pub trait Component: Any + Send + Sync {
    /// 获取组件类型 ID
    fn type_id(&self) -> TypeId {
        TypeId::of::<Self>()
    }
}

/// 资源 trait，全局单例数据必须实现此 trait
pub trait Resource: Any + Send + Sync {}

/// 系统 trait，所有系统必须实现此 trait
pub trait System {
    /// 获取系统名称
    fn name(&self) -> &str;

    /// 执行系统逻辑
    fn execute(&mut self, world: &mut GgWorld) -> GResult<()>;
}

/// 查询过滤器 trait，用于在查询时过滤实体
pub trait QueryFilter {
    /// 检查实体是否满足过滤条件
    fn matches(world: &GgWorld, entity: Entity) -> bool;
}

/// 无过滤器的查询，匹配所有实体
pub struct NoneFilter;

impl QueryFilter for NoneFilter {
    fn matches(_world: &GgWorld, _entity: Entity) -> bool {
        true
    }
}

/// 查询过滤器：仅匹配包含指定组件的实体
pub struct With<T: Component> {
    /// 类型标记
    _marker: PhantomData<T>,
}

impl<T: Component> QueryFilter for With<T> {
    fn matches(world: &GgWorld, entity: Entity) -> bool {
        world.get_component::<T>(entity).is_some()
    }
}

/// 查询过滤器：仅匹配不包含指定组件的实体
pub struct Without<T: Component> {
    /// 类型标记
    _marker: PhantomData<T>,
}

impl<T: Component> QueryFilter for Without<T> {
    fn matches(world: &GgWorld, entity: Entity) -> bool {
        world.get_component::<T>(entity).is_none()
    }
}

/// 组件查询迭代器，用于遍历拥有指定组件类型且满足过滤条件的所有实体
pub struct Query<'a, T: Component, F: QueryFilter = NoneFilter> {
    /// 底层组件存储迭代器
    inner: Option<std::collections::hash_map::Iter<'a, Entity, Box<dyn Any + Send + Sync>>>,
    /// 世界引用，用于过滤检查
    world: &'a GgWorld,
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
            if let Some(typed) = component.downcast_ref::<T>() {
                if F::matches(self.world, *entity) {
                    return Some((*entity, typed));
                }
            }
        }
    }
}

/// 组件存储，按组件类型隔离存储
struct ComponentStore {
    /// 实体到组件的映射
    components: HashMap<Entity, Box<dyn Any + Send + Sync>>,
}

impl ComponentStore {
    /// 创建新的组件存储
    fn new() -> Self {
        Self { components: HashMap::new() }
    }

    /// 添加组件到指定实体
    fn add(&mut self, entity: Entity, component: Box<dyn Any + Send + Sync>) {
        self.components.insert(entity, component);
    }

    /// 获取指定实体的组件引用
    fn get<T: Component>(&self, entity: Entity) -> Option<&T> {
        self.components.get(&entity).and_then(|c| c.downcast_ref::<T>())
    }

    /// 获取指定实体的组件可变引用
    fn get_mut<T: Component>(&mut self, entity: Entity) -> Option<&mut T> {
        self.components.get_mut(&entity).and_then(|c| c.downcast_mut::<T>())
    }

    /// 移除指定实体的组件
    fn remove(&mut self, entity: Entity) -> Option<Box<dyn Any + Send + Sync>> {
        self.components.remove(&entity)
    }

    /// 获取组件迭代器
    fn iter(&self) -> std::collections::hash_map::Iter<'_, Entity, Box<dyn Any + Send + Sync>> {
        self.components.iter()
    }
}

/// 实体构建器，用于链式创建实体并添加组件
pub struct EntityBuilder<'a> {
    /// 世界可变引用
    world: &'a mut GgWorld,
    /// 实体 ID
    entity: Entity,
}

impl<'a> EntityBuilder<'a> {
    /// 向实体插入组件，返回自身以支持链式调用
    pub fn insert<T: Component>(self, component: T) -> Self {
        let type_id = TypeId::of::<T>();
        let store = self.world.component_stores.entry(type_id).or_insert_with(ComponentStore::new);
        store.add(self.entity, Box::new(component));
        self
    }

    /// 获取当前构建实体的 ID
    pub fn id(&self) -> Entity {
        self.entity
    }
}

/// GG 引擎世界结构，管理所有实体、组件、资源和系统
pub struct GgWorld {
    /// 下一个可分配的实体 ID
    next_entity: Entity,
    /// 按组件类型索引的组件存储
    component_stores: HashMap<TypeId, ComponentStore>,
    /// 活动实体集合
    entities: HashSet<Entity>,
    /// 全局资源存储
    resources: HashMap<TypeId, Box<dyn Any + Send + Sync>>,
    /// 已注册的系统列表
    systems: Vec<Box<dyn System>>,
}

impl GgWorld {
    /// 创建新的世界实例
    pub fn new() -> Self {
        Self {
            next_entity: 0,
            component_stores: HashMap::new(),
            entities: HashSet::new(),
            resources: HashMap::new(),
            systems: Vec::new(),
        }
    }

    /// 创建新实体并返回实体构建器，用于链式添加组件
    pub fn spawn(&mut self) -> EntityBuilder<'_> {
        let entity = self.next_entity;
        self.next_entity += 1;
        self.entities.insert(entity);
        EntityBuilder { world: self, entity }
    }

    /// 销毁指定实体及其所有组件
    pub fn despawn(&mut self, entity: Entity) -> GResult<()> {
        if !self.entities.contains(&entity) {
            return Err(GError { kind: GErrorKind::Ecs, message: format!("Entity {} does not exist", entity) });
        }

        for store in self.component_stores.values_mut() {
            store.remove(entity);
        }

        self.entities.remove(&entity);
        Ok(())
    }

    /// 向指定实体添加组件
    pub fn add_component<T: Component>(&mut self, entity: Entity, component: T) -> GResult<()> {
        if !self.entities.contains(&entity) {
            return Err(GError { kind: GErrorKind::Ecs, message: format!("Entity {} does not exist", entity) });
        }

        let type_id = TypeId::of::<T>();
        let store = self.component_stores.entry(type_id).or_insert_with(ComponentStore::new);
        store.add(entity, Box::new(component));
        Ok(())
    }

    /// 添加类型擦除的组件
    ///
    /// 与 add_component 类似，但接受已装箱的类型擦除组件。
    /// 主要用于 PluginRegistrar 在 apply 阶段插入资源。
    pub fn add_component_raw(&mut self, entity: Entity, component: Box<dyn Any + Send + Sync>, type_id: TypeId) -> GResult<()> {
        if !self.entities.contains(&entity) {
            return Err(GError { kind: GErrorKind::Ecs, message: format!("Entity {} does not exist", entity) });
        }
        let store = self.component_stores.entry(type_id).or_insert_with(ComponentStore::new);
        store.add(entity, component);
        Ok(())
    }

    /// 获取指定实体的组件不可变引用
    pub fn get_component<T: Component>(&self, entity: Entity) -> Option<&T> {
        let type_id = TypeId::of::<T>();
        self.component_stores.get(&type_id).and_then(|store| store.get::<T>(entity))
    }

    /// 获取指定实体的组件可变引用
    pub fn get_component_mut<T: Component>(&mut self, entity: Entity) -> Option<&mut T> {
        let type_id = TypeId::of::<T>();
        self.component_stores.get_mut(&type_id).and_then(|store| store.get_mut::<T>(entity))
    }

    /// 移除指定实体的组件并返回被移除的组件
    pub fn remove_component<T: Component>(&mut self, entity: Entity) -> Option<Box<T>> {
        let type_id = TypeId::of::<T>();
        self.component_stores.get_mut(&type_id).and_then(|store| store.remove(entity)).and_then(|c| c.downcast::<T>().ok())
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
        let inner = self.component_stores.get(&type_id).map(|store| store.iter());
        Query { inner, world: self, _filter: PhantomData, _marker: PhantomData }
    }

    /// 查询拥有指定组件类型且满足过滤条件的所有实体
    pub fn query_filtered<T: Component, F: QueryFilter>(&self) -> Query<'_, T, F> {
        let type_id = TypeId::of::<T>();
        let inner = self.component_stores.get(&type_id).map(|store| store.iter());
        Query { inner, world: self, _filter: PhantomData, _marker: PhantomData }
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
        Ok(())
    }

    /// 获取所有活动实体的集合引用
    pub fn entities(&self) -> &HashSet<Entity> {
        &self.entities
    }
}
