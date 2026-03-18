# wae-database

数据库服务抽象层，提供统一的数据库操作抽象，支持多种数据库后端。

## 概述

wae-database 是一个异步优先的数据库抽象层，深度融合 tokio 运行时。通过 feature flags 支持多种数据库后端：`turso`（SQLite 兼容）、`postgres`、`mysql`。同时提供轻量级 ORM 功能和 Schema 定义能力。

## 快速开始

```rust
use wae_database::{DatabaseService, Value, text, integer};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let db = DatabaseService::in_memory().await?;
    let conn = db.connect()?;

    conn.execute("CREATE TABLE users (id INTEGER PRIMARY KEY, name TEXT)").await?;
    conn.execute_with("INSERT INTO users (name) VALUES (?)", vec![text("Alice")]).await?;

    let rows = conn.query("SELECT id, name FROM users").await?;
    while let Some(row) = rows.next().await? {
        let id: i64 = row.get(0)?;
        let name: String = row.get(1)?;
        println!("User {}: {}", id, name);
    }
    Ok(())
}
```

## 基本操作

### 创建数据库连接

```rust
use wae_database::{DatabaseService, DatabaseConfig};

let db = DatabaseService::in_memory().await?;

let db = DatabaseService::file("data/app.db").await?;

let config = DatabaseConfig::file("data/app.db");
let db = DatabaseService::new(&config).await?;
```

### 执行 DDL 语句

```rust
let conn = db.connect()?;

conn.execute(r#"
    CREATE TABLE users (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        name TEXT NOT NULL,
        email TEXT UNIQUE,
        created_at INTEGER
    )
"#).await?;

conn.execute("DROP TABLE IF EXISTS temp_data").await?;
```

### 插入数据

```rust
use wae_database::{text, integer, blob, null};

conn.execute_with(
    "INSERT INTO users (name, email, created_at) VALUES (?, ?, ?)",
    vec![text("Bob"), text("bob@example.com"), integer(1700000000)]
).await?;

conn.execute_with(
    "INSERT INTO users (name, email) VALUES (?, ?)",
    vec![text("Charlie"), null()]
).await?;

let id = conn.execute_with(
    "INSERT INTO users (name) VALUES (?)",
    vec![text("David")]
).await?;
println!("Inserted with row id: {}", id);
```

### 查询数据

```rust
let rows = conn.query("SELECT id, name, email FROM users").await?;

while let Some(row) = rows.next().await? {
    let id: i64 = row.get(0)?;
    let name: String = row.get(1)?;
    let email: Option<String> = row.get(2)?;
    println!("{}: {} ({:?})", id, name, email);
}

let all_rows = conn.query("SELECT * FROM users").await?.collect().await?;
println!("Total users: {}", all_rows.len());
```

### 带参数查询

```rust
let rows = conn.query_with(
    "SELECT * FROM users WHERE id > ? AND name LIKE ?",
    vec![integer(0), text("%a%")]
).await?;

let rows = conn.query_with(
    "SELECT * FROM users WHERE email IS NULL",
    vec![]
).await?;
```

### 预处理语句

预处理语句适合重复执行的场景，性能更优：

```rust
let mut stmt = conn.prepare("SELECT * FROM users WHERE name = ?").await?;

for name in ["Alice", "Bob", "Charlie"] {
    let rows = stmt.query(vec![text(name)]).await?;
    while let Some(row) = rows.next().await? {
        let id: i64 = row.get(0)?;
        println!("Found {}: id={}", name, id);
    }
}
```

### 更新和删除

```rust
let affected = conn.execute_with(
    "UPDATE users SET email = ? WHERE name = ?",
    vec![text("new@email.com"), text("Alice")]
).await?;
println!("Updated {} rows", affected);

let affected = conn.execute_with(
    "DELETE FROM users WHERE id = ?",
    vec![integer(1)]
).await?;
println!("Deleted {} rows", affected);
```

### 值类型

```rust
use wae_database::{integer, text, blob, null, Value};

let values: Vec<Value> = vec![
    integer(42),
    text("hello"),
    blob(vec![1, 2, 3]),
    null(),
];
```

## ORM 用法

### 定义实体

