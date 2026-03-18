//! Schema 同步模块
//!
//! 提供从 schemas.yaml 到数据库的同步功能。

use std::path::Path;
use wae_database::{DatabaseSchema, DatabaseType, SchemaConfig, TableSchema, load_schema_config_from_yaml_file};
use wae_types::{WaeError, WaeResult};

/// 迁移操作类型
#[derive(Debug, Clone)]
pub enum MigrationOperation {
    /// 创建表
    CreateTable { schema: TableSchema },
    /// 删除表
    DropTable { table_name: String },
    /// 添加列
    AddColumn { table_name: String, column: wae_database::ColumnDef },
    /// 删除列
    DropColumn { table_name: String, column_name: String },
    /// 修改列
    AlterColumn { table_name: String, column: wae_database::ColumnDef },
    /// 创建索引
    CreateIndex { table_name: String, index: wae_database::IndexDef },
    /// 删除索引
    DropIndex { table_name: String, index_name: String },
}

/// 单个数据库的迁移计划
#[derive(Debug, Clone)]
pub struct DatabaseMigrationPlan {
    /// 数据库名称
    pub database_name: String,
    /// 数据库类型
    pub db_type: DatabaseType,
    /// 待执行的操作
    pub operations: Vec<MigrationOperation>,
}

impl DatabaseMigrationPlan {
    /// 创建空计划
    pub fn new(database_name: String, db_type: DatabaseType) -> Self {
        Self { database_name, db_type, operations: Vec::new() }
    }

    /// 计划是否为空
    pub fn is_empty(&self) -> bool {
        self.operations.is_empty()
    }

    /// 添加操作
    pub fn add_operation(&mut self, op: MigrationOperation) {
        self.operations.push(op);
    }

    /// 检测是否包含破坏性操作
    pub fn has_destructive_operations(&self) -> bool {
        self.operations.iter().any(Self::is_destructive)
    }

    /// 判断操作是否是破坏性的
    fn is_destructive(op: &MigrationOperation) -> bool {
        matches!(
            op,
            MigrationOperation::DropTable { .. }
                | MigrationOperation::DropColumn { .. }
                | MigrationOperation::AlterColumn { .. }
        )
    }

    /// 获取所有破坏性操作
    pub fn destructive_operations(&self) -> Vec<&MigrationOperation> {
        self.operations.iter().filter(|op| Self::is_destructive(op)).collect()
    }

    /// 生成 SQL 语句列表
    pub fn to_sql(&self) -> Vec<String> {
        let mut sqls = Vec::new();
        for op in &self.operations {
            match op {
                MigrationOperation::CreateTable { schema } => {
                    sqls.extend(schema.to_full_create_sql_for(self.db_type));
                }
                MigrationOperation::DropTable { table_name } => {
                    sqls.push(format!("DROP TABLE IF EXISTS {}", table_name));
                }
                MigrationOperation::AddColumn { table_name, column } => {
                    sqls.push(format!("ALTER TABLE {} ADD COLUMN {}", table_name, column.to_sql_for(self.db_type)));
                }
                MigrationOperation::DropColumn { table_name, column_name } => {
                    sqls.push(format!("ALTER TABLE {} DROP COLUMN {}", table_name, column_name));
                }
                MigrationOperation::AlterColumn { table_name, column } => {
                    sqls.push(format!("ALTER TABLE {} ALTER COLUMN {}", table_name, column.to_sql_for(self.db_type)));
                }
                MigrationOperation::CreateIndex { table_name: _, index } => {
                    sqls.push(index.to_create_sql_for(self.db_type));
                }
                MigrationOperation::DropIndex { table_name: _, index_name } => {
                    sqls.push(format!("DROP INDEX IF EXISTS {}", index_name));
                }
            }
        }
        sqls
    }

    /// 打印迁移计划
    pub fn print(&self) {
        println!("\nDatabase: {}", self.database_name);
        println!("{}", "-".repeat(60));

        if self.is_empty() {
            println!("No migration needed. Database is already up to date.");
            return;
        }

        println!("Migration Plan ({} operations):", self.operations.len());

        for (i, op) in self.operations.iter().enumerate() {
            println!("  {}. {}", i + 1, operation_description(op));
        }

        println!("\nSQL Statements:");
        for sql in self.to_sql() {
            println!("    {sql};");
        }
    }
}

/// 完整的迁移计划（包含所有数据库）
#[derive(Debug, Clone)]
pub struct MigrationPlan {
    /// 各个数据库的迁移计划
    pub database_plans: Vec<DatabaseMigrationPlan>,
}

impl MigrationPlan {
    /// 创建空计划
    pub fn empty() -> Self {
        Self { database_plans: Vec::new() }
    }

    /// 计划是否为空
    pub fn is_empty(&self) -> bool {
        self.database_plans.iter().all(|p| p.is_empty())
    }

    /// 添加数据库迁移计划
    pub fn add_database_plan(&mut self, plan: DatabaseMigrationPlan) {
        self.database_plans.push(plan);
    }

    /// 检测是否包含破坏性操作
    pub fn has_destructive_operations(&self) -> bool {
        self.database_plans.iter().any(|p| p.has_destructive_operations())
    }

