# WAE Effect - 代数效应风格的依赖注入模块

提供声明式的依赖获取能力，通过 `Effectful` 参数统一管理所有依赖。

## 设计理念

传统模式需要在函数签名中声明所有依赖：
```rust,ignore
async fn handler(
    State(db): State<Database>,
    State(cache): State<Cache>,
    Extension(user): Extension<AuthenticatedUser>,
) { ... }
```

代数效应模式通过单一参数统一管理：
```rust,ignore
async fn handler(e: Effectful) {
    let db = e.use_database().await?;
    let user = e.use_auth().await?;
}
```

## 优势

- **声明式**: 依赖在函数体内按需获取
- **类型安全**: 编译时检查依赖类型
- **可测试**: 轻松 mock 依赖
- **组合性**: 依赖可以嵌套组合

---

代数效应模块 - 提供依赖注入和代数效应系统。

## 主要功能

- **依赖注入**: 类型安全的依赖管理
- **代数效应**: 函数式编程风格的副作用处理
- **上下文管理**: 请求上下文传递
- **组合式 API**: 链式构建依赖容器

## 核心概念

- `AlgebraicEffect` - 效应构建器
- `Dependencies` - 依赖容器
- `Effectful` - 效应执行器

## 使用示例

```rust
use wae_effect::{AlgebraicEffect, Dependencies, Effectful};
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
struct AppConfig {
    app_name: String,
    version: String,
}

#[derive(Debug, Clone)]
struct User {
    id: String,
    username: String,
}

#[tokio::main]
async fn main() {
    let deps = AlgebraicEffect::new()
        .with_config(AppConfig {
            app_name: "My App".to_string(),
            version: "1.0.0".to_string(),
        })
        .with_auth(User {
            id: "001".to_string(),
            username: "admin".to_string(),
        })
        .build();
    
    let config = deps.get::<AppConfig>()?;
    let user = deps.get::<User>()?;
}
```

## 在 Axum 中使用

```rust
use axum::Router;
use wae_effect::Effectful;

async fn handler(ae: Effectful) -> Result<Json<Response>, EffectError> {
    let config = ae.use_config::<AppConfig>().await?;
    let user = ae.use_auth::<User>().await?;
    Ok(Json(Response { user }))
}

let app = Router::new()
    .route("/api", get(handler))
    .with_state(deps);
```
