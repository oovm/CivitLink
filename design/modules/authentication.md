# wae-authentication

认证服务模块，提供统一的认证能力抽象，支持多种认证方式和后端。本文档将前后端认证功能串联讲解，帮助开发者理解完整的认证流程。

## 概述

认证模块采用前后端分离架构，后端提供认证服务 API，前端提供客户端 SDK。两者通过统一的类型定义确保数据结构一致性。

### 架构设计

```
┌─────────────────────────────────────────────────────────────────┐
│                         前端 (Frontend)                          │
│  ┌─────────────────────────────────────────────────────────┐    │
│  │                    AuthClient                            │    │
│  │  • login() / logout() / refreshToken()                   │    │
│  │  • 状态管理 (AuthState)                                   │    │
│  │  • 自动 Token 刷新                                        │    │
│  │  • 本地存储管理                                           │    │
│  └─────────────────────────────────────────────────────────┘    │
└─────────────────────────────────────────────────────────────────┘
                              │ HTTP API
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│                         后端 (Backend)                           │
│  ┌─────────────────────────────────────────────────────────┐    │
│  │                    AuthService                            │    │
│  │  • JWT 令牌生成与验证                                      │    │
│  │  • OAuth2 授权流程                                        │    │
│  │  • TOTP 双因素认证                                        │    │
│  │  • 用户/角色/权限管理                                      │    │
│  └─────────────────────────────────────────────────────────┘    │
└─────────────────────────────────────────────────────────────────┘
```

### 功能特性

- **JWT 认证**: JSON Web Token 生成、验证和刷新
- **OAuth2**: 授权码流程、客户端凭证流程
- **TOTP**: 基于时间的一次性密码，支持双因素认证
- **API Key**: API 密钥认证
- **会话管理**: 用户会话状态管理
- **权限验证**: 基于角色的访问控制 (RBAC)

---

## 共享类型定义

前后端使用相同的数据结构，确保类型安全。

### UserInfo 用户信息

```typescript
// 前端 TypeScript
interface UserInfo {
    id: UserId;
    username: string;
    email: string | null;
    phone: string | null;
    displayName: string | null;
    avatarUrl: string | null;
    verified: boolean;
    disabled: boolean;
    attributes: Record<string, unknown>;
    createdAt: number;
    updatedAt: number;
}
```

```rust
// 后端 Rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserInfo {
    pub id: UserId,
    pub username: String,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub display_name: Option<String>,
    pub avatar_url: Option<String>,
    pub verified: bool,
    pub disabled: bool,
    pub attributes: HashMap<String, serde_json::Value>,
    pub created_at: i64,
    pub updated_at: i64,
}
```

### Credentials 用户凭证

```typescript
// 前端 TypeScript
interface Credentials {
    identifier: string;
    password: string;
    extra?: Record<string, string>;
}
```

```rust
// 后端 Rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Credentials {
    pub identifier: String,
    pub password: String,
    pub extra: HashMap<String, String>,
}
```

### AuthToken 认证令牌

```typescript
// 前端 TypeScript
interface AuthToken {
    accessToken: string;
    refreshToken: string | null;
    tokenType: string;
    expiresIn: number;
    expiresAt: number;
}
```

```rust
// 后端 Rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthToken {
    pub access_token: String,
    pub refresh_token: Option<String>,
    pub token_type: String,
    pub expires_in: u64,
    pub expires_at: i64,
}
```

### Role 角色信息

```typescript
// 前端 TypeScript
interface Role {
    id: RoleId;
    name: string;
    description: string | null;
    permissions: PermissionCode[];
}
```

```rust
// 后端 Rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Role {
    pub id: RoleId,
    pub name: String,
    pub description: Option<String>,
    pub permissions: Vec<PermissionCode>,
}
```

### TokenValidation Token 验证结果

```typescript
// 前端 TypeScript
interface TokenValidation {
    userId: UserId;
    user: UserInfo | null;
    roles: Role[];
    permissions: PermissionCode[];
    metadata: Record<string, string>;
}
```

