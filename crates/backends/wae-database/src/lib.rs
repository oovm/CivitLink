#![warn(missing_docs)]
#![doc = include_str!("readme.md")]

#[cfg(any(feature = "limbo", feature = "postgres", feature = "mysql"))]
mod connection;
#[cfg(any(feature = "limbo", feature = "postgres", feature = "mysql"))]
mod extract;
#[cfg(any(feature = "limbo", feature = "postgres", feature = "mysql"))]
mod middleware;
#[cfg(any(feature = "limbo", feature = "postgres", feature = "mysql"))]
mod orm;
mod schema;
#[cfg(any(feature = "limbo", feature = "mysql"))]
mod types;

#[cfg(any(feature = "limbo", feature = "postgres", feature = "mysql"))]
pub use connection::{
    DatabaseBackend, DatabaseConfig, DatabaseConnection, DatabaseError, DatabaseResult, DatabaseRow, DatabaseRows,
    DatabaseStatement, FromDatabaseValue,
};
#[cfg(feature = "limbo")]
pub use connection::{DatabaseService, LimboConnection};
#[cfg(feature = "mysql")]
pub use connection::{MySqlConnection, MySqlDatabaseService};
#[cfg(feature = "postgres")]
pub use connection::{PostgresConnection, PostgresDatabaseService};
#[cfg(feature = "limbo")]
pub use extract::LimboConnectionExtractor;
#[cfg(feature = "mysql")]
pub use extract::MySqlConnectionExtractor;
#[cfg(feature = "postgres")]
pub use extract::PostgresConnectionExtractor;
#[cfg(any(feature = "limbo", feature = "postgres", feature = "mysql"))]
pub use extract::{DatabaseConnectionExtractor, DatabaseRejection};
#[cfg(any(feature = "limbo", feature = "postgres", feature = "mysql"))]
pub use middleware::{TransactionConfig, TransactionLayer, TransactionMiddlewareBuilder, TransactionService};
#[cfg(feature = "limbo")]
pub use orm::DbRepository;
#[cfg(feature = "mysql")]
pub use orm::MySqlDbRepository;
#[cfg(any(feature = "limbo", feature = "postgres", feature = "mysql"))]
pub use orm::{
    BelongsTo, Condition, DeleteBuilder, Entity, FromRow, HasMany, Join, JoinType, ManyToMany, QueryBuilder, SelectBuilder,
    ToRow, UpdateBuilder,
};

#[cfg(any(feature = "limbo", feature = "postgres"))]
pub use orm::Repository;
pub use schema::{
    ColumnDef, ColumnType, DatabaseLinkConfig, DatabaseSchema, DatabaseType, ForeignKeyDef, IndexDef, ReferentialAction,
    SchemaConfig, TableSchema, clear_schemas, col, create_schema_config_from_registered, export_schema_config_to_yaml,
    export_schema_config_to_yaml_file, export_schemas_to_yaml, generate_full_sql_for_registered_schemas,
    generate_full_sql_for_registered_schemas_for, get_registered_schemas, get_schema, load_and_register_schemas_from_yaml_file,
    load_schema_config_from_yaml, load_schema_config_from_yaml_file, load_schemas_from_yaml, load_schemas_from_yaml_file,
    register_schema, register_schemas,
};

#[cfg(debug_assertions)]
pub use schema::{auto_export_schemas, export_schemas_to_yaml_file, export_sql_for_all_databases};
pub use wae_types::Value;
