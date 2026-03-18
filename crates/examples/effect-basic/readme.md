# effect-basic

代数效应基础示例 - 演示 WAE Effect 模块的基本用法。

## 示例说明

本示例展示如何使用 `wae-effect` 模块构建依赖注入容器，实现类型安全的依赖管理。

## 主要功能演示

- 创建代数效应容器
- 注册配置和认证用户
- 从容器中获取依赖
- 检查依赖存在性

## 运行示例

```bash
cd examples/effect-basic
cargo run
```

## 代码示例

```rust
use wae_effect::{AlgebraicEffect, Dependencies};
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
struct AppConfig {
    app_name: String,
    version: String,
}

#[derive(Debug, Clone)]
struct AuthenticatedUser {
    id: String,
    username: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = AppConfig {
        app_name: "WAE Effect Demo".to_string(),
        version: "0.1.0".to_string(),
    };

    let user = AuthenticatedUser {
        id: "user-001".to_string(),
        username: "demo_user".to_string(),
    };

    let deps = AlgebraicEffect::new()
        .with_config(config)
        .with_auth(user)
        .build();

    let config = deps.get::<AppConfig>()?;
    let user = deps.get::<AuthenticatedUser>()?;

    println!("应用名称: {}", config.app_name);
    println!("用户: {}", user.username);

    Ok(())
}
```

## 输出示例

```
=== WAE Effect 基础示例 ===

1. 获取配置:
   应用名称: WAE Effect Demo
   版本: 0.1.0

2. 获取认证用户:
   用户 ID: user-001
   用户名: demo_user

3. 检查依赖存在性:
   配置存在: true
   用户存在: true

=== 示例完成 ===
```

## 依赖

- `wae-effect` - 代数效应模块
- `wae-database` - 数据库模块 (可选)
- `wae-config` - 配置模块 (可选)
