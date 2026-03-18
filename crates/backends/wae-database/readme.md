# WAE Database - 数据库服务抽象层

提供统一的数据库操作抽象，基于 Limbo 后端实现。
Limbo 是一个用 Rust 重写的 SQLite 兼容数据库，支持异步 I/O。

## ORM 功能

- `Entity` trait: 定义数据库实体
- `FromRow`/`ToRow` trait: 行映射
- `QueryBuilder`: 查询构建器
- `Repository`: CRUD 操作

## Schema 功能

- `TableSchema`: 表结构定义
- `ColumnDef`: 列定义
- `IndexDef`: 索引定义

## 值类型统一

本模块统一使用 `wae_types::Value` 作为值类型，
内部自动转换为数据库原生类型。

---

数据库模块 - 提供统一的数据库抽象层。

## 主要功能

- **多数据库支持**: 支持 PostgreSQL、MySQL、SQLite
- **连接池**: 内置连接池管理
- **事务支持**: 支持数据库事务
- **迁移工具**: 集成数据库迁移

## 技术栈

- **ORM**: SeaORM
- **连接池**: sea-orm
- **异步运行时**: Tokio

## 使用示例

```rust,no_run
use wae_database::Value;

fn main() {
    let value = Value::String("hello".to_string());
    println!("{:?}", value);
}
```

## 支持的数据库

| 数据库 | 驱动 |
|--------|------|
| PostgreSQL | sqlx-postgres |
| MySQL | sqlx-mysql |
| SQLite | sqlx-sqlite |
