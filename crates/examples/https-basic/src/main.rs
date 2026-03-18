use serde::{Deserialize, Serialize};
use wae_https::{HttpsServerBuilder, response::JsonResponse, router::RouterBuilder};

#[derive(Debug, Clone, Serialize, Deserialize)]
struct User {
    id: Option<u64>,
    name: String,
    email: String,
}

#[derive(Debug, Clone, Serialize)]
struct ApiStatus {
    service: String,
    version: String,
    status: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== WAE HTTPS 基础示例 ===\n");

    let mut router = RouterBuilder::new();

    router = router.get("/", || async {
        println!("收到根路径请求");
        JsonResponse::success(ApiStatus {
            service: "wae-https-basic".to_string(),
            version: "0.1.0".to_string(),
            status: "running".to_string(),
        })
    });

    router = router.get("/health", || async {
        println!("收到健康检查请求");
        JsonResponse::success(ApiStatus {
            service: "wae-https-basic".to_string(),
            version: "0.1.0".to_string(),
            status: "healthy".to_string(),
        })
    });

    router = router.get("/users", || async {
        println!("收到获取用户列表请求");
        let users = vec![
            User { id: Some(1), name: "张三".to_string(), email: "zhangsan@example.com".to_string() },
            User { id: Some(2), name: "李四".to_string(), email: "lisi@example.com".to_string() },
        ];
        JsonResponse::success(users)
    });

    router = router.get("/users/:id", || async {
        println!("收到获取用户详情请求");
        let user = User { id: Some(1), name: "张三".to_string(), email: "zhangsan@example.com".to_string() };
        JsonResponse::success(user)
    });

    router = router.post("/users", || async {
        println!("收到创建用户请求");
        let user = User { id: Some(3), name: "新用户".to_string(), email: "newuser@example.com".to_string() };
        JsonResponse::success(user)
    });

    router = router.put("/users/:id", || async {
        println!("收到更新用户请求");
        let user =
            User { id: Some(1), name: "张三（已更新）".to_string(), email: "zhangsan_updated@example.com".to_string() };
        JsonResponse::success(user)
    });

    router = router.delete("/users/:id", || async {
        println!("收到删除用户请求");
        JsonResponse::success(())
    });

    let router = router.build();

    println!("已配置以下路由:");
    println!("  GET    /                    - 服务信息");
    println!("  GET    /health              - 健康检查");
    println!("  GET    /users               - 获取用户列表");
    println!("  GET    /users/:id           - 获取用户详情");
    println!("  POST   /users               - 创建用户");
    println!("  PUT    /users/:id           - 更新用户");
    println!("  DELETE /users/:id           - 删除用户");
    println!();

    println!("服务器将在 http://127.0.0.1:3000 启动");
    println!();

    let server =
        HttpsServerBuilder::new().addr("127.0.0.1:3000".parse()?).service_name("wae-https-basic").router(router).build();

    println!("=== 示例说明 ===");
    println!("注意：本示例演示了路由配置和 API 响应格式化");
    println!("实际的 HTTP 服务器功能正在开发中");
    println!();

    Ok(())
}
