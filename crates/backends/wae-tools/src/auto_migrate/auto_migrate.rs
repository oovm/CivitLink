//! 自动迁移器
//!
//! 基于 Model 定义自动生成并执行数据库迁移。

use crate::{
    auto_migrate::{diff::SchemaDiff, reflect::SchemaReflector},
    migration::{Migration, MigrationResult},
};
use wae_types::{WaeError, WaeResult};

use std::collections::HashMap;
use tracing::{info, warn};
use wae_database::{DatabaseConnection, TableSchema};

/// 自动迁移器配置
#[derive(Debug, Clone)]
pub struct AutoMigratorConfig {
    /// 是否自动创建表
    pub create_tables: bool,
    /// 是否自动删除未定义的表
    pub drop_tables: bool,
    /// 是否自动添加列
    pub add_columns: bool,
    /// 是否自动删除未定义的列 (危险操作)
    pub drop_columns: bool,
    /// 是否自动修改列
    pub alter_columns: bool,
    /// 是否自动创建索引
    pub create_indexes: bool,
    /// 是否自动删除未定义的索引
    pub drop_indexes: bool,
    /// 是否自动创建外键约束
    pub create_foreign_keys: bool,
    /// 是否自动删除未定义的外键约束
    pub drop_foreign_keys: bool,
    /// 是否在执行前打印 SQL
    pub dry_run: bool,
}

impl Default for AutoMigratorConfig {
    fn default() -> Self {
        Self {
            create_tables: true,
            drop_tables: false,
            add_columns: true,
            drop_columns: false,
            alter_columns: false,
            create_indexes: true,
            drop_indexes: true,
            create_foreign_keys: true,
            drop_foreign_keys: true,
            dry_run: false,
        }
    }
}

impl AutoMigratorConfig {
    /// 创建安全配置 (不会删除任何内容)
    pub fn safe() -> Self {
        Self {
            create_tables: true,
            drop_tables: false,
            add_columns: true,
            drop_columns: false,
            alter_columns: false,
            create_indexes: true,
            drop_indexes: false,
            create_foreign_keys: true,
            drop_foreign_keys: false,
            dry_run: false,
        }
    }

    /// 创建完整配置 (会删除未定义的表和列)
    pub fn full() -> Self {
        Self {
            create_tables: true,
            drop_tables: true,
            add_columns: true,
            drop_columns: true,
            alter_columns: true,
            create_indexes: true,
            drop_indexes: true,
            create_foreign_keys: true,
            drop_foreign_keys: true,
            dry_run: false,
        }
    }
}

/// 迁移计划
#[derive(Debug, Clone)]
pub struct MigrationPlan {
    /// 差异描述
    pub diff: SchemaDiff,
    /// 要执行的 SQL 语句
    pub sql_statements: Vec<String>,
}

impl MigrationPlan {
    /// 创建空的迁移计划
    pub fn empty() -> Self {
        Self { diff: SchemaDiff::empty(), sql_statements: Vec::new() }
    }

    /// 是否为空
    pub fn is_empty(&self) -> bool {
        self.sql_statements.is_empty()
    }

    /// 获取 SQL 语句数量
    pub fn sql_count(&self) -> usize {
        self.sql_statements.len()
    }

    /// 打印 SQL 语句
    pub fn print_sql(&self) {
        for (i, sql) in self.sql_statements.iter().enumerate() {
            println!("{}. {}", i + 1, sql);
        }
    }
}

/// 自动迁移器
pub struct AutoMigrator {
    /// 表结构定义
    schemas: HashMap<String, TableSchema>,
    /// 配置
    config: AutoMigratorConfig,
}

impl AutoMigrator {
    /// 创建新的自动迁移器
    pub fn new() -> Self {
        Self { schemas: HashMap::new(), config: AutoMigratorConfig::default() }
    }

    /// 使用指定配置创建
    pub fn with_config(config: AutoMigratorConfig) -> Self {
        Self { schemas: HashMap::new(), config }
    }

    /// 注册表结构
    pub fn register(&mut self, schema: TableSchema) -> &mut Self {
        self.schemas.insert(schema.name.clone(), schema);
        self
    }

    /// 批量注册表结构
    pub fn register_all(&mut self, schemas: Vec<TableSchema>) -> &mut Self {
        for schema in schemas {
            self.register(schema);
        }
        self
    }

    /// 设置配置
    pub fn set_config(&mut self, config: AutoMigratorConfig) -> &mut Self {
        self.config = config;
        self
    }

    /// 获取注册的表结构
    pub fn schemas(&self) -> &HashMap<String, TableSchema> {
        &self.schemas
    }

