# wae-macros

宏模块 - 提供过程宏支持。

## 主要功能

- **派生宏**: 自动实现常用 trait
- **属性宏**: 简化代码编写
- **代码生成**: 编译时代码生成

## 提供的宏

| 宏 | 说明 |
|----|------|
| `Effect` | 自动实现代数效应相关 trait |
| `Service` | 服务注册宏 |
| `Route` | 路由定义宏 |

## 使用示例

```rust
use wae_macros::Effect;

#[derive(Debug, Clone, Effect)]
struct AppConfig {
    app_name: String,
    version: String,
}

#[derive(Debug, Clone, Effect)]
struct DatabaseConfig {
    url: String,
    max_connections: u32,
}
```

## 服务宏

```rust
use wae_macros::Service;

#[Service]
pub struct UserService {
    db: DatabasePool,
}

impl UserService {
    pub async fn find_by_id(&self, id: &str) -> Result<Option<User>, Error> {
        self.db.find_by_id(id).await
    }
}
```
