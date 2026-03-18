//! Schema 定义模块
//!
//! 提供数据库表结构的元信息定义，用于自动迁移。

use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, fs, path::Path, sync::Mutex};

/// 数据库类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DatabaseType {
    /// Turso/SQLite
    Turso,
    /// PostgreSQL
    Postgres,
    /// MySQL
    MySql,
}

impl DatabaseType {
    /// 获取当前数据库类型（根据 feature flag）
    pub fn current() -> Self {
        #[cfg(feature = "postgres")]
        {
            return DatabaseType::Postgres;
        }
        #[cfg(feature = "mysql")]
        {
            return DatabaseType::MySql;
        }
        #[cfg(all(not(feature = "postgres"), not(feature = "mysql")))]
        {
            return DatabaseType::Turso;
        }
    }
}

/// 列类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ColumnType {
    /// 整数类型
    Integer,
    /// 浮点类型
    Real,
    /// 文本类型
    Text,
    /// 二进制类型
    Blob,
}

impl ColumnType {
    /// 转换为 SQL 类型字符串
    pub fn to_sql(&self) -> &'static str {
        self.to_sql_for(DatabaseType::Turso)
    }

    /// 转换为指定数据库的 SQL 类型字符串
    pub fn to_sql_for(&self, db_type: DatabaseType) -> &'static str {
        match db_type {
            DatabaseType::Turso => match self {
                ColumnType::Integer => "INTEGER",
                ColumnType::Real => "REAL",
                ColumnType::Text => "TEXT",
                ColumnType::Blob => "BLOB",
            },
            DatabaseType::Postgres => match self {
                ColumnType::Integer => "BIGINT",
                ColumnType::Real => "DOUBLE PRECISION",
                ColumnType::Text => "TEXT",
                ColumnType::Blob => "BYTEA",
            },
            DatabaseType::MySql => match self {
                ColumnType::Integer => "BIGINT",
                ColumnType::Real => "DOUBLE",
                ColumnType::Text => "VARCHAR(255)",
                ColumnType::Blob => "BLOB",
            },
        }
    }

    /// 转换为 MySQL SQL 类型字符串
    #[cfg(feature = "mysql")]
    pub fn to_mysql_sql(&self) -> &'static str {
        self.to_sql_for(DatabaseType::MySql)
    }

    /// 从 SQL 类型字符串解析
    pub fn from_sql(sql: &str) -> Self {
        match sql.to_uppercase().as_str() {
            "INTEGER" | "INT" | "BIGINT" | "SMALLINT" | "TINYINT" | "SERIAL" | "BIGSERIAL" => ColumnType::Integer,
            "REAL" | "FLOAT" | "DOUBLE" | "NUMERIC" | "DECIMAL" | "DOUBLE PRECISION" => ColumnType::Real,
            "TEXT" | "VARCHAR" | "CHAR" | "STRING" | "VARCHAR(255)" | "TEXT[]" => ColumnType::Text,
            "BLOB" | "BINARY" | "BYTEA" | "LONGBLOB" => ColumnType::Blob,
            _ => ColumnType::Text,
        }
    }
}

/// 列定义
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColumnDef {
    /// 列名
    pub name: String,
    /// 列类型
    pub col_type: ColumnType,
    /// 是否可空
    pub nullable: bool,
    /// 是否主键
    pub primary_key: bool,
    /// 是否自增
    pub auto_increment: bool,
    /// 默认值
    pub default_value: Option<String>,
    /// 是否唯一
    pub unique: bool,
}

impl ColumnDef {
    /// 创建新的列定义
    pub fn new(name: impl Into<String>, col_type: ColumnType) -> Self {
        Self {
            name: name.into(),
            col_type,
            nullable: true,
            primary_key: false,
            auto_increment: false,
            default_value: None,
            unique: false,
        }
    }

    /// 设置为不可空
    pub fn not_null(mut self) -> Self {
        self.nullable = false;
        self
    }

    /// 设置为主键
    pub fn primary_key(mut self) -> Self {
        self.primary_key = true;
        self.nullable = false;
        self
    }

    /// 设置为自增
    pub fn auto_increment(mut self) -> Self {
        self.auto_increment = true;
        self
    }

    /// 设置默认值
    pub fn default(mut self, value: impl Into<String>) -> Self {
        self.default_value = Some(value.into());
        self
    }

