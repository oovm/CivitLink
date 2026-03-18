# https-basic

HTTP/HTTPS 基础示例 - 演示 WAE HTTPS 模块的基本用法。

## 示例说明

本示例展示如何使用 `wae-https` 模块构建 HTTP 服务器，包括路由配置、请求处理和响应格式化。

## 主要功能演示

- 路由构建和配置
- GET/POST/PUT/DELETE 等 HTTP 方法支持
- JSON 响应格式化
- API 标准化响应结构
- 健康检查端点

## 运行示例

```bash
cd examples/https-basic
cargo run
```

## 代码示例

```rust
use serde::{Deserialize, Serialize};
use wae_https::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct User {
    id: Option<u64>,
    name: String,
    email: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut router = RouterBuilder::new();

    router = router.get("/", || {
        JsonResponse::success("Welcome to WAE HTTPS!")
    });

    router = router.get("/users", || {
        let users = vec![
            User {
                id: Some(1),
                name: "张三".to_string(),
                email: "zhangsan@example.com".to_string(),
            },
        ];
        JsonResponse::success(users)
    });

    router = router.post("/users", || {
        let user = User {
            id: Some(3),
            name: "新用户".to_string(),
            email: "newuser@example.com".to_string(),
        };
        JsonResponse::success(user)
    });

    let router = router.build();

    let server = HttpsServerBuilder::new()
        .addr("127.0.0.1:3000".parse()?)
        .service_name("wae-https-basic")
        .router(router)
        .build();

    server.serve().await?;

    Ok(())
}
```

## API 端点

| 方法 | 路径 | 描述 |
|------|------|------|
| GET | / | 服务信息 |
| GET | /health | 健康检查 |
| GET | /users | 获取用户列表 |
| GET | /users/:id | 获取用户详情 |
| POST | /users | 创建用户 |
| PUT | /users/:id | 更新用户 |
| DELETE | /users/:id | 删除用户 |

## 输出示例

```
=== WAE HTTPS 基础示例 ===

已配置以下路由:
  GET    /                    - 服务信息
  GET    /health              - 健康检查
  GET    /users               - 获取用户列表
  GET    /users/:id           - 获取用户详情
  POST   /users               - 创建用户
  PUT    /users/:id           - 更新用户
  DELETE /users/:id           - 删除用户

服务器将在 http://127.0.0.1:3000 启动

=== 示例说明 ===
注意：本示例演示了路由配置和 API 响应格式化
实际的 HTTP 服务器功能正在开发中
```

## 依赖

- `wae-https` - HTTP/HTTPS 服务核心模块
- `wae-types` - 通用类型定义
- `tokio` - 异步运行时
- `serde` - 序列化/反序列化
