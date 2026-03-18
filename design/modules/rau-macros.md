# wae-macros

过程宏模块，提供用于简化开发的过程宏，包括 Schema 自动生成和编译时 SQL 查询宏。

## 概述

wae-macros 提供两类核心功能：
- **派生宏**：自动生成 Schema 定义，简化 OpenAPI 文档生成
- **SQL 查询宏**：类似 sqlx 风格的查询宏，底层使用 wae-database

## 快速开始

### 派生宏

```rust
use wae_schema::{Schema, ToSchema};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct User {
    pub id: u64,
    pub name: String,
    pub email: Option<String>,
}
```

### SQL 查询宏

```rust
use wae_macros::{query_as, execute};
use wae_database::{DatabaseConnection, FromRow};

let users = query_as!(User, conn, "SELECT id, name FROM users").await?;

let affected = execute!(conn, "INSERT INTO users (name) VALUES (?)", "Alice").await?;
```

## 核心用法

### SQL 查询宏

#### query! - 基础查询

返回 `DatabaseRows`，需要手动迭代：

```rust
use wae_macros::query;
use wae_database::{DatabaseConnection, DatabaseRow};

async fn get_users(conn: &dyn DatabaseConnection) -> Result<Vec<DatabaseRow>, Box<dyn std::error::Error>> {
    let mut rows = query!(conn, "SELECT id, name, email FROM users WHERE active = ?", true).await?;
    let mut results = Vec::new();
    while let Some(row) = rows.next().await? {
        results.push(row);
    }
    Ok(results)
}
```

#### query_as! - 自动映射

自动将结果映射到结构体，需要实现 `FromRow`：

```rust
use wae_macros::query_as;
use wae_database::{FromRow, DatabaseRow, DatabaseResult};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
struct User {
    id: i64,
    name: String,
    email: String,
}

impl FromRow for User {
    fn from_row(row: &DatabaseRow) -> DatabaseResult<Self> {
        Ok(Self {
            id: row.get(0)?,
            name: row.get(1)?,
            email: row.get(2)?,
        })
    }
}

async fn get_users(conn: &dyn DatabaseConnection) -> Result<Vec<User>, Box<dyn std::error::Error>> {
    let users = query_as!(User, conn, "SELECT id, name, email FROM users WHERE active = ?", true).await?;
    Ok(users)
}
```

#### execute! - 执行语句

执行 INSERT/UPDATE/DELETE，返回影响行数：

```rust
use wae_macros::execute;
use wae_database::DatabaseConnection;

async fn insert_user(conn: &dyn DatabaseConnection) -> Result<u64, Box<dyn std::error::Error>> {
    let affected = execute!(conn, "INSERT INTO users (name, email) VALUES (?, ?)", "Alice", "alice@example.com").await?;
    Ok(affected)
}

async fn update_user(conn: &dyn DatabaseConnection, id: i64, name: &str) -> Result<u64, Box<dyn std::error::Error>> {
    let affected = execute!(conn, "UPDATE users SET name = ? WHERE id = ?", name, id).await?;
    Ok(affected)
}

async fn delete_user(conn: &dyn DatabaseConnection, id: i64) -> Result<u64, Box<dyn std::error::Error>> {
    let affected = execute!(conn, "DELETE FROM users WHERE id = ?", id).await?;
    Ok(affected)
}
```

#### query_scalar! - 单值查询

返回单个值，适用于聚合查询：

```rust
use wae_macros::query_scalar;
use wae_database::DatabaseConnection;

async fn count_users(conn: &dyn DatabaseConnection) -> Result<i64, Box<dyn std::error::Error>> {
    let count = query_scalar!(i64, conn, "SELECT COUNT(*) FROM users").await?;
    Ok(count)
}

async fn sum_orders(conn: &dyn DatabaseConnection, user_id: i64) -> Result<f64, Box<dyn std::error::Error>> {
    let total = query_scalar!(f64, conn, "SELECT SUM(amount) FROM orders WHERE user_id = ?", user_id).await?;
    Ok(total)
}
```

### 派生宏

#### ToSchema

自动生成 OpenAPI Schema：

```rust
use wae_schema::{Schema, ToSchema};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct User {
    pub id: u64,
    pub name: String,
    pub email: Option<String>,
}
```

自动提取文档注释作为字段描述：