    /// 设置为唯一
    pub fn unique(mut self) -> Self {
        self.unique = true;
        self
    }

    /// 生成建表 SQL 片段（使用当前数据库类型）
    pub fn to_sql(&self) -> String {
        self.to_sql_for(DatabaseType::current())
    }

    /// 生成指定数据库类型的建表 SQL 片段
    pub fn to_sql_for(&self, db_type: DatabaseType) -> String {
        let mut sql = format!("{} {}", self.name, self.col_type.to_sql_for(db_type));

        if self.primary_key {
            sql.push_str(" PRIMARY KEY");
        }

        if self.auto_increment {
            match db_type {
                DatabaseType::Turso => sql.push_str(" AUTOINCREMENT"),
                DatabaseType::Postgres => {
                    if self.primary_key && self.col_type == ColumnType::Integer {
                        sql = format!("{} SERIAL", self.name);
                        sql.push_str(" PRIMARY KEY");
                    }
                }
                DatabaseType::MySql => sql.push_str(" AUTO_INCREMENT"),
            }
        }

        if !self.nullable && !self.primary_key {
            sql.push_str(" NOT NULL");
        }

        if let Some(ref default) = self.default_value {
            sql.push_str(&format!(" DEFAULT {}", default));
        }

        if self.unique && !self.primary_key {
            sql.push_str(" UNIQUE");
        }

        sql
    }

    /// 生成 MySQL 建表 SQL 片段
    #[cfg(feature = "mysql")]
    pub fn to_mysql_sql(&self) -> String {
        self.to_sql_for(DatabaseType::MySql)
    }
}

/// 索引定义
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexDef {
    /// 索引名称
    pub name: String,
    /// 表名
    pub table_name: String,
    /// 列名列表
    pub columns: Vec<String>,
    /// 是否唯一索引
    pub unique: bool,
}

impl IndexDef {
    /// 创建新的索引定义
    pub fn new(name: impl Into<String>, table_name: impl Into<String>, columns: Vec<String>) -> Self {
        Self { name: name.into(), table_name: table_name.into(), columns, unique: false }
    }

    /// 设置为唯一索引
    pub fn unique(mut self) -> Self {
        self.unique = true;
        self
    }

    /// 生成创建索引 SQL（使用当前数据库类型）
    pub fn to_create_sql(&self) -> String {
        self.to_create_sql_for(DatabaseType::current())
    }

    /// 生成创建索引 SQL（指定数据库类型）
    pub fn to_create_sql_for(&self, db_type: DatabaseType) -> String {
        let unique_str = if self.unique { "UNIQUE " } else { "" };
        let columns = self.columns.join(", ");
        match db_type {
            DatabaseType::Turso | DatabaseType::Postgres => {
                format!("CREATE {}INDEX {} ON {} ({})", unique_str, self.name, self.table_name, columns)
            }
            DatabaseType::MySql => {
                format!("CREATE {}INDEX {} ON {} ({})", unique_str, self.name, self.table_name, columns)
            }
        }
    }

    /// 生成删除索引 SQL
    pub fn to_drop_sql(&self) -> String {
        format!("DROP INDEX IF EXISTS {}", self.name)
    }
}

/// 数据库 schema 定义
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseSchema {
    /// 数据库名称
    pub name: String,
    /// 数据库类型
    pub r#type: DatabaseType,
    /// 数据库连接字符串（可以直接包含密码）
    pub url: Option<String>,
    /// 环境变量名称（用于从环境变量获取连接字符串）
    pub env: Option<String>,
    /// 该数据库下的表结构列表
    pub schemas: Vec<TableSchema>,
}

