#![warn(missing_docs)]

//! GG 引擎数据库模块
//! 提供数据库驱动抽象与实现

use std::collections::HashMap;

use gg_core::{GError, GErrorKind, GResult};

/// 数据库类型枚举
///
/// 标识支持的数据库后端类型。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DatabaseType {
    /// SQLite 嵌入式数据库
    Sqlite,
    /// PostgreSQL 数据库
    Postgres,
    /// MySQL 数据库
    MySql,
    /// MongoDB 文档数据库
    Mongo,
}

/// 数据库连接配置
///
/// 包含建立数据库连接所需的全部参数。
#[derive(Debug, Clone)]
pub struct DatabaseConfig {
    /// 数据库类型
    pub db_type: DatabaseType,
    /// 主机地址
    pub host: String,
    /// 端口号
    pub port: u16,
    /// 数据库名称
    pub database: String,
    /// 用户名
    pub username: String,
    /// 密码
    pub password: String,
    /// 数据库文件路径（SQLite 专用）
    pub path: String,
}

/// 数据库值类型
///
/// 表示数据库查询结果中单个字段的值，
/// 涵盖常见的 SQL 数据类型。
#[derive(Debug, Clone, PartialEq)]
pub enum DatabaseValue {
    /// 空值
    Null,
    /// 整数值
    Integer(i64),
    /// 浮点数值
    Real(f64),
    /// 文本值
    Text(String),
    /// 二进制数据
    Blob(Vec<u8>),
}

/// 数据库查询结果行
///
/// 以列名到值的映射形式存储单行查询结果。
pub struct Row {
    /// 列名到值的映射
    data: HashMap<String, DatabaseValue>,
}

impl Row {
    /// 获取指定列的值
    ///
    /// 根据列名返回对应的数据库值引用，
    /// 如果列不存在则返回 `None`。
    pub fn get(&self, column: &str) -> Option<&DatabaseValue> {
        self.data.get(column)
    }

    /// 按索引获取列值
    ///
    /// 根据列索引返回对应的数据库值引用，
    /// 如果索引超出范围则返回 `None`。
    pub fn get_by_index(&self, index: usize) -> Option<&DatabaseValue> {
        self.data.values().nth(index)
    }

    /// 获取所有列名
    ///
    /// 返回当前行中所有列名的引用列表。
    pub fn columns(&self) -> Vec<&String> {
        self.data.keys().collect()
    }

    /// 获取列数
    pub fn len(&self) -> usize {
        self.data.len()
    }

    /// 判断行是否为空（无任何列）
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    /// 从 HashMap 构造行
    ///
    /// 将列名到值的映射转换为 `Row` 实例。
    pub fn from_map(map: HashMap<String, DatabaseValue>) -> Self {
        Self { data: map }
    }
}

/// 数据库事务
///
/// 表示一个活跃的数据库事务，
/// 支持提交和回滚操作。
pub struct Transaction {
    /// 数据库路径
    db_path: String,
    /// 事务是否活跃
    active: bool,
}

impl Transaction {
    /// 创建新的事务
    ///
    /// 以指定的数据库路径创建事务实例，
    /// 初始状态为活跃。
    pub fn new(db_path: String) -> Self {
        Self { db_path, active: true }
    }

    /// 提交事务
    ///
    /// 将事务标记为已提交并设置为非活跃状态。
    pub fn commit(&mut self) -> GResult<()> {
        if !self.active {
            return Err(GError {
                kind: GErrorKind::Runtime, message: format!("事务已结束，无法提交: {}", self.db_path)
            });
        }
        self.active = false;
        Ok(())
    }

    /// 回滚事务
    ///
    /// 将事务标记为已回滚并设置为非活跃状态。
    pub fn rollback(&mut self) -> GResult<()> {
        if !self.active {
            return Err(GError {
                kind: GErrorKind::Runtime, message: format!("事务已结束，无法回滚: {}", self.db_path)
            });
        }
        self.active = false;
        Ok(())
    }

    /// 判断事务是否仍处于活跃状态
    pub fn is_active(&self) -> bool {
        self.active
    }
}

/// 数据库驱动 trait
///
/// 定义数据库驱动必须实现的核心操作接口，
/// 包括连接、执行、查询、事务和关闭。
pub trait DatabaseDriver {
    /// 建立数据库连接
    ///
    /// 根据提供的配置创建并返回一个新的数据库驱动实例。
    fn connect(config: &DatabaseConfig) -> GResult<Self>
    where
        Self: Sized;

    /// 执行 SQL 语句
    ///
    /// 执行给定的 SQL 语句并返回受影响的行数。
    fn execute(&mut self, query: &str, params: &[DatabaseValue]) -> GResult<u64>;

    /// 执行查询并返回结果行
    ///
    /// 执行给定的查询语句，以参数绑定方式传入参数，
    /// 返回查询结果行列表。
    fn query(&mut self, query: &str, params: &[DatabaseValue]) -> GResult<Vec<Row>>;

    /// 开始事务
    ///
    /// 在当前连接上开启一个新事务并返回事务实例。
    fn begin_transaction(&mut self) -> GResult<Transaction>;

    /// 关闭数据库连接
    ///
    /// 释放数据库连接资源。
    fn close(&mut self) -> GResult<()>;

    /// 批量执行 SQL 语句
    ///
    /// 在单个调用中执行多条 SQL 语句，
    /// 返回每条语句受影响的行数列表。
    ///
    /// # 参数
    ///
    /// - `queries`: SQL 语句与参数的列表
    fn execute_batch(&mut self, queries: &[(&str, &[DatabaseValue])]) -> GResult<Vec<u64>> {
        let mut results = Vec::with_capacity(queries.len());
        for (query, params) in queries {
            let affected = self.execute(query, params)?;
            results.push(affected);
        }
        Ok(results)
    }

    /// 获取最后插入行的 ID
    ///
    /// 返回最近一次 INSERT 操作的自增主键值。
    /// 如果驱动不支持此操作，返回 None。
    fn last_insert_id(&mut self) -> GResult<Option<i64>> {
        Ok(None)
    }
}

/// SQLite 数据库驱动实现
pub mod sqlite;

/// 数据库连接池实现
pub mod connection_pool;

/// 数据库迁移实现
pub mod migrator;

/// PostgreSQL 数据库驱动实现
#[cfg(feature = "postgres")]
pub mod postgres;

/// MySQL 数据库驱动实现
#[cfg(feature = "mysql")]
pub mod mysql;

pub use connection_pool::DatabaseConnectionPool;
pub use migrator::DatabaseMigrator;
pub use sqlite::SqliteDriver;
#[cfg(feature = "postgres")]
pub use postgres::PostgresDriver;
#[cfg(feature = "mysql")]
pub use mysql::MySqlDriver;