```rust
use wae_database::{Entity, FromRow, ToRow, DatabaseRow, DatabaseResult, Value, integer, text};

#[derive(Debug, Clone)]
struct User {
    id: i64,
    name: String,
    email: Option<String>,
    created_at: i64,
}

impl Entity for User {
    fn table_name() -> &'static str { "users" }
}

impl FromRow for User {
    fn from_row(row: &DatabaseRow) -> DatabaseResult<Self> {
        Ok(User {
            id: row.get(0)?,
            name: row.get(1)?,
            email: row.get(2)?,
            created_at: row.get(3)?,
        })
    }
}

impl ToRow for User {
    fn to_row(&self) -> Vec<Value> {
        vec![
            integer(self.id),
            text(&self.name),
            match &self.email {
                Some(e) => text(e),
                None => null(),
            },
            integer(self.created_at),
        ]
    }
}
```

### 使用 Repository

```rust
use wae_database::{DbRepository, Repository};

let repo: DbRepository<User> = DbRepository::new(conn);

let user = repo.find_by_id(1).await?;
if let Some(u) = user {
    println!("Found user: {:?}", u);
}

let all_users = repo.find_all().await?;
for u in all_users {
    println!("- {} ({})", u.name, u.email.unwrap_or_default());
}

let new_user = User {
    id: 0,
    name: "Eve".to_string(),
    email: Some("eve@example.com".to_string()),
    created_at: 1700000000,
};
let inserted_id = repo.save(&new_user).await?;

let deleted = repo.delete(1).await?;
println!("Deleted: {}", deleted);
```

### 查询构建器

```rust
use wae_database::{SelectBuilder, InsertBuilder, UpdateBuilder, DeleteBuilder, Condition};

let select = SelectBuilder::<User>::new()
    .where_clause(Condition::Gt("id".to_string(), integer(0)))
    .order_by("name")
    .limit(10);

let insert = InsertBuilder::<User>::new(&user);

let update = UpdateBuilder::<User>::new()
    .set("email", text("updated@email.com"))
    .where_clause(Condition::Eq("id".to_string(), integer(1)));

let delete = DeleteBuilder::<User>::new()
    .where_clause(Condition::Eq("id".to_string(), integer(1)));
```

### 条件组合

```rust
use wae_database::Condition;

let complex_condition = Condition::And(vec![
    Condition::Gt("id".to_string(), integer(0)),
    Condition::Or(vec![
        Condition::Like("name".to_string(), "%a%".to_string()),
        Condition::Eq("email".to_string(), text("admin@example.com")),
    ]),
]);

let in_condition = Condition::In("id".to_string(), vec![
    integer(1),
    integer(2),
    integer(3),
]);
```

## Schema 定义

### 创建表结构

```rust
use wae_database::{TableSchema, ColumnDef, ColumnType, IndexDef, ForeignKeyDef, col, ReferentialAction};

let users_schema = TableSchema {
    name: "users".to_string(),
    columns: vec![
        col("id", ColumnType::Integer)
            .auto_increment(true)
            .nullable(false),
        col("name", ColumnType::Varchar(255))
            .nullable(false),
        col("email", ColumnType::Varchar(255))
            .unique(true),
        col("created_at", ColumnType::Timestamp)
            .nullable(true),
    ],
    primary_key: Some("id".to_string()),
    indexes: vec![
        IndexDef {
            name: "idx_users_email".to_string(),
            columns: vec!["email".to_string()],
            unique: true,
        },
    ],
    foreign_keys: vec![],
};

let orders_schema = TableSchema {
    name: "orders".to_string(),
    columns: vec![
        col("id", ColumnType::BigInt).auto_increment(true),
        col("user_id", ColumnType::BigInt).nullable(false),
        col("amount", ColumnType::Decimal(10, 2)).nullable(false),
        col("status", ColumnType::Varchar(50)).nullable(false),
    ],
    primary_key: Some("id".to_string()),
    indexes: vec![
        IndexDef {
            name: "idx_orders_user_id".to_string(),
            columns: vec!["user_id".to_string()],
            unique: false,
        },
    ],
    foreign_keys: vec![
        ForeignKeyDef {
            columns: vec!["user_id".to_string()],
            ref_table: "users".to_string(),
            ref_columns: vec!["id".to_string()],
            on_delete: ReferentialAction::Cascade,
            on_update: ReferentialAction::Cascade,
        },
    ],
};
```

### 可用的列类型