impl DatabaseSchema {
    /// 创建新的数据库 schema
    pub fn new(name: impl Into<String>, db_type: DatabaseType) -> Self {
        Self { name: name.into(), r#type: db_type, url: None, env: None, schemas: Vec::new() }
    }

    /// 设置数据库连接 URL
    pub fn url(mut self, url: impl Into<String>) -> Self {
        self.url = Some(url.into());
        self
    }

    /// 设置环境变量名称
    pub fn env(mut self, env: impl Into<String>) -> Self {
        self.env = Some(env.into());
        self
    }

    /// 添加表结构
    pub fn schema(mut self, schema: TableSchema) -> Self {
        self.schemas.push(schema);
        self
    }

    /// 获取数据库连接字符串
    pub fn get_url(&self) -> Result<String, Box<dyn std::error::Error>> {
        if let Some(url) = &self.url {
            Ok(url.clone())
        }
        else if let Some(env_name) = &self.env {
            std::env::var(env_name).map_err(|e| format!("Failed to read environment variable {}: {}", env_name, e).into())
        }
        else {
            Err("No database URL or environment variable specified".into())
        }
    }
}

/// 表结构定义
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableSchema {
    /// 表名
    pub name: String,
    /// 列定义列表
    pub columns: Vec<ColumnDef>,
    /// 索引定义列表
    pub indexes: Vec<IndexDef>,
    /// 外键约束列表
    pub foreign_keys: Vec<ForeignKeyDef>,
}

impl TableSchema {
    /// 创建新的表结构
    pub fn new(name: impl Into<String>) -> Self {
        Self { name: name.into(), columns: Vec::new(), indexes: Vec::new(), foreign_keys: Vec::new() }
    }

    /// 添加列
    pub fn column(mut self, col: ColumnDef) -> Self {
        self.columns.push(col);
        self
    }

    /// 添加索引
    pub fn index(mut self, idx: IndexDef) -> Self {
        self.indexes.push(idx);
        self
    }

    /// 添加外键约束
    pub fn foreign_key(mut self, fk: ForeignKeyDef) -> Self {
        self.foreign_keys.push(fk);
        self
    }

    /// 生成删表 SQL
    pub fn to_drop_sql(&self) -> String {
        format!("DROP TABLE IF EXISTS {}", self.name)
    }

    /// 获取主键列
    pub fn primary_key(&self) -> Option<&ColumnDef> {
        self.columns.iter().find(|c| c.primary_key)
    }

    /// 根据名称获取列
    pub fn get_column(&self, name: &str) -> Option<&ColumnDef> {
        self.columns.iter().find(|c| c.name == name)
    }
}

/// 外键定义
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForeignKeyDef {
    /// 外键名称
    pub name: String,
    /// 本表列名
    pub column: String,
    /// 引用表名
    pub ref_table: String,
    /// 引用列名
    pub ref_column: String,
    /// 更新行为
    pub on_update: ReferentialAction,
    /// 删除行为
    pub on_delete: ReferentialAction,
}

impl ForeignKeyDef {
    /// 创建新的外键定义
    pub fn new(
        name: impl Into<String>,
        column: impl Into<String>,
        ref_table: impl Into<String>,
        ref_column: impl Into<String>,
    ) -> Self {
        Self {
            name: name.into(),
            column: column.into(),
            ref_table: ref_table.into(),
            ref_column: ref_column.into(),
            on_update: ReferentialAction::NoAction,
            on_delete: ReferentialAction::NoAction,
        }
    }

    /// 设置更新时的引用行为
    pub fn on_update(mut self, action: ReferentialAction) -> Self {
        self.on_update = action;
        self
    }

    /// 设置删除时的引用行为
    pub fn on_delete(mut self, action: ReferentialAction) -> Self {
        self.on_delete = action;
        self
    }

    /// 生成约束 SQL 片段
    pub fn to_constraint_sql(&self) -> String {
        format!(
            "CONSTRAINT {} FOREIGN KEY ({}) REFERENCES {} ({}) ON UPDATE {} ON DELETE {}",
            self.name,
            self.column,
            self.ref_table,
            self.ref_column,
            self.on_update.to_sql(),
            self.on_delete.to_sql()
        )
    }

    /// 生成添加外键约束 SQL
    pub fn to_add_sql(&self, table_name: &str) -> String {
        format!("ALTER TABLE {} ADD {}", table_name, self.to_constraint_sql())
    }

    /// 生成删除外键约束 SQL
    pub fn to_drop_sql(&self, table_name: &str) -> String {
        format!("ALTER TABLE {} DROP CONSTRAINT {}", table_name, self.name)
    }
}

/// 外键引用行为
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReferentialAction {
    /// 无操作
    NoAction,
    /// 限制
    Restrict,
    /// 级联
    Cascade,
    /// 设为空
    SetNull,
    /// 设为默认值
    SetDefault,
}

impl ReferentialAction {
    /// 转换为 SQL
    pub fn to_sql(&self) -> &'static str {
        match self {
            ReferentialAction::NoAction => "NO ACTION",
            ReferentialAction::Restrict => "RESTRICT",
            ReferentialAction::Cascade => "CASCADE",
            ReferentialAction::SetNull => "SET NULL",
            ReferentialAction::SetDefault => "SET DEFAULT",
        }
    }
}

