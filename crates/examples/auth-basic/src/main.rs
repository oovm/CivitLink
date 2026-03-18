use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct User {
    id: String,
    username: String,
    email: String,
    password_hash: String,
    roles: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct LoginRequest {
    username: String,
    password: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct AuthToken {
    access_token: String,
    token_type: String,
    expires_in: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct UserProfile {
    id: String,
    username: String,
    email: String,
    roles: Vec<String>,
}

struct MockAuthService {
    users: HashMap<String, User>,
    tokens: HashMap<String, String>,
}

impl MockAuthService {
    fn new() -> Self {
        let mut users = HashMap::new();

        users.insert(
            "admin".to_string(),
            User {
                id: "user-001".to_string(),
                username: "admin".to_string(),
                email: "admin@example.com".to_string(),
                password_hash: "hashed_admin_password".to_string(),
                roles: vec!["admin".to_string(), "user".to_string()],
            },
        );

        users.insert(
            "user".to_string(),
            User {
                id: "user-002".to_string(),
                username: "user".to_string(),
                email: "user@example.com".to_string(),
                password_hash: "hashed_user_password".to_string(),
                roles: vec!["user".to_string()],
            },
        );

        Self { users, tokens: HashMap::new() }
    }

    fn login(&mut self, request: LoginRequest) -> Option<AuthToken> {
        let user = self.users.get(&request.username)?;

        let is_valid = match request.username.as_str() {
            "admin" => request.password == "admin123",
            "user" => request.password == "user123",
            _ => false,
        };

        if !is_valid {
            return None;
        }

        let token = format!("token_{}_{}", user.id, chrono::Utc::now().timestamp());
        self.tokens.insert(token.clone(), user.id.clone());

        Some(AuthToken { access_token: token, token_type: "Bearer".to_string(), expires_in: 3600 })
    }

    fn validate_token(&self, token: &str) -> Option<UserProfile> {
        let user_id = self.tokens.get(token)?;
        let user = self.users.values().find(|u| u.id == *user_id)?;

        Some(UserProfile {
            id: user.id.clone(),
            username: user.username.clone(),
            email: user.email.clone(),
            roles: user.roles.clone(),
        })
    }

    fn has_role(&self, user: &UserProfile, role: &str) -> bool {
        user.roles.contains(&role.to_string())
    }

    fn has_permission(&self, user: &UserProfile, permission: &str) -> bool {
        match permission {
            "read" => user.roles.iter().any(|r| r == "admin" || r == "user"),
            "write" => user.roles.iter().any(|r| r == "admin"),
            "delete" => user.roles.iter().any(|r| r == "admin"),
            _ => false,
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== WAE Authentication 基础示例 ===\n");

    println!("1. 初始化认证服务:");
    let mut auth_service = MockAuthService::new();
    println!("   认证服务初始化完成\n");

    println!("2. 用户登录 (admin):");
    let admin_login = LoginRequest { username: "admin".to_string(), password: "admin123".to_string() };
    if let Some(token) = auth_service.login(admin_login) {
        println!("   登录成功!");
        println!("   Access Token: {}", token.access_token);
        println!("   Token Type: {}", token.token_type);
        println!("   Expires In: {} seconds", token.expires_in);

        let admin_token = token.access_token.clone();
        println!();

        println!("3. 验证 Token 并获取用户信息:");
        if let Some(profile) = auth_service.validate_token(&admin_token) {
            println!("   Token 验证成功!");
            println!("   用户 ID: {}", profile.id);
            println!("   用户名: {}", profile.username);
            println!("   邮箱: {}", profile.email);
            println!("   角色: {:?}", profile.roles);
            println!();

            println!("4. 检查角色和权限:");
            println!("   是否是 admin 角色: {}", auth_service.has_role(&profile, "admin"));
            println!("   是否有 read 权限: {}", auth_service.has_permission(&profile, "read"));
            println!("   是否有 write 权限: {}", auth_service.has_permission(&profile, "write"));
            println!("   是否有 delete 权限: {}", auth_service.has_permission(&profile, "delete"));
        }
    }
    println!();

    println!("5. 用户登录 (普通用户):");
    let user_login = LoginRequest { username: "user".to_string(), password: "user123".to_string() };
    if let Some(token) = auth_service.login(user_login) {
        println!("   登录成功!");
        let user_token = token.access_token.clone();
        println!();

        println!("6. 验证普通用户 Token:");
        if let Some(profile) = auth_service.validate_token(&user_token) {
            println!("   Token 验证成功!");
            println!("   用户名: {}", profile.username);
            println!("   角色: {:?}", profile.roles);
            println!();

            println!("7. 检查普通用户权限:");
            println!("   是否是 admin 角色: {}", auth_service.has_role(&profile, "admin"));
            println!("   是否有 read 权限: {}", auth_service.has_permission(&profile, "read"));
            println!("   是否有 write 权限: {}", auth_service.has_permission(&profile, "write"));
        }
    }
    println!();

    println!("8. 测试错误密码:");
    let wrong_login = LoginRequest { username: "admin".to_string(), password: "wrongpassword".to_string() };
    if auth_service.login(wrong_login).is_none() {
        println!("   登录失败 (密码错误) - 预期行为");
    }
    println!();

    println!("9. 测试无效 Token:");
    if auth_service.validate_token("invalid_token").is_none() {
        println!("   Token 验证失败 (无效 Token) - 预期行为");
    }
    println!();

    println!("=== 示例总结 ===");
    println!("本示例展示了基本的认证与授权功能:");
    println!("  - 用户登录 (用户名/密码认证)");
    println!("  - Token 生成与验证");
    println!("  - 基于角色的访问控制 (RBAC)");
    println!("  - 权限检查");
    println!();
    println!("注意: 本示例使用内存数据结构模拟认证服务");
    println!("实际使用时需要配置真实的数据库和 JWT 密钥");
    println!();

    Ok(())
}
