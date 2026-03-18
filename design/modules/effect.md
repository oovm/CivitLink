# 代数效应 (Algebraic Effects)

WAE 提供了代数效应风格的依赖注入模块 `wae-effect`，通过单一的 `Effectful` 参数统一管理所有依赖。

## 概述

代数效应模式通过单一 `Effectful` 参数统一管理依赖，解决传统 Axum 模式参数膨胀问题。

### 传统模式 vs 代数效应模式

**传统模式**：
```rust
async fn handler(
    State(db): State<Arc<Database>>,
    State(cache): State<Arc<Cache>>,
    Extension(user): Extension<AuthenticatedUser>,
    Extension(repo): Extension<Arc<RepositoryService>>,
) { ... }
```

**代数效应模式**：
```rust
async fn handler(ae: Effectful) -> Result<Json<Response>, EffectError> {
    let db = ae.use_database().await?;
    let user = ae.use_auth::<AuthenticatedUser>().await?;
    let config = ae.use_config::<AppConfig>().await?;
}
```

**优势**：
- **声明式**: 依赖在函数体内按需获取
- **类型安全**: 编译时检查依赖类型
- **可测试**: 轻松 mock 依赖
- **组合性**: 依赖可以嵌套组合

## 快速开始

### 1. 定义配置类型

```rust
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct AppConfig {
    pub database_url: String,
    pub redis_url: String,
}
```

### 2. 构建依赖容器

```rust
use wae::effect::{AlgebraicEffect, Dependencies};
use std::sync::Arc;

async fn build_deps() -> Arc<Dependencies> {
    let config = AppConfig {
        database_url: "sqlite://app.db".to_string(),
        redis_url: "redis://localhost".to_string(),
    };
    
    let db = DatabaseService::file("app.db").await?;
    
    AlgebraicEffect::new()
        .with_config(config)
        .with_database(Arc::new(db))
        .build()
}
```

### 3. 配置路由

```rust
use axum::Router;

fn routes(deps: Arc<Dependencies>) -> Router {
    Router::new()
        .route("/api/users", get(list_users))
        .route("/api/posts", get(list_posts))
        .with_state(deps)
}
```

### 4. 编写 Handler

```rust
async fn list_users(ae: Effectful) -> Result<Json<Vec<User>>, EffectError> {
    let db = ae.use_database().await?;
    let config = ae.use_config::<AppConfig>().await?;
    
    let users = db.query("SELECT * FROM users").await?;
    Ok(Json(users))
}
```

## 核心用法

### Effectful 方法

| 方法 | 说明 |
|------|------|
| `use_config::<T>()` | 获取配置 |
| `use_database()` | 获取数据库服务 |
| `use_auth::<T>()` | 获取认证用户 |
| `use_::<T>()` | 获取任意依赖 |
| `use_arc::<T>()` | 获取 Arc 包装的依赖 |
| `header(name)` | 获取请求头 |
| `parts()` | 获取原始 HTTP 请求部分 |

### AlgebraicEffect 方法

| 方法 | 说明 |
|------|------|
| `with_config(config)` | 注册配置 |
| `with_database(db)` | 注册数据库服务 |
| `with_auth(auth)` | 注册认证信息 |
| `with(value)` | 注册任意依赖 |
| `with_arc(value)` | 注册 Arc 依赖 |
| `build()` | 构建依赖容器 |

### 自定义依赖

```rust
#[derive(Clone)]
pub struct CacheService {
    // ...
}

async fn handler(ae: Effectful) -> Result<Json<Data>, EffectError> {
    let cache = ae.use_arc::<CacheService>().await?;
    // ...
}

let deps = AlgebraicEffect::new()
    .with_arc(Arc::new(CacheService::new()))
    .build();
```

### 便捷宏

```rust
use wae::effect::use_effect;

async fn handler(ae: Effectful) -> Result<(), EffectError> {
    use_effect!(ae, config: AppConfig);
    use_effect!(ae, db: arc DatabaseService);
    use_effect!(ae, user: auth AuthenticatedUser);
    use_effect!(ae, database db);
    
    // 使用 config, db, user
}
```

宏语法说明：

| 语法 | 说明 |
|------|------|
| `use_effect!(ae, name: Type)` | 获取任意依赖 |
| `use_effect!(ae, name: arc Type)` | 获取 Arc 包装的依赖 |
| `use_effect!(ae, config name: Type)` | 获取配置 |
| `use_effect!(ae, auth name: Type)` | 获取认证用户 |
| `use_effect!(ae, database name)` | 获取数据库服务 |

### Dependencies 容器