/// 完整的 schema 配置（包含多个数据库）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchemaConfig {
    /// 数据库列表
    pub databases: Vec<DatabaseSchema>,
    /// 默认数据库名称（可选，不指定则使用第一个数据库）
    pub default_database: Option<String>,
}

impl SchemaConfig {
    /// 创建新的 schema 配置
    pub fn new() -> Self {
        Self { databases: Vec::new(), default_database: None }
    }

    /// 添加数据库
    pub fn database(mut self, database: DatabaseSchema) -> Self {
        self.databases.push(database);
        self
    }

    /// 设置默认数据库
    pub fn default_database(mut self, name: impl Into<String>) -> Self {
        self.default_database = Some(name.into());
        self
    }

    /// 获取默认数据库
    pub fn get_default_database(&self) -> Option<&DatabaseSchema> {
        if let Some(default_name) = &self.default_database {
            self.databases.iter().find(|db| db.name == *default_name)
        }
        else {
            self.databases.first()
        }
    }

    /// 根据名称获取数据库
    pub fn get_database(&self, name: &str) -> Option<&DatabaseSchema> {
        self.databases.iter().find(|db| db.name == name)
    }
}

/// 数据库链接配置（保留向后兼容）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseLinkConfig {
    /// 数据库类型
    pub r#type: DatabaseType,
    /// 数据库连接字符串（可以直接包含密码）
    pub url: Option<String>,
    /// 环境变量名称（用于从环境变量获取连接字符串）
    pub env: Option<String>,
}

impl DatabaseLinkConfig {
    /// 获取数据库连接字符串
    pub fn get_url(&self) -> Result<String, Box<dyn std::error::Error>> {
        if let Some(url) = &self.url {
            Ok(url.clone())
        }
        else if let Some(env_name) = &self.env {
            std::env::var(env_name).map_err(|e| format!("Failed to read environment variable {}: {}", env_name, e).into())
        }
        else {
            Err("No database URL or environment variable specified".into())
        }
    }
}

/// 列类型快捷构造函数
pub mod col {
    use super::{ColumnDef, ColumnType};

    /// 创建整数列
    pub fn integer(name: &str) -> ColumnDef {
        ColumnDef::new(name, ColumnType::Integer)
    }

    /// 创建浮点列
    pub fn real(name: &str) -> ColumnDef {
        ColumnDef::new(name, ColumnType::Real)
    }

    /// 创建文本列
    pub fn text(name: &str) -> ColumnDef {
        ColumnDef::new(name, ColumnType::Text)
    }

    /// 创建二进制列
    pub fn blob(name: &str) -> ColumnDef {
        ColumnDef::new(name, ColumnType::Blob)
    }

    /// 创建主键列 (自增整数)
    pub fn id(name: &str) -> ColumnDef {
        ColumnDef::new(name, ColumnType::Integer).primary_key().auto_increment()
    }

    /// 创建布尔列
    pub fn boolean(name: &str) -> ColumnDef {
        ColumnDef::new(name, ColumnType::Integer).default("0")
    }

    /// 创建时间戳列
    pub fn timestamp(name: &str) -> ColumnDef {
        ColumnDef::new(name, ColumnType::Text)
    }

    /// 创建 JSON 列
    pub fn json(name: &str) -> ColumnDef {
        ColumnDef::new(name, ColumnType::Text)
    }
}

/// 全局 Schema 注册表
static SCHEMA_REGISTRY: Lazy<Mutex<HashMap<String, TableSchema>>> = Lazy::new(|| Mutex::new(HashMap::new()));

/// 注册 TableSchema 到全局注册表
pub fn register_schema(schema: TableSchema) {
    let mut registry = SCHEMA_REGISTRY.lock().unwrap();
    registry.insert(schema.name.clone(), schema);
}

/// 批量注册 TableSchema 到全局注册表
pub fn register_schemas(schemas: Vec<TableSchema>) {
    let mut registry = SCHEMA_REGISTRY.lock().unwrap();
    for schema in schemas {
        registry.insert(schema.name.clone(), schema);
    }
}

