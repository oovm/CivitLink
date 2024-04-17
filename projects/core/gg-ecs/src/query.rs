#![warn(missing_docs)]

//! 查询模块
//!
//! 提供 WorldQuery trait 和多组件查询支持，
//! 允许通过元组语法同时查询多个组件类型。

use std::any::TypeId;

use crate::{Component, archetype::Archetype};

/// 世界查询 trait，定义可以从世界查询的数据类型
///
/// 实现此 trait 的类型可以作为 `Query<Q>` 的类型参数，
/// 支持单组件、多组件元组等查询形式。
pub trait WorldQuery {
    /// 查询状态，在初始化时创建
    type State;

    /// 初始化查询状态
    fn init_state(archetype: &Archetype) -> Self::State;

    /// 获取查询所需的所有组件类型 ID
    fn component_types(state: &Self::State) -> Vec<TypeId>;

    /// 检查 Archetype 是否匹配此查询
    fn matches_archetype(state: &Self::State, archetype: &Archetype) -> bool;
}

/// 不可变组件引用查询
impl<T: Component> WorldQuery for &T {
    type State = TypeId;

    fn init_state(_archetype: &Archetype) -> Self::State {
        TypeId::of::<T>()
    }

    fn component_types(state: &Self::State) -> Vec<TypeId> {
        vec![*state]
    }

    fn matches_archetype(state: &Self::State, archetype: &Archetype) -> bool {
        archetype.has_component(*state)
    }
}

/// 可变组件引用查询
impl<T: Component> WorldQuery for &mut T {
    type State = TypeId;

    fn init_state(_archetype: &Archetype) -> Self::State {
        TypeId::of::<T>()
    }

    fn component_types(state: &Self::State) -> Vec<TypeId> {
        vec![*state]
    }

    fn matches_archetype(state: &Self::State, archetype: &Archetype) -> bool {
        archetype.has_component(*state)
    }
}

/// 二元组查询，支持同时查询两个组件类型
impl<A: WorldQuery, B: WorldQuery> WorldQuery for (A, B) {
    type State = (A::State, B::State);

    fn init_state(archetype: &Archetype) -> Self::State {
        (A::init_state(archetype), B::init_state(archetype))
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

/// 三元组查询
impl<A: WorldQuery, B: WorldQuery, C: WorldQuery> WorldQuery for (A, B, C) {
    type State = (A::State, B::State, C::State);

    fn init_state(archetype: &Archetype) -> Self::State {
        (A::init_state(archetype), B::init_state(archetype), C::init_state(archetype))
    }

    fn component_types(state: &Self::State) -> Vec<TypeId> {
        let mut types = A::component_types(&state.0);
        types.extend(B::component_types(&state.1));
        types.extend(C::component_types(&state.2));
        types
    }

    fn matches_archetype(state: &Self::State, archetype: &Archetype) -> bool {
        A::matches_archetype(&state.0, archetype)
            && B::matches_archetype(&state.1, archetype)
            && C::matches_archetype(&state.2, archetype)
    }
}

/// 四元组查询
impl<A: WorldQuery, B: WorldQuery, C: WorldQuery, D: WorldQuery> WorldQuery for (A, B, C, D) {
    type State = (A::State, B::State, C::State, D::State);

    fn init_state(archetype: &Archetype) -> Self::State {
        (A::init_state(archetype), B::init_state(archetype), C::init_state(archetype), D::init_state(archetype))
    }

    fn component_types(state: &Self::State) -> Vec<TypeId> {
        let mut types = A::component_types(&state.0);
        types.extend(B::component_types(&state.1));
        types.extend(C::component_types(&state.2));
        types.extend(D::component_types(&state.3));
        types
    }

    fn matches_archetype(state: &Self::State, archetype: &Archetype) -> bool {
        A::matches_archetype(&state.0, archetype)
            && B::matches_archetype(&state.1, archetype)
            && C::matches_archetype(&state.2, archetype)
            && D::matches_archetype(&state.3, archetype)
    }
}

/// 多组件查询结果 trait，从 Archetype 行中提取组件数据
///
/// 当前仅支持不可变组件引用，可变查询需要通过 GgWorld::get_component_mut 实现。
pub trait QueryResult<'a>: Sized {
    /// 从 Archetype 的指定行提取查询结果
    fn fetch(archetype: &'a Archetype, row: usize) -> Option<Self>;
}

impl<'a, T: Component> QueryResult<'a> for &'a T {
    fn fetch(archetype: &'a Archetype, row: usize) -> Option<Self> {
        archetype.get_component(row)
    }
}

impl<'a, A: QueryResult<'a>, B: QueryResult<'a>> QueryResult<'a> for (A, B) {
    fn fetch(archetype: &'a Archetype, row: usize) -> Option<Self> {
        Some((A::fetch(archetype, row)?, B::fetch(archetype, row)?))
    }
}

impl<'a, A: QueryResult<'a>, B: QueryResult<'a>, C: QueryResult<'a>> QueryResult<'a> for (A, B, C) {
    fn fetch(archetype: &'a Archetype, row: usize) -> Option<Self> {
        Some((A::fetch(archetype, row)?, B::fetch(archetype, row)?, C::fetch(archetype, row)?))
    }
}

impl<'a, A: QueryResult<'a>, B: QueryResult<'a>, C: QueryResult<'a>, D: QueryResult<'a>> QueryResult<'a> for (A, B, C, D) {
    fn fetch(archetype: &'a Archetype, row: usize) -> Option<Self> {
        Some((A::fetch(archetype, row)?, B::fetch(archetype, row)?, C::fetch(archetype, row)?, D::fetch(archetype, row)?))
    }
}
