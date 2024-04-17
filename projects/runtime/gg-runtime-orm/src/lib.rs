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

/// JOIN 类型
///
/// 定义查询中支持的 JOIN 操作类型。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JoinKind {
    /// 内连接（INNER JOIN）
    Inner,
    /// 左外连接（LEFT JOIN）
    Left,
    /// 右外连接（RIGHT JOIN）
    Right,
}

/// 聚合函数
///
/// 定义查询中支持的 SQL 聚合函数类型。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AggregateFunction {
    /// 计数（COUNT）
    Count,
    /// 求和（SUM）
    Sum,
    /// 平均值（AVG）
    Avg,
    /// 最小值（MIN）
    Min,
    /// 最大值（MAX）
    Max,
}

/// 分页参数
///
/// 包含分页查询所需的页码和每页大小。
#[derive(Debug, Clone)]
pub struct Pagination {
    /// 当前页码（从 1 开始）
    pub page: usize,
    /// 每页记录数
    pub per_page: usize,
}

impl Pagination {
    /// 创建新的分页参数
    ///
    /// # 参数
    ///
    /// - `page`: 当前页码（从 1 开始）
    /// - `per_page`: 每页记录数
    pub fn new(page: usize, per_page: usize) -> Self {
        Self { page: page.max(1), per_page: per_page.max(1) }
    }

    /// 计算 SQL OFFSET 值
    pub fn offset(&self) -> usize {
        (self.page - 1) * self.per_page
    }

    /// 获取 LIMIT 值
    pub fn limit(&self) -> usize {
        self.per_page
    }
}

/// 分页查询结果
///
/// 包含分页查询的数据和分页元信息。
#[derive(Debug, Clone)]
pub struct PaginatedResult<T> {
    /// 当前页数据
    pub data: Vec<T>,
    /// 分页参数
    pub pagination: Pagination,
    /// 总记录数
    pub total: usize,
    /// 总页数
    pub total_pages: usize,
}

impl<T> PaginatedResult<T> {
    /// 创建新的分页结果
    ///
    /// # 参数
    ///
    /// - `data`: 当前页数据
    /// - `pagination`: 分页参数
    /// - `total`: 总记录数
    pub fn new(data: Vec<T>, pagination: Pagination, total: usize) -> Self {
        let total_pages = if pagination.per_page > 0 {
            (total + pagination.per_page - 1) / pagination.per_page
        }
        else {
            0
        };
        Self { data, pagination, total, total_pages }
    }

    /// 判断是否为第一页
    pub fn is_first_page(&self) -> bool {
        self.pagination.page == 1
    }

    /// 判断是否为最后一页
    pub fn is_last_page(&self) -> bool {
        self.pagination.page >= self.total_pages
    }
}

/// 查询构建器模块
pub mod query_builder;

/// 仓库模式模块
pub mod repository;

pub use query_builder::{FilterCondition, JoinClause, HavingCondition, OrderBy, QueryBuilder};
pub use repository::{EntityMapper, Repository};