/// 获取所有已注册的 TableSchema
pub fn get_registered_schemas() -> Vec<TableSchema> {
    let registry = SCHEMA_REGISTRY.lock().unwrap();
    registry.values().cloned().collect()
}

/// 获取指定名称的 TableSchema
pub fn get_schema(name: &str) -> Option<TableSchema> {
    let registry = SCHEMA_REGISTRY.lock().unwrap();
    registry.get(name).cloned()
}

/// 清空注册表
pub fn clear_schemas() {
    let mut registry = SCHEMA_REGISTRY.lock().unwrap();
    registry.clear();
}

/// 将所有已注册的 TableSchema 导出为 YAML 字符串
pub fn export_schemas_to_yaml() -> String {
    let schemas = get_registered_schemas();
    serde_yaml::to_string(&schemas).unwrap_or_else(|e| format!("# Error: {}", e))
}

/// 将所有已注册的 TableSchema 导出到 YAML 文件
#[cfg(debug_assertions)]
pub fn export_schemas_to_yaml_file(path: impl AsRef<std::path::Path>) -> std::io::Result<()> {
    use std::{fs::File, io::Write};

    let yaml = export_schemas_to_yaml();
    let mut file = File::create(path)?;
    file.write_all(yaml.as_bytes())?;
    Ok(())
}

/// 在 debug 模式下自动导出 schema 到默认路径
#[cfg(debug_assertions)]
pub fn auto_export_schemas() {
    let path = std::path::Path::new("schemas.yaml");
    if let Err(e) = export_schemas_to_yaml_file(path) {
        eprintln!("Warning: Failed to export schemas: {}", e);
    }
    else {
        println!("Schemas exported to: {}", path.display());
    }
}

impl TableSchema {
    /// 注册当前 TableSchema 到全局注册表
    pub fn register(self) -> Self {
        let name = self.name.clone();
        register_schema(self);
        get_schema(&name).unwrap()
    }

    /// 将当前 TableSchema 导出为 YAML 字符串
    pub fn to_yaml(&self) -> String {
        serde_yaml::to_string(self).unwrap_or_else(|e| format!("# Error: {}", e))
    }

    /// 生成建表 SQL（使用当前数据库类型）
    pub fn to_create_sql(&self) -> String {
        self.to_create_sql_for(DatabaseType::current())
    }

    /// 生成指定数据库类型的建表 SQL
    pub fn to_create_sql_for(&self, db_type: DatabaseType) -> String {
        let mut parts: Vec<String> = self.columns.iter().map(|c| c.to_sql_for(db_type)).collect();
        for fk in &self.foreign_keys {
            parts.push(fk.to_constraint_sql());
        }
        match db_type {
            DatabaseType::Turso | DatabaseType::Postgres => {
                format!("CREATE TABLE {} ({})", self.name, parts.join(", "))
            }
            DatabaseType::MySql => {
                format!("CREATE TABLE {} ({}) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4", self.name, parts.join(", "))
            }
        }
    }

    /// 生成 MySQL 建表 SQL
    #[cfg(feature = "mysql")]
    pub fn to_mysql_create_sql(&self) -> String {
        self.to_create_sql_for(DatabaseType::MySql)
    }

    /// 生成所有创建索引的 SQL（使用当前数据库类型）
    pub fn to_create_indexes_sql(&self) -> Vec<String> {
        self.to_create_indexes_sql_for(DatabaseType::current())
    }

    /// 生成所有创建索引的 SQL（指定数据库类型）
    pub fn to_create_indexes_sql_for(&self, db_type: DatabaseType) -> Vec<String> {
        self.indexes.iter().map(|idx| idx.to_create_sql_for(db_type)).collect()
    }

    /// 生成完整的建表和索引 SQL（使用当前数据库类型）
    pub fn to_full_create_sql(&self) -> Vec<String> {
        self.to_full_create_sql_for(DatabaseType::current())
    }

    /// 生成完整的建表和索引 SQL（指定数据库类型）
    pub fn to_full_create_sql_for(&self, db_type: DatabaseType) -> Vec<String> {
        let mut sqls = vec![self.to_create_sql_for(db_type)];
        sqls.extend(self.to_create_indexes_sql_for(db_type));
        sqls
    }
}