```rust
// 后端 Rust
#[derive(Debug, Clone)]
pub struct TokenValidation {
    pub user_id: UserId,
    pub user: Option<UserInfo>,
    pub roles: Vec<Role>,
    pub permissions: Vec<PermissionCode>,
    pub metadata: HashMap<String, String>,
}
```

---

## 认证流程串讲

### 完整登录流程

```
前端                                后端
 │                                   │
 │  1. 用户输入凭证                   │
 │  ──────────────────────────────▶  │
 │  POST /auth/login                 │
 │  { identifier, password }         │
 │                                   │  2. 验证凭证
 │                                   │  3. 生成 JWT Token
 │                                   │  4. 返回 AuthToken
 │  ◀──────────────────────────────  │
 │  { accessToken, refreshToken,     │
 │    expiresIn, expiresAt }         │
 │                                   │
 │  5. 存储到 localStorage           │
 │  6. 安排自动刷新                   │
 │                                   │
```

### 前端实现

```typescript
import { createAuthClient } from "@wae/auth";

const authClient = createAuthClient({
    baseUrl: "https://api.example.com",
    refreshBeforeExpiry: 60000, // 提前 60 秒刷新
});

async function handleLogin(username: string, password: string) {
    try {
        const token = await authClient.login({
            identifier: username,
            password,
        });
        console.log("登录成功", token);
    } catch (error) {
        console.error("登录失败", error);
    }
}
```

### 后端实现

```rust
use wae::authentication::{
    AuthConfig, AuthService, Credentials,
    MemoryAuthService, memory_auth_service,
};

let config = AuthConfig::default();
let auth = memory_auth_service(config);

let token = auth.login(&Credentials {
    identifier: "alice".into(),
    password: "SecurePass123".into(),
    extra: HashMap::new(),
}).await?;
```

---

## 后端 API 详解

### AuthService Trait

认证服务核心接口：

```rust

pub trait AuthService: Send + Sync {
    async fn login(&self, credentials: &Credentials) -> AuthResult<AuthToken>;
    async fn logout(&self, token: &str) -> AuthResult<()>;
    async fn refresh_token(&self, refresh_token: &str) -> AuthResult<AuthToken>;
    async fn validate_token(&self, token: &str) -> AuthResult<TokenValidation>;
    async fn create_user(&self, request: &CreateUserRequest) -> AuthResult<UserInfo>;
    async fn get_user(&self, user_id: &str) -> AuthResult<UserInfo>;
    async fn update_user(&self, user_id: &str, request: &UpdateUserRequest) -> AuthResult<UserInfo>;
    async fn delete_user(&self, user_id: &str) -> AuthResult<()>;
    async fn change_password(&self, user_id: &str, request: &ChangePasswordRequest) -> AuthResult<()>;
    async fn reset_password(&self, identifier: &str) -> AuthResult<()>;
    async fn check_permission(&self, user_id: &str, permission: &str) -> AuthResult<bool>;
    async fn get_user_roles(&self, user_id: &str) -> AuthResult<Vec<Role>>;
    async fn assign_role(&self, user_id: &str, role_id: &str) -> AuthResult<()>;
    async fn remove_role(&self, user_id: &str, role_id: &str) -> AuthResult<()>;
    fn config(&self) -> &AuthConfig;
}
```

### AuthConfig 配置

```rust
#[derive(Debug, Clone)]
pub struct AuthConfig {
    pub token_expires_in: u64,
    pub refresh_token_expires_in: u64,
    pub issuer: String,
    pub audience: String,
    pub password_min_length: usize,
    pub password_require_digit: bool,
    pub password_require_special: bool,
    pub max_login_attempts: u32,
    pub lockout_duration: u64,
}
```

| 配置项 | 默认值 | 说明 |
|--------|--------|------|
| `token_expires_in` | 3600 秒 | Access Token 有效期 |
| `refresh_token_expires_in` | 604800 秒 | Refresh Token 有效期 |
| `issuer` | "wae-authentication" | Token 签发者 |
| `audience` | "wae-api" | Token 受众 |
| `password_min_length` | 8 | 密码最小长度 |
| `password_require_digit` | true | 密码需包含数字 |
| `password_require_special` | false | 密码需包含特殊字符 |
| `max_login_attempts` | 5 | 最大登录尝试次数 |
| `lockout_duration` | 1800 秒 | 锁定时长 |

