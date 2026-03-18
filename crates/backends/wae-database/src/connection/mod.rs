//! 数据库连接模块

mod config;
mod row;
mod statement;
mod trait_impl;
mod value_impl;

#[cfg(feature = "limbo")]
mod limbo;

#[cfg(feature = "postgres")]
pub(crate) mod postgres;

#[cfg(feature = "mysql")]
mod mysql;

pub use config::{DatabaseConfig, DatabaseError, DatabaseResult, PoolConfig, RecyclingMethod};
pub use row::{DatabaseRow, DatabaseRows};
pub use statement::DatabaseStatement;
pub use trait_impl::{DatabaseBackend, DatabaseConnection};
pub use value_impl::FromDatabaseValue;

#[cfg(feature = "limbo")]
pub use limbo::{DatabaseService, LimboConnection};

#[cfg(feature = "postgres")]
pub use postgres::{PoolMetrics, PostgresConnection, PostgresDatabaseService};

#[cfg(feature = "mysql")]
pub use mysql::{MySqlConnection, MySqlDatabaseService, MySqlPoolMetrics};