```rust
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct Product {
    pub id: i64,
    pub name: String,
    pub price: f64,
    pub stock: i32,
}
```

#### 枚举支持

枚举自动生成字符串枚举 Schema：

```rust
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub enum Status {
    Active,
    Inactive,
    Pending,
}
```

### API 文档宏

#### api_doc!

生成 OpenAPI 路由文档：

```rust
use wae_macros::api_doc;

let doc = api_doc! {
    "/users" => {
        GET => {
            summary: "获取用户列表",
            response: 200 => "成功",
        },
        POST => {
            summary: "创建用户",
            body: "User",
            response: 201 => "创建成功",
        },
    },
};
```

### 参数类型支持

SQL 查询宏支持多种参数类型：

| Rust 类型 | Value 类型 |
|-----------|-----------|
| `i8` - `i128`, `u8` - `u128`, `isize`, `usize` | `Integer` |
| `f32`, `f64` | `Float` |
| `bool` | `Bool` |
| `String`, `&str` | `String` |
| `Vec<u8>` | `Bytes` |
| 其他类型 | 通过 `Into<Value>` 转换 |

## 最佳实践

### 实现 FromRow

使用索引访问字段：

```rust
impl FromRow for Product {
    fn from_row(row: &DatabaseRow) -> DatabaseResult<Self> {
        Ok(Self {
            id: row.get(0)?,
            name: row.get(1)?,
            price: row.get(2)?,
            stock: row.get(3)?,
        })
    }
}
```

### 完整 CRUD 示例

```rust
use wae_macros::{query_as, execute, query_scalar, ToSchema};
use wae_database::{DatabaseConnection, FromRow, DatabaseRow, DatabaseResult};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct Product {
    pub id: i64,
    pub name: String,
    pub price: f64,
    pub stock: i32,
}

impl FromRow for Product {
    fn from_row(row: &DatabaseRow) -> DatabaseResult<Self> {
        Ok(Self {
            id: row.get(0)?,
            name: row.get(1)?,
            price: row.get(2)?,
            stock: row.get(3)?,
        })
    }
}

async fn create_product(conn: &dyn DatabaseConnection, name: &str, price: f64, stock: i32) -> Result<u64, Box<dyn std::error::Error>> {
    let affected = execute!(conn, "INSERT INTO products (name, price, stock) VALUES (?, ?, ?)", name, price, stock).await?;
    Ok(affected)
}

async fn get_product_by_name(conn: &dyn DatabaseConnection, name: &str) -> Result<Option<Product>, Box<dyn std::error::Error>> {
    let products = query_as!(Product, conn, "SELECT id, name, price, stock FROM products WHERE name = ?", name).await?;
    Ok(products.into_iter().next())
}

async fn count_products(conn: &dyn DatabaseConnection) -> Result<i64, Box<dyn std::error::Error>> {
    let count = query_scalar!(i64, conn, "SELECT COUNT(*) FROM products").await?;
    Ok(count)
}

async fn update_stock(conn: &dyn DatabaseConnection, id: i64, delta: i32) -> Result<u64, Box<dyn std::error::Error>> {
    let affected = execute!(conn, "UPDATE products SET stock = stock + ? WHERE id = ?", delta, id).await?;
    Ok(affected)
}
```

## 常见问题

### Q: query_as! 返回什么类型？

返回 `Vec<T>`，其中 `T` 是你指定的类型：

```rust
let users: Vec<User> = query_as!(User, conn, "SELECT ...").await?;
```

### Q: 如何处理 NULL 值？

使用 `Option<T>` 类型：

```rust
#[derive(Debug, Clone)]
struct User {
    id: i64,
    name: String,
    email: Option<String>,
}

impl FromRow for User {
    fn from_row(row: &DatabaseRow) -> DatabaseResult<Self> {
        Ok(Self {
            id: row.get(0)?,
            name: row.get(1)?,
            email: row.get::<Option<String>>(2)?,
        })
    }
}
```

### Q: 如何传递多个参数？

直接在 SQL 后添加参数：

```rust
let users = query_as!(
    User,
    conn,
    "SELECT * FROM users WHERE status = ? AND age > ?",
    "active",
    18
).await?;
```

### Q: ToSchema 支持嵌套结构吗？

支持，嵌套类型也需要实现 `ToSchema`：

```rust
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct Order {
    pub id: i64,
    pub user: User,
    pub items: Vec<OrderItem>,
}
```