---

## 前端客户端详解

### AuthClientConfig 配置

```typescript
interface AuthClientConfig extends Partial<HttpClientConfig> {
    tokenKey?: string;
    refreshTokenKey?: string;
    userKey?: string;
    refreshBeforeExpiry?: number;
    loginPath?: string;
    logoutPath?: string;
    refreshTokenPath?: string;
    validateTokenPath?: string;
    userPath?: string;
}
```

| 配置项 | 默认值 | 说明 |
|--------|--------|------|
| `tokenKey` | `"wae_access_token"` | Token 存储键名 |
| `refreshTokenKey` | `"wae_refresh_token"` | Refresh Token 存储键名 |
| `userKey` | `"wae_user"` | 用户信息存储键名 |
| `refreshBeforeExpiry` | `60000` | 提前刷新时间 (毫秒) |
| `loginPath` | `"/auth/login"` | 登录接口路径 |
| `logoutPath` | `"/auth/logout"` | 登出接口路径 |
| `refreshTokenPath` | `"/auth/refresh"` | 刷新 Token 路径 |

### AuthClient 方法

| 方法 | 说明 |
|------|------|
| `login(credentials)` | 用户登录 |
| `logout()` | 用户登出 |
| `refreshToken()` | 刷新令牌 |
| `validateToken()` | 验证令牌 |
| `getUser()` | 获取用户信息 |
| `updateUser(request)` | 更新用户信息 |
| `changePassword(request)` | 修改密码 |
| `createUser(request)` | 创建用户 |
| `getUserRoles()` | 获取角色列表 |
| `checkPermission(permission)` | 检查权限 |
| `getAccessToken()` | 获取访问令牌 |
| `getRefreshToken()` | 获取刷新令牌 |
| `subscribe(listener)` | 订阅状态变化 |

### AuthState 状态管理

```typescript
interface AuthState {
    isAuthenticated: boolean;
    user: UserInfo | null;
    token: AuthToken | null;
    isLoading: boolean;
}

const unsubscribe = authClient.subscribe((state) => {
    if (state.isAuthenticated) {
        console.log("当前用户:", state.user);
    }
});
```

---

## JWT 认证详解

### 后端 JWT 服务

```rust
use wae::authentication::jwt::{
    JwtConfig, JwtService, AccessTokenClaims,
};
use std::time::Duration;

let config = JwtConfig::new("super-secret-key")
    .with_issuer("my-app")
    .with_audience("my-api")
    .with_access_token_ttl(Duration::from_secs(900))
    .with_refresh_token_ttl(Duration::from_secs(604800));

let jwt = JwtService::new(config)?;

let claims = AccessTokenClaims::new("user-123")
    .with_username("bob")
    .with_roles(vec!["admin".into(), "editor".into()])
    .with_permissions(vec!["read".into(), "write".into()]);

let token_pair = jwt.generate_token_pair("user-123", claims, "session-abc")?;

let verified = jwt.verify_access_token(&token_pair.access_token)?;
```

### JwtConfig 配置

```rust
#[derive(Debug, Clone)]
pub struct JwtConfig {
    pub secret: String,
    pub public_key: Option<String>,
    pub algorithm: JwtAlgorithm,
    pub issuer: Option<String>,
    pub audience: Option<String>,
    pub access_token_ttl: Duration,
    pub refresh_token_ttl: Duration,
    pub validate_issuer: bool,
    pub validate_audience: bool,
    pub leeway_seconds: i64,
}
```

### JwtAlgorithm 支持的算法

```rust
pub enum JwtAlgorithm {
    HS256, HS384, HS512,  // HMAC
    RS256, RS384, RS512,  // RSA
    ES256, ES384,         // ECDSA
}
```

---

## OAuth2 登录流程

### 流程图