```rust
use wae::effect::Dependencies;

let mut deps = Dependencies::new();

deps.register(my_service);
deps.register_arc(Arc::new(shared_service));

let service: MyService = deps.get()?;
let shared: Arc<SharedService> = deps.get_arc()?;

if deps.contains::<MyService>() {
    // ...
}
```

## 最佳实践

### 完整应用示例

```rust
use wae::effect::{AlgebraicEffect, Dependencies, Effectful};
use axum::{Router, routing::get, Json};
use std::sync::Arc;

#[derive(Debug, Clone, serde::Deserialize)]
pub struct AppConfig {
    pub database_url: String,
    pub api_key: String,
}

#[derive(Debug, Clone)]
pub struct UserService {
    // ...
}

async fn create_app() -> Router {
    let config = AppConfig {
        database_url: "sqlite://app.db".to_string(),
        api_key: "secret".to_string(),
    };
    
    let deps = AlgebraicEffect::new()
        .with_config(config)
        .with_arc(Arc::new(UserService::new()))
        .build();
    
    Router::new()
        .route("/users", get(list_users))
        .route("/users/:id", get(get_user))
        .with_state(deps)
}

async fn list_users(ae: Effectful) -> Result<Json<Vec<String>>, EffectError> {
    use_effect!(ae, config: AppConfig);
    use_effect!(ae, service: arc UserService);
    
    Ok(Json(vec!["user1".to_string(), "user2".to_string()]))
}

async fn get_user(ae: Effectful) -> Result<Json<String>, EffectError> {
    let user = ae.use_auth::<AuthenticatedUser>().await?;
    Ok(Json(user.name))
}
```

### 分层架构

```rust
struct RepositoryLayer;
struct ServiceLayer;
struct HandlerLayer;

async fn handler(ae: Effectful) -> Result<Json<Data>, EffectError> {
    let repo = ae.use_arc::<RepositoryLayer>().await?;
    let service = ae.use_arc::<ServiceLayer>().await?;
    
    let data = service.process(&repo).await?;
    Ok(Json(data))
}
```

## 测试

代数效应模式使测试变得简单：

```rust
#[tokio::test]
async fn test_handler() {
    let mock_config = AppConfig::default();
    let mock_db = DatabaseService::in_memory().await.unwrap();
    
    let deps = AlgebraicEffect::new()
        .with_config(mock_config)
        .with_database(Arc::new(mock_db))
        .build();
    
    // 直接调用 handler 进行测试
}
```

### Mock 依赖

```rust
#[derive(Clone)]
struct MockUserService;

impl MockUserService {
    fn new() -> Self { Self }
}

#[tokio::test]
async fn test_with_mock() {
    let deps = AlgebraicEffect::new()
        .with_arc(Arc::new(MockUserService::new()))
        .build();
    
    // 测试逻辑
}
```

## 常见问题

### Q: EffectBuilder 废弃了吗？

是的，`EffectBuilder` 已被标记为废弃，请使用 `AlgebraicEffect` 替代：

```rust
let deps = AlgebraicEffect::new()
    .with_config(config)
    .build();
```

### Q: 如何获取请求头？

```rust
async fn handler(ae: Effectful) -> Result<(), EffectError> {
    let auth_header = ae.header("Authorization").await?;
    // ...
}
```

### Q: 如何在中间件中注入依赖？

使用 `Extensions` 或直接访问 `Dependencies`：

```rust
async fn middleware(
    Extension(deps): Extension<Arc<Dependencies>>,
    mut req: Request,
    next: Next,
) -> Response {
    let config: AppConfig = deps.get()?;
    // ...
    next.run(req).await
}
```

### Q: 如何处理依赖不存在的情况？

`use_` 方法返回 `Result`，可以处理错误：

```rust
async fn handler(ae: Effectful) -> Result<Json<Data>, EffectError> {
    match ae.use_::<OptionalService>().await {
        Ok(service) => { /* 使用服务 */ },
        Err(_) => { /* 服务不存在，使用默认行为 */ },
    }
    Ok(Json(Data::default()))
}
```

### Q: 如何注册多个同类型依赖？

使用 newtype 模式：

```rust
#[derive(Clone)]
struct PrimaryDb(Arc<Database>);
#[derive(Clone)]
struct ReplicaDb(Arc<Database>);

let deps = AlgebraicEffect::new()
    .with(PrimaryDb(primary))
    .with(ReplicaDb(replica))
    .build();

async fn handler(ae: Effectful) -> Result<(), EffectError> {
    let primary = ae.use_::<PrimaryDb>().await?;
    let replica = ae.use_::<ReplicaDb>().await?;
    Ok(())
}
```
