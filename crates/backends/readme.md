# WAE Backend 架构规范

本文档定义 WAE 后端模块的架构规范，所有模块必须遵循。

## 分层架构

```
┌─────────────────────────────────────────────────────────────┐
│                      Application Layer                       │
│                         (wae crate)                          │
├─────────────────────────────────────────────────────────────┤
│                       Core Layer                             │
│  wae-https │ wae-authentication │ wae-storage │ wae-ai ...  │
├─────────────────────────────────────────────────────────────┤
│                      Foundation Layer                        │
│              wae-types (错误类型、基础工具)                    │
└─────────────────────────────────────────────────────────────┘
```

### 依赖方向

- **单向依赖**：上层依赖下层，禁止反向依赖
- **横向隔离**：同层模块之间禁止直接依赖，通过 `wae-types` 共享类型

## 模块职责

### Foundation Layer

| Crate | 职责 | 禁止 |
|-------|------|------|
| `wae-types` | 统一错误类型、基础工具函数、通用 trait | 业务逻辑、外部依赖 |

### Core Layer

| Crate | 职责 | 禁止 |
|-------|------|------|
| `wae-https` | HTTP 服务、路由、中间件、TLS | 业务处理 |
| `wae-authentication` | JWT、OAuth2、SAML、TOTP | 用户管理 |
| `wae-storage` | 对象存储抽象 | 队列、缓存 |
| `wae-ai` | AI 能力抽象 | 任务调度 |
| `wae-database` | 数据库连接、ORM | 业务查询 |
| `wae-cache` | 缓存抽象 | 持久化 |
| `wae-queue` | 消息队列抽象 | 任务执行 |
| `wae-resilience` | 熔断、限流、重试 | 业务逻辑 |
| `wae-scheduler` | 定时任务调度 | 任务实现 |
| `wae-email` | 邮件发送 | 模板渲染 |
| `wae-session` | 会话管理 | 认证逻辑 |
| `wae-service` | 服务发现、注册 | 服务实现 |
| `wae-websocket` | WebSocket 通信 | 业务协议 |
| `wae-request` | HTTP 客户端 | 服务端逻辑 |
| `wae-event` | 事件总线 | 事件处理 |
| `wae-distributed` | 分布式锁、协调 | 业务逻辑 |
| `wae-schema` | 数据验证 | 数据存储 |
| `wae-effect` | 副作用管理 | 状态管理 |
| `wae-config` | 配置管理 | 配置源 |
| `wae-testing` | 测试工具 | 生产代码 |
| `wae-tools` | 开发工具 | 运行时依赖 |
| `wae-macros` | 过程宏 | 运行时逻辑 |

## 设计模式

### Capability Pattern (能力模式)

用于 `wae-ai` 等需要多服务商适配的模块：

```rust
// 1. 定义能力接口
pub trait TextToImageCapability: Send + Sync {
    async fn generate(&self, params: &TextToImageParams) -> AiResult<Vec<String>>;
}

// 2. 实现服务商适配
pub struct HunyuanProvider;
impl TextToImageCapability for HunyuanProvider { ... }

pub struct VolcengineProvider;
impl TextToImageCapability for VolcengineProvider { ... }

// 3. 业务层选择实现
let provider: Box<dyn TextToImageCapability> = match config.provider {
    ProviderType::Hunyuan => Box::new(HunyuanProvider),
    ProviderType::Volcengine => Box::new(VolcengineProvider),
};
```

**规范**：
- 每个 Capability 是独立的 trait
- Provider 实现必须无状态
- 配置通过参数传递，不存储在 Provider 中

### Provider Pattern (提供商模式)

用于 `wae-storage` 等需要多后端支持的模块：

```rust
pub trait StorageProvider: Send + Sync {
    fn get_presigned_put_url(&self, key: &str, config: &StorageConfig) -> StorageResult<Url>;
    fn sign_url(&self, path: &str, config: &StorageConfig) -> StorageResult<Url>;
}

pub struct StorageService;
impl StorageService {
    pub fn get_provider(ty: &StorageProviderType) -> Box<dyn StorageProvider> {
        match ty {
            StorageProviderType::Cos => Box::new(CosProvider),
            StorageProviderType::Oss => Box::new(OssProvider),
            StorageProviderType::Local => Box::new(LocalStorageProvider),
        }
    }
}
```

**规范**：
- Service 层提供统一入口
- Provider 层实现具体逻辑
- 配置通过 `Config` 结构体传递

### Pipeline Pattern (管道模式)

用于 `wae-resilience` 等需要组合处理的模块：

```rust
let pipeline = ResiliencePipelineBuilder::new()
    .with_rate_limiter(rate_limiter)
    .with_circuit_breaker(circuit_breaker)
    .with_retry(retry_policy)
    .with_timeout(timeout_config)
    .build();

pipeline.execute(|| async_operation()).await?;
```

**规范**：
- 使用 Builder 模式构建
- 组件可独立配置
- 执行顺序由框架保证

## 错误处理规范

### 统一错误类型

所有模块使用 `wae-types::WaeError`：

```rust
pub struct WaeError {
    kind: Box<WaeErrorKind>,
}
```

### 错误分类

