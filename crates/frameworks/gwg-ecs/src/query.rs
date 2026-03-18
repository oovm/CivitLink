//! Query 系统
//!
//! 提供高效的组件查询机制。

use std::any::TypeId;
use std::marker::PhantomData;

use gwg_types::prelude::*;
use crate::archetype::{Archetype, ArchetypeGraph, ArchetypeId};

/// World Query trait，定义可以从世界查询的数据类型
pub trait WorldQuery {
    /// 查询状态
    type State;

    /// 初始化查询状态
    fn init_state(world: &crate::World) -> Self::State;

    /// 获取查询所需的组件类型
    fn component_types(state: &Self::State) -> Vec<TypeId>;

    /// 检查 Archetype 是否匹配
    fn matches_archetype(state: &Self::State, archetype: &Archetype) -> bool;
}

/// 单个组件的查询状态
pub struct ComponentQueryState {
    type_id: TypeId,
}

impl<T: Component> WorldQuery for &T {
    type State = ComponentQueryState;

    fn init_state(_world: &crate::World) -> Self::State {
        ComponentQueryState {
            type_id: TypeId::of::<T>(),
        }
    }

    fn component_types(state: &Self::State) -> Vec<TypeId> {
        vec![state.type_id]
    }

    fn matches_archetype(state: &Self::State, archetype: &Archetype) -> bool {
        archetype.has_component(state.type_id)
    }
}

impl<T: Component> WorldQuery for &mut T {
    type State = ComponentQueryState;

    fn init_state(_world: &crate::World) -> Self::State {
        ComponentQueryState {
            type_id: TypeId::of::<T>(),
        }
    }

    fn component_types(state: &Self::State) -> Vec<TypeId> {
        vec![state.type_id]
    }

    fn matches_archetype(state: &Self::State, archetype: &Archetype) -> bool {
        archetype.has_component(state.type_id)
    }
}

/// 元组查询实现
impl<A: WorldQuery, B: WorldQuery> WorldQuery for (A, B) {
    type State = (A::State, B::State);

    fn init_state(world: &crate::World) -> Self::State {
        (A::init_state(world), B::init_state(world))
    }

    fn component_types(state: &Self::State) -> Vec<TypeId> {
        let mut types = A::component_types(&state.0);
        types.extend(B::component_types(&state.1));
        types
    }

    fn matches_archetype(state: &Self::State, archetype: &Archetype) -> bool {
        A::matches_archetype(&state.0, archetype) && B::matches_archetype(&state.1, archetype)
    }
}

/// 查询状态
pub struct QueryState<Q: WorldQuery> {
    /// 查询内部状态
    inner: Q::State,
    /// 匹配的 Archetype ID 列表
    matched_archetypes: Vec<ArchetypeId>,
    /// 查询所需的组件类型
    component_types: Vec<TypeId>,
}

impl<Q: WorldQuery> QueryState<Q> {
    /// 创建新的查询状态
    pub fn new(world: &crate::World) -> Self {
        let inner = Q::init_state(world);
        let component_types = Q::component_types(&inner);
        Self {
            inner,
            matched_archetypes: Vec::new(),
            component_types,
        }
    }

    /// 更新匹配的 Archetype 列表
    pub fn update_archetypes(&mut self, graph: &ArchetypeGraph) {
        self.matched_archetypes.clear();
        for archetype in graph.iter() {
            if Q::matches_archetype(&self.inner, archetype) {
                self.matched_archetypes.push(archetype.id());
            }
        }
    }

    /// 获取匹配的 Archetype 列表
    pub fn matched_archetypes(&self) -> &[ArchetypeId] {
        &self.matched_archetypes
    }

    /// 获取组件类型
    pub fn component_types(&self) -> &[TypeId] {
        &self.component_types
    }
}

/// 查询迭代器
pub struct QueryIter<'w, T: Component> {
    /// 世界引用
    world: &'w crate::World,
    /// 当前 Archetype 索引
    archetype_index: usize,
    /// 当前实体索引
    entity_index: usize,
    /// 匹配的 Archetype ID 列表
    matched_archetypes: Vec<ArchetypeId>,
    _marker: PhantomData<T>,
}

impl<'w, T: Component> QueryIter<'w, T> {
    /// 创建新的查询迭代器
    pub fn new(world: &'w crate::World, matched_archetypes: Vec<ArchetypeId>) -> Self {
        Self {
            world,
            archetype_index: 0,
            entity_index: 0,
            matched_archetypes,
            _marker: PhantomData,
        }
    }
}

impl<'w, T: Component> Iterator for QueryIter<'w, T> {
    type Item = (Entity, &'w T);

    fn next(&mut self) -> Option<Self::Item> {
        while self.archetype_index < self.matched_archetypes.len() {
            let archetype_id = self.matched_archetypes[self.archetype_index];
            let archetype = self.world.archetype_graph().get(archetype_id)?;

            if self.entity_index < archetype.len() {
                let entity = archetype.get_entity(self.entity_index)?;
                let component = archetype.get_component::<T>(self.entity_index)?;
                self.entity_index += 1;
                return Some((entity, component));
            }

            self.archetype_index += 1;
            self.entity_index = 0;
        }
        None
    }
}

/// 查询对象
pub struct Query<'w, Q: WorldQuery> {
    /// 世界引用
    world: &'w crate::World,
    /// 查询状态
    state: QueryState<Q>,
}

impl<'w, Q: WorldQuery> Query<'w, Q> {
    /// 创建新的查询
    pub fn new(world: &'w crate::World) -> Self {
        let mut state = QueryState::new(world);
        state.update_archetypes(world.archetype_graph());
        Self { world, state }
    }

    /// 获取查询状态
    pub fn state(&self) -> &QueryState<Q> {
        &self.state
    }

    /// 更新查询状态
    pub fn update(&mut self) {
        self.state.update_archetypes(self.world.archetype_graph());
    }
}

impl<'w, T: Component> Query<'w, &T> {
    /// 迭代所有匹配的实体和组件
    pub fn iter(&self) -> QueryIter<'_, T> {
        QueryIter::new(
            self.world,
            self.state.matched_archetypes.clone(),
        )
    }
}

/// 可变查询对象
pub struct QueryMut<'w, Q: WorldQuery> {
    /// 世界可变引用
    world: &'w mut crate::World,
    /// 查询状态
    state: QueryState<Q>,
}

impl<'w, Q: WorldQuery> QueryMut<'w, Q> {
    /// 创建新的可变查询
    pub fn new(world: &'w mut crate::World) -> Self {
        let mut state = QueryState::new(world);
        state.update_archetypes(world.archetype_graph());
        Self { world, state }
    }

    /// 对每个匹配的组件执行操作
    pub fn for_each<T: Component, F>(&mut self, mut f: F)
    where
        F: FnMut(Entity, &mut T),
    {
        for archetype_id in self.state.matched_archetypes.clone() {
            if let Some(archetype) = self.world.archetype_graph_mut().get_mut(archetype_id) {
                for row in 0..archetype.len() {
                    if let Some(entity) = archetype.get_entity(row) {
                        if let Some(component) = archetype.get_component_mut::<T>(row) {
                            f(entity, component);
                        }
                    }
                }
            }
        }
    }
}
