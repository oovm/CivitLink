//! GWG Engine ECS 核心实现
//!
//! 提供基于 Archetype 的高性能实体组件系统。

mod world;
mod archetype;
mod storage;
mod entity;
mod query;
mod resource;

pub use world::World;
pub use archetype::Archetype;
pub use storage::ComponentStorage;
pub use entity::{EntityAllocator, EntityLocation};
pub use query::{Query, QueryIter, QueryState, WorldQuery};
pub use resource::{Resources, ResourceRef, ResourceRefMut};

pub use gwg_types::prelude::*;

pub mod prelude {
    //! ECS 核心的预导入模块

    pub use gwg_types::prelude::*;
    pub use super::{
        World, Archetype, ComponentStorage, EntityAllocator, EntityLocation,
        Query, QueryIter, QueryState, WorldQuery, Resources, ResourceRef, ResourceRefMut,
    };
}
