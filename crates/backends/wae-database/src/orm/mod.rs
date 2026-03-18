//! ORM 核心模块
//!
//! 提供轻量级 ORM 功能，支持：
//! - 实体定义和映射
//! - 查询构建器
//! - CRUD 操作封装
//! - 一对多和多对多关系

#[cfg(any(feature = "limbo", feature = "postgres", feature = "mysql"))]
mod builder;
#[cfg(any(feature = "limbo", feature = "postgres", feature = "mysql"))]
mod condition;
#[cfg(any(feature = "limbo", feature = "postgres", feature = "mysql"))]
mod entity;
#[cfg(any(feature = "limbo", feature = "postgres", feature = "mysql"))]
mod macros;
#[cfg(any(feature = "limbo", feature = "postgres", feature = "mysql"))]
mod repository;

#[cfg(any(feature = "limbo", feature = "postgres", feature = "mysql"))]
pub use builder::{DeleteBuilder, InsertBuilder, Join, JoinType, QueryBuilder, SelectBuilder, UpdateBuilder};
#[cfg(any(feature = "limbo", feature = "postgres", feature = "mysql"))]
pub use condition::Condition;
#[cfg(any(feature = "limbo", feature = "postgres", feature = "mysql"))]
pub use entity::{BelongsTo, Entity, FromRow, HasMany, ManyToMany, ToRow};
#[cfg(feature = "mysql")]
pub use repository::MySqlDbRepository;
#[cfg(any(feature = "limbo", feature = "postgres"))]
pub use repository::{DbRepository, Repository};
