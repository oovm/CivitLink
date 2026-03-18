use serde::Deserialize;
use wae_effect::AlgebraicEffect;

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
    println!("=== WAE Effect 基础示例 ===\n");

    let config = AppConfig { app_name: "WAE Effect Demo".to_string(), version: "0.1.0".to_string() };

    let user = AuthenticatedUser { id: "user-001".to_string(), username: "demo_user".to_string() };

    let deps = AlgebraicEffect::new().with("config", config).with("auth", user).build();

    println!("1. 获取配置:");
    let config: AppConfig = deps.get("config")?;
    println!("   应用名称: {}", config.app_name);
    println!("   版本: {}\n", config.version);

    println!("2. 获取认证用户:");
    let user: AuthenticatedUser = deps.get("auth")?;
    println!("   用户 ID: {}", user.id);
    println!("   用户名: {}\n", user.username);

    println!("=== 示例完成 ===");
    Ok(())
}