```rust
pub enum WaeErrorKind {
    Auth(AuthErrorKind),
    Database(DatabaseErrorKind),
    Cache(CacheErrorKind),
    Storage(StorageErrorKind),
    Network(NetworkErrorKind),
    Resilience(ResilienceErrorKind),
    Scheduler(SchedulerErrorKind),
    Queue(QueueErrorKind),
    Validation(ValidationErrorKind),
    Common(CommonErrorKind),
}
```

### 错误创建规范

```rust
// 正确：使用模块特定的错误类型，关联数据直接嵌入变体
WaeError::new(DatabaseErrorKind::RecordNotFound {
    table: "users".to_string(),
    id: user_id,
})

// 正确：使用便捷方法
WaeError::required_field("email")
WaeError::invalid_format("email", "valid email address")

// 错误：使用字符串错误
return Err("something went wrong".into());
```

### i18n 支持

每个错误类型对应唯一的 i18n key：

```rust
impl DatabaseErrorKind {
    pub fn i18n_key(&self) -> &'static str {
        match self {
            DatabaseErrorKind::RecordNotFound { .. } => "error.database.record_not_found",
            DatabaseErrorKind::UniqueViolation { .. } => "error.database.unique_violation",
            // ...
        }
    }
}
```

## 模块内部结构规范

### 目录结构

```
wae-{module}/
├── src/
│   ├── lib.rs          # 模块入口，公开 API
│   ├── error.rs        # 模块特定错误（如有）
│   ├── {domain}/       # 领域子模块
│   │   ├── mod.rs
│   │   └── *.rs
│   └── tests/          # 内部测试（可选）
├── tests/
│   └── *.rs            # 集成测试
├── Cargo.toml
└── readme.md
```

### lib.rs 规范

```rust
//! WAE {Module} - {一句话描述}
//!
//! {详细描述}
//!
//! 深度融合 tokio 运行时，所有 API 都是异步优先设计。
#![warn(missing_docs)]

mod internal_module;

pub use wae_types::{WaeError, WaeResult};

pub type ModuleResult<T> = WaeResult<T>;

// 公开 API
pub use internal_module::{PublicStruct, public_function};
```

### Cargo.toml 规范

```toml
[package]
name = "wae-{module}"
version.workspace = true
edition.workspace = true

[dependencies]
wae-types = { workspace = true }
tokio = { workspace = true }
serde = { workspace = true }
# ... 其他依赖

[dev-dependencies]
# 仅测试依赖
```

## 异步设计规范

### 异步优先

所有 I/O 操作必须使用 `async`：

```rust
// 正确
pub async fn fetch_user(id: u64) -> DatabaseResult<User> { ... }

// 错误
pub fn fetch_user_blocking(id: u64) -> User { ... }
```

### Trait 异步方法

使用 `async-trait`：

```rust
#[async_trait::async_trait]
pub trait Repository: Send + Sync {
    async fn find_by_id(&self, id: u64) -> DatabaseResult<Option<User>>;
}
```

### 取消支持

长时间运行的操作应支持取消：

```rust
pub async fn process_stream(
    mut stream: impl Stream<Item = Item>,
    cancel: CancellationToken,
) -> Result<()> {
    while let Some(item) = tokio::select! {
        item = stream.next() => item,
        _ = cancel.cancelled() => return Ok(()),
    } {
        // 处理 item
    }
    Ok(())
}
```

## 测试规范

### 单元测试

放在源文件中：

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_function_name() {
        // 测试代码
    }
}
```

### 集成测试

放在 `tests/` 目录：

```
tests/
├── main.rs           # 基础集成测试
├── module_tests.rs   # 模块特定测试
```

### 测试命名

```rust
// 格式：test_{被测函数}_{场景}_{预期结果}
#[test]
fn test_sign_url_cos_provider_returns_valid_url() { ... }

#[test]
fn test_sign_url_invalid_config_returns_error() { ... }
```

## 文档规范

### 公开 API 必须有文档

```rust
/// 存储服务配置
///
/// 包含连接对象存储所需的所有配置项。
///
/// # Example
///
/// ```
/// use wae_storage::StorageConfig;
///
/// let config = StorageConfig {
///     provider: StorageProviderType::Cos,
///     secret_id: "id".to_string(),
///     secret_key: "key".to_string(),
///     bucket: "my-bucket".to_string(),
///     region: "ap-guangzhou".to_string(),
///     endpoint: None,
///     cdn_url: None,
/// };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageConfig {
    /// 存储提供商类型
    pub provider: StorageProviderType,
    // ...
}
```

### 禁止后置注释

```rust
// 正确
/// 用户 ID
pub user_id: u64,

// 错误
pub user_id: u64, // 用户 ID
```

## 扩展指南

### 添加新 Capability

1. 在 `wae-ai/src/capabilities/` 创建新文件
2. 定义 trait 和相关类型
3. 在 `lib.rs` 中导出
4. 在 `providers/` 中实现各服务商适配

### 添加新 Provider

1. 在 `wae-{module}/src/providers/` 创建新文件
2. 实现对应 trait
3. 在 `mod.rs` 中导出
4. 更新 `StorageProviderType` 或类似枚举

### 添加新模块

1. 在 `backends/` 创建 `wae-{module}/`
2. 添加到 `Cargo.toml` workspace members
3. 遵循本文档的架构规范
4. 添加到 `wae` crate 的依赖（如需要）
