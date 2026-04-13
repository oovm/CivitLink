#![warn(missing_docs)]

//! GG 引擎 ORM 模块
//!
//! 提供对象关系映射抽象，支持 Repository 模式和 QueryBuilder。
//! 通过 QueryBuilder 构建类型安全的 SQL 查询，
//! 通过 Repository 封装数据库的 CRUD 操作。

pub use gg_core::{GError, GErrorKind, GResult};

/// 过滤操作符
///
/// 定义查询条件中支持的比较操作类型，
/// 用于 QueryBuilder 的 WHERE 子句构建。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FilterOp {
    /// 等于（=）
    Eq,
    /// 不等于（<>）
    Ne,
    /// 小于（<）
    Lt,
    /// 小于等于（<=）
    Le,
    /// 大于（>）
    Gt,
    /// 大于等于（>=）
    Ge,
    /// 模糊匹配（LIKE）
    Like,
    /// 包含于（IN）
    In,
}

/// 查询构建器模块
pub mod query_builder;

/// 仓库模式模块
pub mod repository;

pub use query_builder::{FilterCondition, OrderBy, QueryBuilder};
pub use repository::{EntityMapper, Repository};