| 类型 | 说明 |
|------|------|
| `Integer` | 32 位整数 |
| `BigInt` | 64 位整数 |
| `Text` | 无限制文本 |
| `Varchar(n)` | 限制长度文本 |
| `Boolean` | 布尔值 |
| `Float` | 单精度浮点 |
| `Double` | 双精度浮点 |
| `Blob` | 二进制数据 |
| `Timestamp` | 时间戳 |
| `Decimal(p, s)` | 精确小数 |

## 迁移工具

### 简单迁移脚本

```rust
async fn run_migrations(conn: &TursoConnection) -> DatabaseResult<()> {
    conn.execute(r#"
        CREATE TABLE IF NOT EXISTS users (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL,
            email TEXT UNIQUE,
            created_at INTEGER DEFAULT (strftime('%s', 'now'))
        )
    "#).await?;

    conn.execute(r#"
        CREATE TABLE IF NOT EXISTS orders (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            user_id INTEGER NOT NULL,
            amount REAL NOT NULL,
            FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
        )
    "#).await?;

    conn.execute("CREATE INDEX IF NOT EXISTS idx_users_email ON users(email)").await?;

    Ok(())
}
```

### 版本化迁移

```rust
struct Migration {
    version: i64,
    name: &'static str,
    up: &'static str,
    down: &'static str,
}

const MIGRATIONS: &[Migration] = &[
    Migration {
        version: 1,
        name: "create_users",
        up: "CREATE TABLE users (id INTEGER PRIMARY KEY, name TEXT)",
        down: "DROP TABLE users",
    },
    Migration {
        version: 2,
        name: "add_email_to_users",
        up: "ALTER TABLE users ADD COLUMN email TEXT",
        down: "ALTER TABLE users DROP COLUMN email",
    },
];

async fn ensure_migrations_table(conn: &TursoConnection) -> DatabaseResult<()> {
    conn.execute(r#"
        CREATE TABLE IF NOT EXISTS _migrations (
            version INTEGER PRIMARY KEY,
            applied_at INTEGER
        )
    "#).await
}

async fn get_current_version(conn: &TursoConnection) -> DatabaseResult<i64> {
    let rows = conn.query("SELECT COALESCE(MAX(version), 0) FROM _migrations").await?;
    if let Some(row) = rows.next().await? {
        Ok(row.get(0)?)
    } else {
        Ok(0)
    }
}

async fn run_pending_migrations(conn: &TursoConnection) -> DatabaseResult<()> {
    ensure_migrations_table(conn).await?;
    let current = get_current_version(conn).await?;

    for migration in MIGRATIONS {
        if migration.version > current {
            conn.execute(migration.up).await?;
            conn.execute_with(
                "INSERT INTO _migrations (version, applied_at) VALUES (?, strftime('%s', 'now'))",
                vec![integer(migration.version)]
            ).await?;
            println!("Applied migration: {}", migration.name);
        }
    }

    Ok(())
}
```

## 最佳实践

### 连接管理

```rust
struct AppState {
    db: DatabaseService,
}

impl AppState {
    async fn new() -> DatabaseResult<Self> {
        let db = DatabaseService::file("data/app.db").await?;
        Ok(Self { db })
    }

    fn connect(&self) -> DatabaseResult<TursoConnection> {
        self.db.connect()
    }
}

async fn handle_request(state: &AppState) -> DatabaseResult<()> {
    let conn = state.connect()?;
    let rows = conn.query("SELECT * FROM users").await?;
    Ok(())
}
```

### 错误处理

```rust
use wae_database::{DatabaseResult, DatabaseError};

async fn find_user(conn: &TursoConnection, id: i64) -> DatabaseResult<Option<User>> {
    let rows = conn.query_with("SELECT * FROM users WHERE id = ?", vec![integer(id)]).await?;
    if let Some(row) = rows.next().await? {
        Ok(Some(User::from_row(&row)?))
    } else {
        Ok(None)
    }
}

async fn safe_delete(conn: &TursoConnection, id: i64) -> Result<bool, String> {
    match conn.execute_with("DELETE FROM users WHERE id = ?", vec![integer(id)]).await {
        Ok(0) => Ok(false),
        Ok(_) => Ok(true),
        Err(e) => Err(format!("Database error: {}", e)),
    }
}
```

### 事务模式

