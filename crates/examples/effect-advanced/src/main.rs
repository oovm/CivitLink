use serde::{Deserialize, Serialize};
use wae_effect::{AlgebraicEffect, Dependencies};

#[derive(Debug, Clone, Serialize, Deserialize)]
struct DatabaseConfig {
    url: String,
    max_connections: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct AppConfig {
    app_name: String,
    version: String,
    database: DatabaseConfig,
}

#[derive(Debug, Clone)]
struct UserRepository {
    users: Vec<User>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct User {
    id: String,
    name: String,
    email: String,
}

impl UserRepository {
    fn new() -> Self {
        Self {
            users: vec![
                User { id: "user-001".to_string(), name: "张三".to_string(), email: "zhangsan@example.com".to_string() },
                User { id: "user-002".to_string(), name: "李四".to_string(), email: "lisi@example.com".to_string() },
            ],
        }
    }

    fn get_user(&self, id: &str) -> Option<&User> {
        self.users.iter().find(|u| u.id == id)
    }

    fn get_all_users(&self) -> &[User] {
        &self.users
    }
}

#[derive(Debug, Clone)]
struct LoggingService {
    service_name: String,
}

impl LoggingService {
    fn new(service_name: String) -> Self {
        Self { service_name }
    }

    fn info(&self, message: &str) {
        println!("[INFO] [{}] {}", self.service_name, message);
    }

    fn error(&self, message: &str) {
        println!("[ERROR] [{}] {}", self.service_name, message);
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== WAE Effect 进阶示例 ===\n");

    println!("1. 创建配置对象:");
    let config = AppConfig {
        app_name: "WAE Effect Demo".to_string(),
        version: "1.0.0".to_string(),
        database: DatabaseConfig { url: "postgres://localhost:5432/wae".to_string(), max_connections: 20 },
    };
    println!("   配置创建完成\n");

    println!("2. 创建服务实例:");
    let user_repo = UserRepository::new();
    let logging = LoggingService::new("effect-demo".to_string());
    println!("   服务实例创建完成\n");

    println!("3. 使用 AlgebraicEffect 构建依赖容器:");
    let deps = AlgebraicEffect::new()
        .with("config", config.clone())
        .with("user_repo", user_repo.clone())
        .with("logging", logging.clone())
        .build();
    println!("   依赖容器构建完成\n");

    println!("4. 使用字符串键获取依赖:");
    let retrieved_config: AppConfig = deps.get("config")?;
    println!("   应用名称: {}", retrieved_config.app_name);
    println!("   版本: {}", retrieved_config.version);
    println!("   数据库 URL: {}", retrieved_config.database.url);
    println!();

    println!("5. 使用用户仓储服务:");
    let repo: UserRepository = deps.get("user_repo")?;
    let users = repo.get_all_users();
    println!("   用户数量: {}", users.len());
    for user in users {
        println!("   - {} ({})", user.name, user.email);
    }
    println!();

    println!("6. 按类型注册和获取依赖:");
    let mut typed_deps = Dependencies::new();
    typed_deps.register_type(config.clone());
    typed_deps.register_type(user_repo.clone());

    let config_by_type: AppConfig = typed_deps.get_type()?;
    println!("   通过类型获取配置: {}", config_by_type.app_name);
    println!();

    println!("7. 使用日志服务:");
    let logger: LoggingService = deps.get("logging")?;
    logger.info("这是一条信息日志");
    logger.error("这是一条错误日志");
    println!();

    println!("8. 演示服务协作:");
    let user_id = "user-001";
    if let Some(user) = repo.get_user(user_id) {
        logger.info(&format!("找到用户: {} ({})", user.name, user.email));
    }
    else {
        logger.error(&format!("未找到用户 ID: {}", user_id));
    }
    println!();

    println!("=== 示例总结 ===");
    println!("本示例展示了:");
    println!("  - 使用字符串键注册和获取依赖");
    println!("  - 使用类型安全的方式注册和获取依赖");
    println!("  - 多个服务之间的协作");
    println!("  - 配置管理与服务分离");
    println!();

    Ok(())
}
