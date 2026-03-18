//! GWG Engine ECS 核心实现
//!
//! 提供基于 Archetype 的高性能实体组件系统。

mod archetype;
mod entity;
mod query;
mod resource;
mod storage;
mod world;

pub use archetype::{Archetype, ArchetypeGraph, ArchetypeId};
pub use entity::{EntityAllocator, EntityLocation};
pub use query::{Query, QueryIter, QueryMut, QueryState, WorldQuery};
pub use resource::{ResourceRef, ResourceRefMut, Resources};
pub use storage::{ComponentColumn, ComponentStorage};
pub use world::{EntityMut, EntityRef, World};

pub use gwg_types::prelude::*;

pub mod prelude {
    //! ECS 核心的预导入模块

    pub use gwg_types::prelude::*;
    pub use crate::{
        Archetype, ArchetypeGraph, ArchetypeId,
        EntityAllocator, EntityLocation,
        Query, QueryIter, QueryMut, QueryState, WorldQuery,
        ResourceRef, ResourceRefMut, Resources,
        ComponentColumn, ComponentStorage,
        EntityMut, EntityRef, World,
    };
}
