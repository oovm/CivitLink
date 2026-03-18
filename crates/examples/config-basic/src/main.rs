use serde::{Deserialize, Serialize};
use wae_config::ConfigLoader;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct AppConfig {
    app_name: String,
    version: String,
    database: DatabaseConfig,
    server: ServerConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct DatabaseConfig {
    url: String,
    max_connections: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ServerConfig {
    host: String,
    port: u16,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            app_name: "WAE App".to_string(),
            version: "0.1.0".to_string(),
            database: DatabaseConfig { url: "sqlite://:memory:".to_string(), max_connections: 10 },
            server: ServerConfig { host: "127.0.0.1".to_string(), port: 3000 },
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== WAE Config 基础示例 ===\n");

    println!("1. 使用默认配置:");
    let default_config = AppConfig::default();
    println!("   应用名称: {}", default_config.app_name);
    println!("   版本: {}", default_config.version);
    println!("   数据库 URL: {}", default_config.database.url);
    println!("   数据库最大连接数: {}", default_config.database.max_connections);
    println!("   服务器地址: {}:{}", default_config.server.host, default_config.server.port);
    println!();

    println!("2. 使用 ConfigLoader 加载配置（仅默认值）:");
    let config: AppConfig = ConfigLoader::new().with_defaults(&AppConfig::default()).extract()?;
    println!("   应用名称: {}", config.app_name);
    println!("   版本: {}", config.version);
    println!();

    println!("3. 尝试从环境变量加载（设置 APP_NAME）:");
    std::env::set_var("APP_APP_NAME", "My Awesome App");
    std::env::set_var("APP_SERVER_PORT", "8080");

    let config_with_env: AppConfig = ConfigLoader::new().with_defaults(&AppConfig::default()).with_env("APP_").extract()?;
    println!("   应用名称: {}", config_with_env.app_name);
    println!("   服务器端口: {}", config_with_env.server.port);
    println!();

    println!("=== 示例完成 ===");
    Ok(())
}