```
前端                                后端                    OAuth2 Provider
 │                                   │                           │
 │  1. 获取授权 URL                   │                           │
 │  ◀─────────────────────────────   │                           │
 │                                   │                           │
 │  2. 重定向到 OAuth2 Provider       │                           │
 │  ─────────────────────────────────────────────────────────▶   │
 │                                   │                           │
 │  3. 用户授权                       │                           │
 │  ◀────────────────────────────────────────────────────────   │
 │  回调带 code                       │                           │
 │                                   │                           │
 │  4. 用 code 换取 token             │                           │
 │  ──────────────────────────────▶  │  5. 换取 token            │
 │                                   │  ───────────────────────▶ │
 │                                   │  ◀─────────────────────── │
 │                                   │  { access_token }         │
 │  ◀──────────────────────────────  │                           │
 │  返回用户信息                       │                           │
```

### 后端 OAuth2 客户端

```rust
use wae::authentication::oauth2::{
    OAuth2Client, OAuth2ClientConfig, OAuth2ProviderConfig,
};

let provider = OAuth2ProviderConfig::new(
    "github",
    env!("GITHUB_CLIENT_ID"),
    env!("GITHUB_CLIENT_SECRET"),
    "http://localhost:8080/auth/callback",
)
.with_authorization_url("https://github.com/login/oauth/authorize")
.with_token_url("https://github.com/login/oauth/access_token")
.with_userinfo_url("https://api.github.com/user")
.add_scope("user:email");

let config = OAuth2ClientConfig::new(provider)
    .with_pkce(true)
    .with_state_validation(true);

let client = OAuth2Client::new(config)?;

let auth_url = client.authorization_url()?;
let token = client.exchange_code("auth-code", auth_url.code_verifier.as_deref()).await?;
let user_info = client.get_user_info(&token.access_token).await?;
```

---

## TOTP 双因素认证

### 后端 TOTP 服务

```rust
use wae::authentication::totp::TotpService;

let service = TotpService::create("MyApp", "user@example.com")?;

println!("Secret: {}", service.secret().as_base32());
println!("URI: {}", service.to_uri());

let code = service.generate_code()?;
println!("Current code: {}", code);

if service.verify("123456")? {
    println!("Valid!");
}
```

### TotpConfig 配置

```rust
#[derive(Debug, Clone)]
pub struct TotpConfig {
    pub issuer: String,
    pub account_name: String,
    pub secret: String,
    pub algorithm: TotpAlgorithm,
    pub digits: u32,
    pub time_step: u64,
    pub valid_window: u32,
}
```

| 配置项 | 默认值 | 说明 |
|--------|--------|------|
| `algorithm` | SHA1 | 哈希算法 |
| `digits` | 6 | 验证码位数 |
| `time_step` | 30 秒 | 时间步长 |
| `valid_window` | 1 | 有效窗口 (前后各允许 1 步) |

---

## API Key 认证

### ApiKeyAuth Trait

```rust

pub trait ApiKeyAuth: Send + Sync {
    async fn validate_api_key(&self, api_key: &str) -> AuthResult<TokenValidation>;
    async fn create_api_key(&self, user_id: &str, name: &str, expires_in: Option<u64>) -> AuthResult<String>;
    async fn revoke_api_key(&self, api_key: &str) -> AuthResult<()>;
    async fn list_api_keys(&self, user_id: &str) -> AuthResult<Vec<ApiKeyInfo>>;
}
```

