# wae-tools

开发工具集，提供数据库迁移、自动迁移等开发辅助工具。

## 概述

wae-tools 是 WAE 框架的开发工具模块，提供数据库 schema 版本管理和自动迁移能力。支持多种数据库后端（Turso、PostgreSQL、MySQL），采用异步优先设计，与 tokio 运行时深度融合。

## 快速开始

### 手动迁移

```rust
use wae_tools::{Migrator, SimpleMigration};

let mut migrator = Migrator::new();

migrator.register(SimpleMigration::new(1, "create_users_table", r#"
    CREATE TABLE users (
        id INTEGER PRIMARY KEY,
        name TEXT NOT NULL,
        email TEXT UNIQUE
    )
"#).with_description("创建用户表"));

let results = migrator.migrate(&conn).await?;
```

### 自动迁移

```rust
use wae_tools::{AutoMigrator, AutoMigratorConfig};
use wae_database::{TableSchema, col};

let mut migrator = AutoMigrator::new();

migrator.register(TableSchema::new("users")
    .column(col::id("id"))
    .column(col::text("name").not_null())
    .column(col::text("email").unique()));

let results = migrator.migrate(&conn).await?;
```

## 核心用法

### Migration 手动迁移

#### 创建迁移

```rust
use wae_tools::{Migrator, SimpleMigration};

let mut migrator = Migrator::new();

migrator
    .register(SimpleMigration::new(1, "create_users", r#"
        CREATE TABLE users (
            id INTEGER PRIMARY KEY,
            name TEXT NOT NULL,
            email TEXT UNIQUE,
            created_at TEXT DEFAULT CURRENT_TIMESTAMP
        )
    "#).with_down_sql("DROP TABLE users"))
    .register(SimpleMigration::new(2, "create_posts", r#"
        CREATE TABLE posts (
            id INTEGER PRIMARY KEY,
            title TEXT NOT NULL,
            user_id INTEGER REFERENCES users(id)
        )
    "#).with_down_sql("DROP TABLE posts"));
```

#### 执行迁移

```rust
let results = migrator.migrate(&conn).await?;

for result in &results {
    println!(
        "Migration {} (v{}) completed in {}ms",
        result.name, result.version, result.duration_ms
    );
}
```

#### 回滚迁移

回滚最后一个迁移：

```rust
let result = migrator.rollback(&conn).await?;
```

回滚到指定版本：

```rust
let results = migrator.rollback_to(&conn, 1).await?;
```

重置所有迁移：

```rust
let results = migrator.reset(&conn).await?;
```

#### 刷新数据库

回滚所有迁移后重新执行：

```rust
let results = migrator.refresh(&conn).await?;
```

#### 查询迁移状态

```rust
let status_list = migrator.status(&conn).await?;

for status in &status_list {
    println!(
        "[{}] {} - {}",
        if status.executed { "x" } else { " " },
        status.version,
        status.name
    );
}
```

### AutoMigrate 自动迁移

#### 创建自动迁移器

```rust
use wae_tools::{AutoMigrator, AutoMigratorConfig};
use wae_database::{TableSchema, col};

let config = AutoMigratorConfig::safe();
let mut migrator = AutoMigrator::with_config(config);
```

#### 注册表结构

```rust
migrator
    .register(TableSchema::new("users")
        .column(col::id("id"))
        .column(col::text("name").not_null())
        .column(col::text("email").unique())
        .column(col::timestamp("created_at").default("CURRENT_TIMESTAMP")))
    .register(TableSchema::new("posts")
        .column(col::id("id"))
        .column(col::text("title").not_null())
        .column(col::text("content"))
        .column(col::integer("user_id").references("users(id)")));
```

#### 生成迁移计划

预览将要执行的 SQL：

```rust
let plan = migrator.plan(&conn).await?;

if !plan.is_empty() {
    println!("Migration plan:");
    plan.print_sql();
}
```

