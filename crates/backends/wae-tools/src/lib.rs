//! WAE Tools - 开发工具集
//!
//! 提供数据库迁移、种子数据等开发辅助工具。
#![warn(missing_docs)]

#[cfg(any(feature = "database-limbo", feature = "database-postgres", feature = "database-mysql"))]
pub mod auto_migrate;
pub mod cmds;
#[cfg(any(feature = "database-limbo", feature = "database-postgres", feature = "database-mysql"))]
pub mod migration;
#[cfg(any(feature = "database-limbo", feature = "database-postgres", feature = "database-mysql"))]
pub mod schema_sync;

#[cfg(any(feature = "database-limbo", feature = "database-postgres", feature = "database-mysql"))]
pub use auto_migrate::{
    AutoMigrate, AutoMigrator, AutoMigratorConfig, CodeGenerator, ColumnDiff, DiffAction, IndexDiff, MigrationPlan, SchemaDiff,
    SchemaReflector, TableDiff,
};
#[cfg(any(feature = "database-limbo", feature = "database-postgres", feature = "database-mysql"))]
pub use migration::{
    Migration, MigrationOptions, MigrationRecord, MigrationResult, MigrationStatus, MigrationStatusSummary, Migrator,
    SimpleMigration,
};
#[cfg(any(feature = "database-limbo", feature = "database-postgres", feature = "database-mysql"))]
pub use schema_sync::{MigrationOperation, MigrationPlan as SchemaMigrationPlan, SchemaSynchronizer};

pub use cmds::*;
