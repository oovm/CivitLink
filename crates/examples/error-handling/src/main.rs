use serde::{Deserialize, Serialize};
use wae_https::{
    HttpsServerBuilder,
    error::{ErrorExt, ErrorResponse, HttpError, HttpResult},
    response::JsonResponse,
    router::RouterBuilder,
};
use wae_types::{WaeError, WaeResult};

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

fn validate_user(user: &User) -> WaeResult<()> {
    if user.name.is_empty() {
        return Err(WaeError::required("name"));
    }
    if user.name.len() > 50 {
        return Err(WaeError::invalid_params("name", "must be at most 50 characters"));
    }
    if !user.email.contains('@') {
        return Err(WaeError::invalid_format("email", "must contain @"));
    }
    Ok(())
}

fn find_user(id: u64) -> WaeResult<User> {
    if id == 1 {
        Ok(User { id: Some(1), name: "张三".to_string(), email: "zhangsan@example.com".to_string() })
    }
    else {
        Err(WaeError::not_found("user", id.to_string()))
    }
}

async fn demonstrate_errors() {
    println!("=== 错误处理演示 ===\n");

    println!("1. 验证错误演示:");
    let invalid_user = User { id: None, name: "".to_string(), email: "invalid-email".to_string() };
    match validate_user(&invalid_user) {
        Ok(_) => println!("验证通过（不应该发生）"),
        Err(err) => {
            println!("验证失败: {}", err);
            let http_error = HttpError::from(err);
            println!("HTTP 错误代码: {}", http_error.i18n_key());
        }
    }
    println!();

    println!("2. 资源未找到错误演示:");
    match find_user(999) {
        Ok(user) => println!("找到用户: {:?}", user),
        Err(err) => {
            println!("未找到用户: {}", err);
            let error_response = ErrorResponse::from_wae_error(&err);
            println!("错误响应 JSON: {}", serde_json::to_string_pretty(&error_response).unwrap());
        }
    }
    println!();

    println!("3. 便捷错误转换演示:");
    let result: WaeResult<()> = Err(WaeError::internal("数据库连接失败"));
    match result.bad_request() {
        Ok(_) => println!("成功（不应该发生）"),
        Err(err) => println!("转换为 BAD_REQUEST: {}", err),
    }
    println!();
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== WAE Error Handling 示例 ===\n");

    demonstrate_errors().await;

    let mut router = RouterBuilder::new();

    router = router.get("/", || async {
        JsonResponse::success(ApiStatus {
            service: "wae-error-handling".to_string(),
            version: "0.1.0".to_string(),
            status: "running".to_string(),
        })
    });

    router = router.get("/health", || async {
        JsonResponse::success(ApiStatus {
            service: "wae-error-handling".to_string(),
            version: "0.1.0".to_string(),
            status: "healthy".to_string(),
        })
    });

    let router = router.build();

    println!("已配置以下路由:");
    println!("  GET    /                    - 服务信息");
    println!("  GET    /health              - 健康检查");
    println!();

    println!("服务器将在 http://127.0.0.1:3000 启动");
    println!();

    let server =
        HttpsServerBuilder::new().addr("127.0.0.1:3000".parse()?).service_name("wae-error-handling").router(router).build();

    println!("=== 示例说明 ===");
    println!("本示例展示了 WAE 框架的统一错误处理机制，包括:");
    println!("  - 统一的错误类型定义");
    println!("  - 错误分类和 HTTP 状态码映射");
    println!("  - 错误响应格式化");
    println!("  - 便捷的错误转换方法");
    println!("  - 验证错误处理");
    println!("  - 资源未找到错误处理");
    println!();

    Ok(())
}