```rust
async fn transfer_points(
    conn: &TursoConnection,
    from_id: i64,
    to_id: i64,
    amount: i64
) -> DatabaseResult<()> {
    conn.execute("BEGIN TRANSACTION").await?;

    let result = async {
        conn.execute_with(
            "UPDATE users SET points = points - ? WHERE id = ?",
            vec![integer(amount), integer(from_id)]
        ).await?;

        conn.execute_with(
            "UPDATE users SET points = points + ? WHERE id = ?",
            vec![integer(amount), integer(to_id)]
        ).await?;

        Ok::<_, DatabaseError>(())
    }.await;

    match result {
        Ok(_) => conn.execute("COMMIT").await?,
        Err(_) => conn.execute("ROLLBACK").await?,
    };

    Ok(())
}
```

### 批量操作

```rust
async fn batch_insert(conn: &TursoConnection, users: &[User]) -> DatabaseResult<()> {
    let mut stmt = conn.prepare("INSERT INTO users (name, email) VALUES (?, ?)").await?;

    for user in users {
        stmt.execute(vec![text(&user.name), match &user.email {
            Some(e) => text(e),
            None => null(),
        }]).await?;
    }

    Ok(())
}
```

## 常见问题

### Q: 如何处理 NULL 值？

使用 `Option<T>` 类型接收可能为 NULL 的列：

```rust
let email: Option<String> = row.get(2)?;
```

插入时使用 `null()` 函数：

```rust
conn.execute_with("INSERT INTO users (name, email) VALUES (?, ?)", vec![
    text("Bob"),
    null(),
]).await?;
```

### Q: 如何获取最后插入的 ID？

SQLite 会自动返回最后插入的 rowid：

```rust
let rowid = conn.execute_with("INSERT INTO users (name) VALUES (?)", vec![text("Alice")]).await?;
println!("Inserted ID: {}", rowid);
```

### Q: 如何处理大结果集？

使用流式处理而非一次性收集：

```rust
let mut rows = conn.query("SELECT * FROM large_table").await?;

while let Some(row) = rows.next().await? {
    let id: i64 = row.get(0)?;
    process_row(id)?;
}
```

### Q: 如何选择数据库后端？

在 `Cargo.toml` 中启用对应的 feature：

```toml
[dependencies]
wae-database = { version = "0.1", features = ["turso"] }
```

- **turso**: 嵌入式 SQLite，适合本地应用和测试
- **postgres**: 生产级关系数据库，适合高并发场景
- **mysql**: 广泛使用的关系数据库

### Q: 如何调试 SQL 语句？

建议在开发环境打印执行的 SQL：

```rust
let sql = "SELECT * FROM users WHERE id = ?";
println!("[SQL] {}", sql);
let rows = conn.query_with(sql, vec![integer(1)]).await?;
```

### Q: 连接池如何管理？

当前版本每次 `db.connect()` 返回新连接。对于高并发场景，建议在应用层实现连接池或使用单一连接配合 Arc 共享：

```rust
use std::sync::Arc;
use tokio::sync::Mutex;

let conn = Arc::new(Mutex::new(db.connect()?));

async fn query_user(conn: Arc<Mutex<TursoConnection>>, id: i64) -> DatabaseResult<Option<User>> {
    let guard = conn.lock().await;
    let rows = guard.query_with("SELECT * FROM users WHERE id = ?", vec![integer(id)]).await?;
    // ...
}
```

## 依赖说明

### turso 后端

turso 是一个纯 Rust 编写的 SQLite 兼容数据库（原 limbo 项目）。当前配置已禁用默认的 mimalloc 内存分配器：

```toml
[dependencies]
turso = { version = "0.5.0", default-features = false }
```

#### 纯血化状态

| 依赖 | 状态 | 说明 |
|------|------|------|
| mimalloc | ✅ 已禁用 | 通过 `default-features = false` |
| io-uring | ✅ 非默认 | Linux 特有，Windows 不受影响 |
| aegis | ⚠️ 需要 C 编译器 | AES 加密库，需要上游支持纯 Rust |
| simsimd | ⚠️ 需要 C 编译器 | SIMD 优化，需要上游支持纯 Rust |
| bindgen | ⚠️ 需要 clang | 构建时依赖，需要上游支持 |

#### 完全纯 Rust 构建的限制

当前 turso crate 不提供禁用 aegis/simsimd 的 features。如需完全纯 Rust 构建，需要：
1. 向 turso 项目提交 issue 请求添加 `pure-rust` feature
2. 或等待 turso 项目官方支持纯 Rust 构建

详见项目规格文档 `.trae/specs/pure-rust-limbo/spec.md`。
