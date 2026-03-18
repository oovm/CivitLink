//! 实体定义模块
//!
//! 提供实体与数据库表的映射关系

use crate::{
    DatabaseResult,
    connection::{DatabaseRow, FromDatabaseValue},
};
use wae_types::Value;

/// 数据库实体 trait
///
/// 定义实体与数据库表的映射关系
///
/// # Example
///
/// ```ignore
/// struct User {
///     id: i64,
///     name: String,
///     email: String,
/// }
///
/// impl Entity for User {
///     type Id = i64;
///
///     fn table_name() -> &'static str {
///         "users"
///     }
///
///     fn id(&self) -> Self::Id {
///         self.id
///     }
/// }
/// ```
pub trait Entity: Sized + Send + Sync + 'static {
    /// 主键类型
    type Id: FromDatabaseValue + Into<Value> + Send + Sync + Clone;

    /// 表名
    fn table_name() -> &'static str;

    /// 获取主键值
    fn id(&self) -> Self::Id;

    /// 主键列名，默认为 "id"
    fn id_column() -> &'static str {
        "id"
    }
}

/// 从数据库行解析实体
pub trait FromRow: Sized {
    /// 从数据库行解析
    fn from_row(row: &DatabaseRow) -> DatabaseResult<Self>;
}

/// 将实体转换为数据库行
pub trait ToRow {
    /// 转换为列名和值的列表
    fn to_row(&self) -> Vec<(&'static str, Value)>;
}

/// 一对多关系 trait
///
/// 定义主实体与从实体的一对多关系
///
/// # Example
///
/// ```ignore
/// struct User {
///     id: i64,
///     posts: Vec<Post>,
/// }
///
/// struct Post {
///     id: i64,
///     user_id: i64,
/// }
///
/// impl HasMany<Post> for User {
///     fn foreign_key() -> &'static str {
///         "user_id"
///     }
///
///     fn local_key() -> &'static str {
///         "id"
///     }
/// }
/// ```
pub trait HasMany<T: Entity>: Entity {
    /// 外键列名（从表中的列名）
    fn foreign_key() -> &'static str;

    /// 本地键列名（主表中的列名），默认为主键列名
    fn local_key() -> &'static str {
        Self::id_column()
    }
}

/// 一对多关系的反向 trait
///
/// 定义从实体与主实体的多对一关系
///
/// # Example
///
/// ```ignore
/// struct Post {
///     id: i64,
///     user_id: i64,
///     user: Option<User>,
/// }
///
/// impl BelongsTo<User> for Post {
///     fn foreign_key() -> &'static str {
///         "user_id"
///     }
///
///     fn owner_key() -> &'static str {
///         "id"
///     }
/// }
/// ```
pub trait BelongsTo<T: Entity>: Entity {
    /// 外键列名（当前表中的列名）
    fn foreign_key() -> &'static str;

    /// 拥有者键列名（主表中的列名），默认为主键列名
    fn owner_key() -> &'static str {
        T::id_column()
    }
}

/// 多对多关系 trait
///
/// 定义实体之间的多对多关系
///
/// # Example
///
/// ```ignore
/// struct User {
///     id: i64,
///     roles: Vec<Role>,
/// }
///
/// struct Role {
///     id: i64,
///     users: Vec<User>,
/// }
///
/// impl ManyToMany<Role> for User {
///     fn join_table() -> &'static str {
///         "user_roles"
///     }
///
///     fn local_key() -> &'static str {
///         "user_id"
///     }
///
///     fn foreign_key() -> &'static str {
///         "role_id"
///     }
/// }
/// ```
pub trait ManyToMany<T: Entity>: Entity {
    /// 中间表名
    fn join_table() -> &'static str;

    /// 本地键列名（中间表中指向当前表的列名）
    fn local_key() -> &'static str;

    /// 外键列名（中间表中指向关联表的列名）
    fn foreign_key() -> &'static str;
}