### ApiKeyInfo

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiKeyInfo {
    pub id: String,
    pub name: String,
    pub prefix: String,
    pub created_at: i64,
    pub expires_at: Option<i64>,
    pub last_used_at: Option<i64>,
}
```

---

## 错误处理

### AuthError 错误类型

```rust
#[derive(Debug)]
pub enum AuthError {
    AuthenticationFailed(String),
    InvalidCredentials,
    InvalidToken(String),
    TokenExpired,
    PermissionDenied(String),
    UserNotFound(String),
    UserAlreadyExists(String),
    PasswordRequirement(String),
    AccountLocked,
    AccountNotVerified,
    Timeout(String),
    Internal(String),
}
```

| 错误类型 | 说明 | HTTP 状态码 |
|---------|------|------------|
| `AuthenticationFailed` | 认证失败 | 401 |
| `InvalidCredentials` | 无效凭证 | 401 |
| `InvalidToken` | Token 无效 | 401 |
| `TokenExpired` | Token 过期 | 401 |
| `PermissionDenied` | 权限不足 | 403 |
| `UserNotFound` | 用户不存在 | 404 |
| `UserAlreadyExists` | 用户已存在 | 409 |
| `PasswordRequirement` | 密码不符合要求 | 400 |
| `AccountLocked` | 账户被锁定 | 423 |
| `AccountNotVerified` | 账户未验证 | 403 |

### 前端错误处理

```typescript
import { CloudError, isAuthenticationError } from 'wae-types';

function handleError(error: CloudError) {
    switch (error.type) {
        case "Authentication":
            console.error("认证失败:", error.message);
            break;
        case "RateLimit":
            console.error("请求过于频繁:", error.message);
            break;
        default:
            console.error("未知错误:", error.message);
    }
}
```

---

## 完整使用示例

### 后端服务搭建

```rust
use wae::authentication::{
    AuthConfig, AuthService, Credentials, CreateUserRequest,
    MemoryAuthService, memory_auth_service,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = AuthConfig::default();
    let auth = memory_auth_service(config);

    let user = auth.create_user(&CreateUserRequest {
        username: "alice".into(),
        password: "SecurePass123".into(),
        email: Some("alice@example.com".into()),
        phone: None,
        display_name: Some("Alice".into()),
        attributes: HashMap::new(),
    }).await?;

    auth.assign_role(&user.id, "user").await?;

    Ok(())
}
```

### 前端 React 集成

```typescript
import { createAuthClient, type AuthState } from "@wae/auth";
import { useEffect, useState } from "react";

const authClient = createAuthClient({
    baseUrl: import.meta.env.VITE_API_URL,
});

export function useAuth() {
    const [state, setState] = useState<AuthState>(authClient.getState());

    useEffect(() => {
        return authClient.subscribe(setState);
    }, []);

    return {
        ...state,
        login: authClient.login.bind(authClient),
        logout: authClient.logout.bind(authClient),
        getUser: authClient.getUser.bind(authClient),
    };
}

function LoginPage() {
    const { login, isLoading, isAuthenticated } = useAuth();

    const handleSubmit = async (e: React.FormEvent) => {
        e.preventDefault();
        await login({ identifier: "user", password: "pass" });
    };

    if (isAuthenticated) {
        return <div>已登录</div>;
    }

    return (
        <form onSubmit={handleSubmit}>
            <button disabled={isLoading}>
                {isLoading ? "登录中..." : "登录"}
            </button>
        </form>
    );
}
```

---

## 自动 Token 刷新机制

前端 `AuthClient` 内置自动刷新机制：

1. 登录成功后，根据令牌过期时间自动安排刷新
2. 在令牌过期前 `refreshBeforeExpiry` 毫秒自动刷新
3. 刷新失败时自动清除认证状态
4. 登出时清除刷新定时器

```
Token 生命周期:

  创建时间                    刷新时间点                  过期时间
     │                           │                          │
     ├───────────────────────────┼──────────────────────────┤
     │                           │                          │
     │◀───── 有效期 ────────────▶│◀── refreshBeforeExpiry ──▶│
     │                           │                          │
     │                      自动刷新                        │
```

---

## 自定义存储适配器

前端支持自定义存储实现：

```typescript
import { AuthClient, type StorageAdapter } from "@wae/auth";

class MemoryStorageAdapter implements StorageAdapter {
    private store = new Map<string, string>();

    getItem(key: string): string | null {
        return this.store.get(key) ?? null;
    }

    setItem(key: string, value: string): void {
        this.store.set(key, value);
    }

    removeItem(key: string): void {
        this.store.delete(key);
    }
}

const authClient = new AuthClient(
    { baseUrl: "https://api.example.com" },
    new MemoryStorageAdapter()
);
```
