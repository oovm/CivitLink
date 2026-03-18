//! 自动迁移模块
//!
//! 提供基于 Model 的自动数据库迁移能力。

#[cfg(any(feature = "database-limbo", feature = "database-postgres", feature = "database-mysql"))]
mod auto_migrate;
#[cfg(any(feature = "database-limbo", feature = "database-postgres", feature = "database-mysql"))]
mod diff;
#[cfg(any(feature = "database-limbo", feature = "database-postgres", feature = "database-mysql"))]
mod generate;
#[cfg(any(feature = "database-limbo", feature = "database-postgres", feature = "database-mysql"))]
mod reflect;

#[cfg(any(feature = "database-limbo", feature = "database-postgres", feature = "database-mysql"))]
pub use auto_migrate::{AutoMigrate, AutoMigrator, AutoMigratorConfig, MigrationPlan};
#[cfg(any(feature = "database-limbo", feature = "database-postgres", feature = "database-mysql"))]
pub use diff::{ColumnDiff, DiffAction, ForeignKeyDiff, IndexDiff, SchemaDiff, TableDiff};
#[cfg(any(feature = "database-limbo", feature = "database-postgres", feature = "database-mysql"))]
pub use generate::CodeGenerator;
#[cfg(any(feature = "database-limbo", feature = "database-postgres", feature = "database-mysql"))]
pub use reflect::SchemaReflector;