#### 执行迁移

```rust
let results = migrator.migrate(&conn).await?;
println!("Executed {} statements", results.len());
```

#### 同步表结构

仅创建不存在的表，不修改已有表：

```rust
let created = migrator.sync(&conn).await?;
for table in &created {
    println!("Created table: {}", table);
}
```

#### 干运行模式

只生成 SQL 不执行：

```rust
let config = AutoMigratorConfig {
    dry_run: true,
    ..AutoMigratorConfig::default()
};

let migrator = AutoMigrator::with_config(config);

let plan = migrator.plan(&conn).await?;
plan.print_sql();
```

### 配置选项

#### AutoMigratorConfig

```rust
let safe_config = AutoMigratorConfig::safe();

let full_config = AutoMigratorConfig::full();

let custom_config = AutoMigratorConfig {
    create_tables: true,
    drop_tables: false,
    add_columns: true,
    drop_columns: false,
    create_indexes: true,
    drop_indexes: false,
    dry_run: false,
};
```

### 实现 AutoMigrate Trait

为自定义类型实现自动迁移：

```rust
use wae_tools::AutoMigrate;
use wae_database::{TableSchema, col};

struct User;

impl AutoMigrate for User {
    fn table_schema() -> TableSchema {
        TableSchema::new("users")
            .column(col::id("id"))
            .column(col::text("name").not_null())
            .column(col::text("email").unique())
    }
}
```

## 最佳实践

### 迁移版本管理

使用递增版本号，每个迁移只做一件事：

```rust
migrator
    .register(SimpleMigration::new(1, "create_users", "CREATE TABLE users (...)"))
    .register(SimpleMigration::new(2, "add_user_email", "ALTER TABLE users ADD COLUMN email TEXT"))
    .register(SimpleMigration::new(3, "create_posts", "CREATE TABLE posts (...)"));
```

### 可逆迁移

始终提供 down SQL：

```rust
SimpleMigration::new(1, "create_users", r#"
    CREATE TABLE users (id INTEGER PRIMARY KEY, name TEXT)
"#).with_down_sql("DROP TABLE users")
```

### 生产环境安全配置

```rust
let config = AutoMigratorConfig {
    create_tables: true,
    drop_tables: false,
    add_columns: true,
    drop_columns: false,
    create_indexes: true,
    drop_indexes: false,
    dry_run: false,
};
```

### 开发环境完整配置

```rust
let config = AutoMigratorConfig::full();
```

### 迁移前检查

```rust
let plan = migrator.plan(&conn).await?;

if !plan.is_empty() {
    println!("Pending migrations:");
    plan.print_sql();

    if confirm("Execute migrations?") {
        migrator.migrate(&conn).await?;
    }
}
```

## 常见问题

### Q: 如何处理迁移冲突？

确保版本号唯一且递增。如果多人开发，协调版本号分配：

```rust
migrator.register(SimpleMigration::new(101, "team_a_feature", "..."));
migrator.register(SimpleMigration::new(102, "team_b_feature", "..."));
```

### Q: SQLite 不支持某些 ALTER 操作怎么办？

使用 `rebuild_table` 方法：

```rust
AutoMigrator::rebuild_table(&conn, &schema).await?;
```

### Q: 如何查看迁移历史？

```rust
let status_list = migrator.status(&conn).await?;

for status in status_list.iter().filter(|s| s.executed) {
    println!("{}: {} at {:?}", status.version, status.name, status.executed_at);
}
```

### Q: 自动迁移会删除数据吗？

使用 `safe` 配置不会删除列或表：

```rust
let config = AutoMigratorConfig::safe();
```

`full` 配置会删除不存在的列和表，谨慎使用。

### Q: 如何在 CI 中使用？

```rust
let plan = migrator.plan(&conn).await?;

if !plan.is_empty() {
    panic!("Database schema is out of sync. Run migrations locally.");
}
```