    /// 获取所有破坏性操作
    pub fn destructive_operations(&self) -> Vec<(&str, &MigrationOperation)> {
        let mut result = Vec::new();
        for plan in &self.database_plans {
            for op in plan.destructive_operations() {
                result.push((plan.database_name.as_str(), op));
            }
        }
        result
    }

    /// 打印完整的迁移计划
    pub fn print(&self) {
        if self.is_empty() {
            println!("No migration needed. All databases are already up to date.");
            return;
        }

        println!("Complete Migration Plan");
        println!("{}", "=".repeat(60));

        for plan in &self.database_plans {
            plan.print();
        }

        println!("\n{}", "=".repeat(60));
    }
}

/// 迁移操作描述辅助函数
pub fn operation_description(op: &MigrationOperation) -> String {
    match op {
        MigrationOperation::CreateTable { schema } => {
            format!("Create table '{}'", schema.name)
        }
        MigrationOperation::DropTable { table_name } => {
            format!("Drop table '{}'", table_name)
        }
        MigrationOperation::AddColumn { table_name, column } => {
            format!("Add column '{}' to table '{}'", column.name, table_name)
        }
        MigrationOperation::DropColumn { table_name, column_name } => {
            format!("Drop column '{}' from table '{}'", column_name, table_name)
        }
        MigrationOperation::AlterColumn { table_name, column } => {
            format!("Alter column '{}' in table '{}'", column.name, table_name)
        }
        MigrationOperation::CreateIndex { table_name, index } => {
            format!("Create index '{}' on table '{}'", index.name, table_name)
        }
        MigrationOperation::DropIndex { table_name, index_name } => {
            format!("Drop index '{}' from table '{}'", index_name, table_name)
        }
    }
}

/// Schema 同步器
pub struct SchemaSynchronizer {
    /// 完整的 schema 配置（包含多个数据库）
    config: SchemaConfig,
}

impl SchemaSynchronizer {
    /// 从 YAML 文件创建同步器
    pub fn from_yaml_file(path: impl AsRef<std::path::Path>) -> WaeResult<Self> {
        let config = load_schema_config_from_yaml_file(path)
            .map_err(|e| wae_types::WaeError::internal(format!("Failed to load schema config: {}", e)))?;
        Ok(Self { config })
    }

    /// 从 WAE 文件创建同步器
    pub fn from_wae_file(_path: impl AsRef<Path>) -> WaeResult<Self> {
        Err(wae_types::WaeError::internal("WAE file loading is temporarily disabled"))
    }

    /// 从 WAE 目录创建同步器
    pub fn from_wae_directory(_path: impl AsRef<Path>) -> WaeResult<Self> {
        Err(wae_types::WaeError::internal("WAE directory loading is temporarily disabled"))
    }

    /// 从已加载的 schema config 创建同步器
    pub fn new(config: SchemaConfig) -> Self {
        Self { config }
    }

    /// 获取 schema 配置
    pub fn config(&self) -> &SchemaConfig {
        &self.config
    }

    /// 获取所有数据库
    pub fn databases(&self) -> &[DatabaseSchema] {
        &self.config.databases
    }

    /// 获取默认数据库
    pub fn default_database(&self) -> Option<&DatabaseSchema> {
        self.config.get_default_database()
    }

    /// 生成预览 SQL
    pub fn generate_preview_sql(&self) -> Vec<(String, Vec<String>)> {
        let mut result = Vec::new();
        for db in &self.config.databases {
            let mut sqls = Vec::new();
            for schema in &db.schemas {
                sqls.extend(schema.to_full_create_sql_for(db.r#type));
            }
            result.push((db.name.clone(), sqls));
        }
        result
    }

    /// 打印预览
    pub fn print_preview(&self) {
        println!("Schema Preview");
        println!("{}", "=".repeat(60));
        println!("Total databases: {}", self.config.databases.len());

        if let Some(default_name) = &self.config.default_database {
            println!("Default database: {}", default_name);
        }

        for db in &self.config.databases {
            println!("\nDatabase: {} ({:?})", db.name, db.r#type);

            if let Ok(url) = db.get_url() {
                println!("  URL: {}", mask_database_url(&url));
            }

            println!("  Tables: {}", db.schemas.len());

            for schema in &db.schemas {
                println!("    - {} ({} columns, {} indexes)", schema.name, schema.columns.len(), schema.indexes.len());
            }
        }

        println!("\n{}", "=".repeat(60));
        println!("\nGenerated SQL:");

        for (db_name, sqls) in self.generate_preview_sql() {
            println!("\n-- Database: {}", db_name);
            for sql in sqls {
                println!("  {sql};");
            }
        }
    }
}

/// 屏蔽数据库 URL 中的密码
fn mask_database_url(url: &str) -> String {
    if let Some(pos) = url.find('@') {
        if let Some(protocol_pos) = url.find("://") {
            let protocol = &url[..=protocol_pos + 2];
            let rest = &url[pos + 1..];
            format!("{}***@{}", protocol, rest)
        }
        else {
            url.to_string()
        }
    }
    else {
        url.to_string()
    }
}