    /// 生成迁移计划
    pub async fn plan(&self, conn: &dyn DatabaseConnection) -> WaeResult<MigrationPlan> {
        let reflector = SchemaReflector::new(conn);
        let actual_schemas = reflector.get_all_schemas().await?;

        let mut filtered_expected = HashMap::new();
        for (name, schema) in &self.schemas {
            filtered_expected.insert(name.clone(), schema.clone());
        }

        let diff = SchemaDiff::compare(&filtered_expected, &actual_schemas);

        let sql_statements = self.generate_sql(&diff);

        Ok(MigrationPlan { diff, sql_statements })
    }

    /// 根据配置过滤并生成 SQL
    fn generate_sql(&self, diff: &SchemaDiff) -> Vec<String> {
        let mut sqls = Vec::new();

        for table_diff in &diff.table_diffs {
            match table_diff.action {
                crate::auto_migrate::diff::DiffAction::Create => {
                    if self.config.create_tables {
                        sqls.extend(table_diff.to_sql());
                    }
                }
                crate::auto_migrate::diff::DiffAction::Drop => {
                    if self.config.drop_tables {
                        sqls.extend(table_diff.to_sql());
                    }
                }
                crate::auto_migrate::diff::DiffAction::Alter => {
                    for col_diff in &table_diff.column_diffs {
                        match col_diff.action {
                            crate::auto_migrate::diff::DiffAction::Create => {
                                if self.config.add_columns {
                                    if let Some(sql) = col_diff.to_sql(&table_diff.table_name) {
                                        sqls.push(sql);
                                    }
                                }
                            }
                            crate::auto_migrate::diff::DiffAction::Drop => {
                                if self.config.drop_columns {
                                    warn!(
                                        table = %table_diff.table_name,
                                        column = %col_diff.column_name,
                                        "SQLite does not support DROP COLUMN, table rebuild required"
                                    );
                                }
                            }
                            crate::auto_migrate::diff::DiffAction::Alter => {
                                if self.config.alter_columns {
                                    warn!(
                                        table = %table_diff.table_name,
                                        column = %col_diff.column_name,
                                        "Column alteration may require table rebuild depending on database backend"
                                    );
                                }
                            }
                        }
                    }

                    for idx_diff in &table_diff.index_diffs {
                        match idx_diff.action {
                            crate::auto_migrate::diff::DiffAction::Create => {
                                if self.config.create_indexes {
                                    if let Some(sql) = idx_diff.to_sql() {
                                        sqls.push(sql);
                                    }
                                }
                            }
                            crate::auto_migrate::diff::DiffAction::Drop => {
                                if self.config.drop_indexes {
                                    if let Some(sql) = idx_diff.to_sql() {
                                        sqls.push(sql);
                                    }
                                }
                            }
                            crate::auto_migrate::diff::DiffAction::Alter => {
                                if self.config.create_indexes && self.config.drop_indexes {
                                    if let Some(sql) = idx_diff.to_sql() {
                                        sqls.push(sql);
                                    }
                                }
                            }
                        }
                    }

                    for fk_diff in &table_diff.foreign_key_diffs {
                        match fk_diff.action {
                            crate::auto_migrate::diff::DiffAction::Create => {
                                if self.config.create_foreign_keys {
                                    if let Some(sql) = fk_diff.to_sql(&table_diff.table_name) {
                                        sqls.push(sql);
                                    }
                                }
                            }
                            crate::auto_migrate::diff::DiffAction::Drop => {
                                if self.config.drop_foreign_keys {
                                    if let Some(sql) = fk_diff.to_sql(&table_diff.table_name) {
                                        sqls.push(sql);
                                    }
                                }
                            }
                            crate::auto_migrate::diff::DiffAction::Alter => {
                                if self.config.create_foreign_keys && self.config.drop_foreign_keys {
                                    if let Some(sql) = fk_diff.to_sql(&table_diff.table_name) {
                                        sqls.push(sql);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        sqls
    }

    /// 执行迁移
    pub async fn migrate(&self, conn: &dyn DatabaseConnection) -> WaeResult<Vec<MigrationResult>> {
        let plan = self.plan(conn).await?;

        if plan.is_empty() {
            info!("No migration needed");
            return Ok(Vec::new());
        }

        if self.config.dry_run {
            info!("Dry run mode, SQL statements:");
            plan.print_sql();
            return Ok(Vec::new());
        }

        info!(sql_count = plan.sql_count(), "Executing auto migration");

        let mut results = Vec::new();
        for sql in &plan.sql_statements {
            let start = std::time::Instant::now();
            let executed_at = chrono::Utc::now();

            info!(sql = %sql, "Executing SQL");

            match conn.execute(sql).await {
                Ok(_) => {
                    let duration_ms = start.elapsed().as_millis() as u64;
                    results.push(MigrationResult {
                        name: format!("auto_migrate_{}", results.len()),
                        version: 0,
                        executed_at,
                        duration_ms,
                        success: true,
                        dry_run: false,
                        error: None,
                    });
                }
                Err(e) => {
                    let duration_ms = start.elapsed().as_millis() as u64;
                    let error_msg = e.to_string();

                    warn!(error = %error_msg, sql = %sql, "SQL execution failed");

                    results.push(MigrationResult {
                        name: format!("auto_migrate_{}", results.len()),
                        version: 0,
                        executed_at,
                        duration_ms,
                        success: false,
                        dry_run: false,
                        error: Some(error_msg.clone()),
                    });

                    return Err(WaeError::internal(format!("SQL execution failed: {}", error_msg)));
                }
            }
        }

        info!(count = results.len(), "Auto migration completed successfully");

        Ok(results)
    }

    /// 同步表结构 (创建不存在的表)
    pub async fn sync(&self, conn: &dyn DatabaseConnection) -> WaeResult<Vec<String>> {
        let reflector = SchemaReflector::new(conn);
        let mut created = Vec::new();

        for (table_name, schema) in &self.schemas {
            if !reflector.table_exists(table_name).await? {
                info!(table = %table_name, "Creating table");

                let sql = schema.to_create_sql();
                conn.execute(&sql).await?;

                for idx in &schema.indexes {
                    conn.execute(&idx.to_create_sql()).await?;
                }

                created.push(table_name.clone());
            }
        }

        Ok(created)
    }

    /// 重建表 (用于处理 SQLite 不支持的 ALTER 操作)
    pub async fn rebuild_table(conn: &dyn DatabaseConnection, schema: &TableSchema) -> WaeResult<()> {
        let temp_name = format!("{}_new", schema.name);

        let create_sql = schema.to_create_sql().replace(&schema.name, &temp_name);
        conn.execute(&create_sql).await?;

        let columns: Vec<&str> = schema.columns.iter().map(|c| c.name.as_str()).collect();
        let cols_str = columns.join(", ");

        let copy_sql = format!("INSERT INTO {} ({}) SELECT {} FROM {}", temp_name, cols_str, cols_str, schema.name);
        conn.execute(&copy_sql).await?;

        let drop_sql = format!("DROP TABLE {}", schema.name);
        conn.execute(&drop_sql).await?;

        let rename_sql = format!("ALTER TABLE {} RENAME TO {}", temp_name, schema.name);
        conn.execute(&rename_sql).await?;

        for idx in &schema.indexes {
            conn.execute(&idx.to_create_sql()).await?;
        }

        Ok(())
    }
}

impl Default for AutoMigrator {
    fn default() -> Self {
        Self::new()
    }
}

/// 自动迁移 trait - 实现此 trait 以支持自动迁移

pub trait AutoMigrate: Send + Sync + 'static {
    /// 返回表结构定义
    fn table_schema() -> TableSchema;

    /// 表名
    fn table_name() -> String {
        Self::table_schema().name
    }
}

/// 基于 AutoMigrate trait 的迁移包装
pub struct AutoModelMigration<M: AutoMigrate> {
    _marker: std::marker::PhantomData<M>,
}

impl<M: AutoMigrate> AutoModelMigration<M> {
    /// 创建新的迁移
    pub fn new() -> Self {
        Self { _marker: std::marker::PhantomData }
    }
}

impl<M: AutoMigrate> Default for AutoModelMigration<M> {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait::async_trait]
impl<M: AutoMigrate> Migration for AutoModelMigration<M> {
    fn name(&self) -> &str {
        Box::leak(format!("auto_{}", M::table_name()).into_boxed_str())
    }

    fn version(&self) -> i64 {
        0
    }

    fn description(&self) -> Option<&str> {
        Some("Auto-generated migration")
    }

    async fn up(&self, conn: &dyn DatabaseConnection) -> wae_database::DatabaseResult<()> {
        let schema = M::table_schema();
        let sql = schema.to_create_sql();
        conn.execute(&sql).await?;

        for idx in &schema.indexes {
            conn.execute(&idx.to_create_sql()).await?;
        }

        Ok(())
    }

    async fn down(&self, conn: &dyn DatabaseConnection) -> wae_database::DatabaseResult<()> {
        let schema = M::table_schema();
        conn.execute(&schema.to_drop_sql()).await?;
        Ok(())
    }

    fn reversible(&self) -> bool {
        true
    }
}