/// 从 YAML 字符串解析 SchemaConfig
pub fn load_schema_config_from_yaml(yaml_str: &str) -> Result<SchemaConfig, serde_yaml::Error> {
    serde_yaml::from_str(yaml_str)
}

/// 从 YAML 文件加载 SchemaConfig
pub fn load_schema_config_from_yaml_file(path: impl AsRef<Path>) -> Result<SchemaConfig, Box<dyn std::error::Error>> {
    let content = fs::read_to_string(path)?;
    let config = load_schema_config_from_yaml(&content)?;
    Ok(config)
}

/// 从 YAML 字符串解析 TableSchema 列表（兼容旧格式）
pub fn load_schemas_from_yaml(yaml_str: &str) -> Result<Vec<TableSchema>, serde_yaml::Error> {
    serde_yaml::from_str(yaml_str)
}

/// 从 YAML 文件加载 TableSchema 列表（兼容旧格式）
pub fn load_schemas_from_yaml_file(path: impl AsRef<Path>) -> Result<Vec<TableSchema>, Box<dyn std::error::Error>> {
    let content = fs::read_to_string(path)?;
    let schemas = load_schemas_from_yaml(&content)?;
    Ok(schemas)
}

/// 从 YAML 文件加载并注册所有 TableSchema（兼容旧格式）
pub fn load_and_register_schemas_from_yaml_file(path: impl AsRef<Path>) -> Result<(), Box<dyn std::error::Error>> {
    let schemas = load_schemas_from_yaml_file(path)?;
    register_schemas(schemas);
    Ok(())
}

/// 将 SchemaConfig 导出为 YAML 字符串
pub fn export_schema_config_to_yaml(config: &SchemaConfig) -> String {
    serde_yaml::to_string(config).unwrap_or_else(|e| format!("# Error: {}", e))
}

/// 将 SchemaConfig 导出到 YAML 文件
pub fn export_schema_config_to_yaml_file(config: &SchemaConfig, path: impl AsRef<Path>) -> std::io::Result<()> {
    use std::{fs::File, io::Write};

    let yaml = export_schema_config_to_yaml(config);
    let mut file = File::create(path)?;
    file.write_all(yaml.as_bytes())?;
    Ok(())
}

/// 从已注册的 schemas 创建 SchemaConfig（将所有表放在默认数据库中）
pub fn create_schema_config_from_registered(default_db: DatabaseSchema, default_database: Option<String>) -> SchemaConfig {
    let schemas = get_registered_schemas();
    let mut default_db = default_db;
    default_db.schemas = schemas;
    SchemaConfig { databases: vec![default_db], default_database }
}

/// 为所有已注册的 schema 生成完整的 SQL（使用当前数据库类型）
pub fn generate_full_sql_for_registered_schemas() -> Vec<String> {
    generate_full_sql_for_registered_schemas_for(DatabaseType::current())
}

/// 为所有已注册的 schema 生成完整的 SQL（指定数据库类型）
pub fn generate_full_sql_for_registered_schemas_for(db_type: DatabaseType) -> Vec<String> {
    let schemas = get_registered_schemas();
    let mut all_sql = Vec::new();
    for schema in schemas {
        all_sql.extend(schema.to_full_create_sql_for(db_type));
    }
    all_sql
}

/// 导出所有数据库类型的 SQL 到文件
#[cfg(debug_assertions)]
pub fn export_sql_for_all_databases(output_dir: impl AsRef<Path>) -> Result<(), Box<dyn std::error::Error>> {
    let output_dir = output_dir.as_ref();
    fs::create_dir_all(output_dir)?;

    let db_types = [DatabaseType::Turso, DatabaseType::Postgres, DatabaseType::MySql];

    for &db_type in &db_types {
        let sqls = generate_full_sql_for_registered_schemas_for(db_type);
        let db_name = match db_type {
            DatabaseType::Turso => "turso",
            DatabaseType::Postgres => "postgres",
            DatabaseType::MySql => "mysql",
        };
        let file_path = output_dir.join(format!("schema_{}.sql", db_name));
        let content = sqls.join(";\n\n") + ";\n";
        fs::write(&file_path, content)?;
        println!("Exported SQL for {} to: {}", db_name, file_path.display());
    }

    Ok(())
}
