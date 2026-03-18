# wae-types

类型定义模块，提供前后端统一的类型定义，确保类型安全通信。本文档将前后端类型定义串联讲解，帮助开发者理解数据结构的一致性。

## 概述

类型定义模块是 WAE 框架的基础层，所有类型都设计为：

- **异步友好**: 支持 tokio 运行时
- **序列化就绪**: 支持 JSON/MessagePack 等格式
- **零拷贝**: 最小化数据复制
- **前后端一致**: TypeScript 和 Rust 类型定义保持同步

### 架构设计

```
┌─────────────────────────────────────────────────────────────────┐
│                         前端 (Frontend)                          │
│  ┌─────────────────────────────────────────────────────────┐    │
│  │                    wae-types (TypeScript)                │    │
│  │  • CloudError / ApiResponse                              │    │
│  │  • BillingDimensions / BillingCostConfig                 │    │
│  │  • UserInfo / AuthToken / Credentials                    │    │
│  │  • StorageConfig / FileInfo                              │    │
│  │  • WebSocketMessage / Connection                         │    │
│  └─────────────────────────────────────────────────────────┘    │
└─────────────────────────────────────────────────────────────────┘
                              │ JSON 序列化
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│                         后端 (Backend)                           │
│  ┌─────────────────────────────────────────────────────────┐    │
│  │                    wae-types (Rust)                      │    │
│  │  • CloudError / CloudResult                              │    │
│  │  • BillingDimensions / BillingCostConfig                 │    │
│  │  • Value (动态类型)                                       │    │
│  │  • Decimal (精确计算)                                     │    │
│  └─────────────────────────────────────────────────────────┘    │
└─────────────────────────────────────────────────────────────────┘
```

---

## 错误处理类型

### CloudError 云服务错误

前后端使用统一的错误类型，确保错误处理一致性。

```typescript
// 前端 TypeScript
type CloudErrorType =
    | "Authentication"
    | "InvalidParams"
    | "NotFound"
    | "PermissionDenied"
    | "RateLimit"
    | "Internal"
    | "Network"
    | "Unknown";

interface CloudError {
    type: CloudErrorType;
    message: string;
}
```

```rust
// 后端 Rust
#[derive(Debug, Serialize, Deserialize)]
pub enum CloudError {
    Authentication(String),
    InvalidParams(String),
    NotFound(String),
    PermissionDenied(String),
    RateLimit(String),
    Internal(String),
    Network(String),
    Unknown(String),
}
```

### 错误类型与 HTTP 状态码映射

| 错误类型 | HTTP 状态码 | 说明 |
|---------|------------|------|
| `Authentication` | 401 | 认证失败 |
| `InvalidParams` | 400 | 参数错误 |
| `NotFound` | 404 | 资源不存在 |
| `PermissionDenied` | 403 | 权限不足 |
| `RateLimit` | 429 | 请求频率超限 |
| `Internal` | 500 | 内部错误 |
| `Network` | 502 | 网络错误 |
| `Unknown` | 500 | 未知错误 |

### CloudResult 结果类型

```typescript
// 前端 TypeScript
type CloudResult<T> = Result<T, CloudError>;
```

```rust
// 后端 Rust
pub type CloudResult<T> = Result<T, CloudError>;
```

### 错误创建函数 (前端)

```typescript
import {
    authenticationError,
    invalidParamsError,
    notFoundError,
    permissionDeniedError,
    rateLimitError,
    internalError,
    networkError,
    unknownError
} from 'wae-types';

const error = authenticationError("Token 已过期");
const notFound = notFoundError("用户不存在");
```

### 使用示例

```rust
// 后端 Rust
use wae_types::CloudError;

fn validate_user(id: u64) -> Result<User, CloudError> {
    if id == 0 {
        return Err(CloudError::InvalidParams("用户 ID 不能为 0".to_string()));
    }
    
    match find_user(id) {
        Some(user) => Ok(user),
        None => Err(CloudError::NotFound(format!("用户 {} 不存在", id))),
    }
}
```

```typescript
// 前端 TypeScript
import { CloudError } from 'wae-types';

function handleError(error: CloudError) {
    switch (error.type) {
        case "Authentication":
            console.error("认证失败:", error.message);
            break;
        case "RateLimit":
            console.error("请求过于频繁:", error.message);
            break;
        default:
            console.error("错误:", error.message);
    }
}
```

---

## API 响应类型

### ApiResponse 统一响应结构

```typescript
// 前端 TypeScript
interface ApiResponse<T> {
    success: boolean;
    data: T | null;
    error: ApiErrorBody | null;
    traceId: string | null;
}

interface ApiErrorBody {
    code: string;
    message: string;
}
```

```rust
// 后端 Rust
#[derive(Debug, Serialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub error: Option<ApiErrorBody>,
    pub trace_id: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ApiErrorBody {
    pub code: String,
    pub message: String,
}
```

### 响应示例

成功响应：

```json
{
    "success": true,
    "data": { "id": 1, "name": "Alice" },
    "error": null,
    "traceId": null
}
```

错误响应：

```json
{
    "success": false,
    "data": null,
    "error": {
        "code": "INVALID_PARAMS",
        "message": "参数错误"
    },
    "traceId": "trace-123"
}
```

### 响应创建函数 (前端)

```typescript
import {
    successResponse,
    successResponseWithTrace,
    errorResponse,
    errorResponseWithTrace,
    isSuccess,
    unwrapResponse
} from 'wae-types';

const response = successResponse({ id: 1, name: "张三" });
const error = errorResponse("VALIDATION_ERROR", "参数验证失败");

if (isSuccess(response)) {
    console.log(response.data.name);
}

const data = unwrapResponse(response);
```

### 响应处理示例

```typescript
// 前端 TypeScript
import { ApiResponse, isSuccess, unwrapResponse } from 'wae-types';

async function fetchUser(id: string): Promise<UserInfo> {
    const response: ApiResponse<UserInfo> = await fetch(`/api/users/${id}`).then(r => r.json());

    if (!isSuccess(response)) {
        throw new Error(response.error?.message ?? 'Unknown error');
    }

    return response.data!;
}
```

---

## 计费类型

### BillingDimensions 计费维度

四元矩阵：输入文本、输出文本、输入像素、输出像素。

```typescript
// 前端 TypeScript
interface BillingDimensions {
    inputText: number;
    outputText: number;
    inputPixels: number;
    outputPixels: number;
}
```

```rust
// 后端 Rust
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BillingDimensions {
    pub input_text: u64,
    pub output_text: u64,
    pub input_pixels: u64,
    pub output_pixels: u64,
}
```

| 字段 | 类型 | 说明 |
|-----|------|------|
| `inputText` / `input_text` | `number` / `u64` | 输入文本长度 (字符数或 Token 数) |
| `outputText` / `output_text` | `number` / `u64` | 输出文本长度 |
| `inputPixels` / `input_pixels` | `number` / `u64` | 输入像素数 (宽 * 高) |
| `outputPixels` / `output_pixels` | `number` / `u64` | 输出像素数 |

### TextCost 文本成本

每百万 Tokens/字符的成本。

```typescript
// 前端 TypeScript
interface TextCost {
    perMillion: number;
}
```

```rust
// 后端 Rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextCost {
    pub per_million: Decimal,
}
```

### ImageCost 图像成本

每百万像素的成本。

```typescript
// 前端 TypeScript
interface ImageCost {
    perMillion: number;
}
```

```rust
// 后端 Rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageCost {
    pub per_million: Decimal,
}
```

### BillingCostConfig 成本配置

```typescript
// 前端 TypeScript
interface BillingCostConfig {
    inputText: TextCost;
    outputText: TextCost;
    inputPixels: ImageCost;
    outputPixels: ImageCost;
}
```

```rust
// 后端 Rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BillingCostConfig {
    pub input_text: TextCost,
    pub output_text: TextCost,
    pub input_pixels: ImageCost,
    pub output_pixels: ImageCost,
}
```

### 计算总成本

计算公式: `(消耗数量 / 1,000,000) * 每百万单价`

```typescript
// 前端 TypeScript
import { calculateTotalCost, BillingDimensions, BillingCostConfig } from 'wae-types';

const usage: BillingDimensions = {
    inputText: 1000,
    outputText: 500,
    inputPixels: 512 * 512,
    outputPixels: 1024 * 1024
};

const config: BillingCostConfig = {
    inputText: { perMillion: 0.5 },
    outputText: { perMillion: 1.5 },
    inputPixels: { perMillion: 0.01 },
    outputPixels: { perMillion: 0.02 }
};

const totalCost = calculateTotalCost(usage, config);
```

```rust
// 后端 Rust
use wae_types::{BillingDimensions, BillingCostConfig, TextCost, ImageCost};
use rust_decimal_macros::dec;

let usage = BillingDimensions {
    input_text: 1000,
    output_text: 500,
    input_pixels: 0,
    output_pixels: 1024 * 1024,
};

let cost_config = BillingCostConfig {
    input_text: TextCost { per_million: dec!(0.5) },
    output_text: TextCost { per_million: dec!(1.5) },
    input_pixels: ImageCost { per_million: dec!(0.01) },
    output_pixels: ImageCost { per_million: dec!(0.02) },
};

let total_cost = cost_config.calculate_total_cost(&usage);
println!("总成本: {}", total_cost);
```

---

## 认证类型

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

### AuthConfig 认证配置

```typescript
// 前端 TypeScript
interface AuthConfig {
    tokenExpiresIn: number;
    refreshTokenExpiresIn: number;
    issuer: string;
    audience: string;
    passwordMinLength: number;
    passwordRequireDigit: boolean;
    passwordRequireSpecial: boolean;
    maxLoginAttempts: number;
    lockoutDuration: number;
}
```

```rust
// 后端 Rust
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

### ApiKeyInfo API Key 信息

```typescript
// 前端 TypeScript
interface ApiKeyInfo {
    id: string;
    name: string;
    prefix: string;
    createdAt: number;
    expiresAt: number | null;
    lastUsedAt: number | null;
}
```

```rust
// 后端 Rust
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

## 存储类型

### StorageProviderType 存储提供商

```typescript
// 前端 TypeScript
type StorageProviderType = "Cos" | "Oss" | "Local";
```

```rust
// 后端 Rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StorageProviderType {
    Cos,
    Oss,
    Local,
}
```

| 值 | 说明 |
| --- | --- |
| `Cos` | 腾讯云对象存储 |
| `Oss` | 阿里云对象存储 |
| `Local` | 本地存储 |

### StorageConfig 存储配置

```typescript
// 前端 TypeScript
interface StorageConfig {
    provider: StorageProviderType;
    secretId: string;
    secretKey: string;
    bucket: string;
    region: string;
    endpoint?: string;
    cdnUrl?: string;
}
```

```rust
// 后端 Rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageConfig {
    pub provider: StorageProviderType,
    pub secret_id: String,
    pub secret_key: String,
    pub bucket: String,
    pub region: String,
    pub endpoint: Option<String>,
    pub cdn_url: Option<String>,
}
```

### UploadOptions 上传选项

```typescript
// 前端 TypeScript
interface UploadOptions {
    key: string;
    contentType?: string;
    metadata?: Record<string, string>;
    expiresIn?: number;
}
```

### PresignedUploadResult 预签名上传结果

```typescript
// 前端 TypeScript
interface PresignedUploadResult {
    uploadUrl: string;
    accessUrl: string;
    expiresAt: number;
}
```

### FileInfo 文件信息

```typescript
// 前端 TypeScript
interface FileInfo {
    key: string;
    size: number;
    contentType: string;
    etag: string;
    lastModified: number;
}
```

---

## WebSocket 类型

### WebSocketMessage 消息类型

```typescript
// 前端 TypeScript
type WebSocketMessageType = "Text" | "Binary" | "Ping" | "Pong" | "Close";

interface WebSocketMessage {
    type: WebSocketMessageType;
    text?: string;
    binary?: Uint8Array;
}
```

```rust
// 后端 Rust
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Message {
    Text(String),
    Binary(Vec<u8>),
    Ping,
    Pong,
    Close,
}
```

### 消息创建函数 (前端)

```typescript
import {
    textMessage,
    binaryMessage,
    closeMessage,
    isTextMessage,
    isBinaryMessage
} from 'wae-types';

const msg = textMessage("Hello, World!");
const binMsg = binaryMessage(new Uint8Array([1, 2, 3]));
const close = closeMessage();

if (isTextMessage(msg)) {
    console.log(msg.text);
}
```

### Connection 连接信息

```typescript
// 前端 TypeScript
interface Connection {
    id: ConnectionId;
    addr: string;
    connectedAt: number;
    metadata: Record<string, string>;
    rooms: RoomId[];
}
```

```rust
// 后端 Rust
#[derive(Debug, Clone)]
pub struct Connection {
    pub id: ConnectionId,
    pub addr: SocketAddr,
    pub connected_at: std::time::Instant,
    pub metadata: HashMap<String, String>,
    pub rooms: Vec<RoomId>,
}
```

### ConnectionState 连接状态

```typescript
// 前端 TypeScript
type ConnectionState = "connecting" | "connected" | "disconnecting" | "disconnected";
```

### WebSocket 事件类型

```typescript
type WebSocketEventType =
    | "connect"
    | "disconnect"
    | "message"
    | "error"
    | "reconnect"
    | "reconnect_failed";

interface ConnectEventData {
    connection: Connection;
}

interface DisconnectEventData {
    connection: Connection;
    reason: string;
}

interface MessageEventData {
    connection: Connection;
    message: WebSocketMessage;
}

interface ErrorEventData {
    error: Error;
}

interface ReconnectEventData {
    attempt: number;
    maxAttempts: number;
}

interface ReconnectFailedEventData {
    totalAttempts: number;
}
```

---

## Value 动态类型 (后端)

后端 Rust 提供动态值类型，类似 JSON 的动态类型支持：

```rust
#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Null,
    Bool(bool),
    Integer(i64),
    Float(f64),
    String(String),
    Bytes(Vec<u8>),
    Array(Vec<Value>),
    Object(HashMap<String, Value>),
}
```

### 创建值

```rust
use wae_types::Value;

let null = Value::null();
let bool_val = Value::bool(true);
let int_val = Value::integer(42);
let float_val = Value::float(3.14);
let str_val = Value::string("hello");
let bytes_val = Value::bytes(vec![1, 2, 3]);
let arr_val = Value::array(vec![Value::integer(1), Value::integer(2)]);
let obj_val = Value::object(HashMap::from([
    ("name".to_string(), Value::string("Alice")),
]));
```

### 类型检查

```rust
let val = Value::integer(42);

val.is_null();      // false
val.is_bool();      // false
val.is_integer();   // true
val.is_float();     // false
val.is_number();    // true
val.is_string();    // false
val.is_array();     // false
val.is_object();    // false
```

### 值获取

```rust
let val = Value::integer(42);

val.as_bool();      // None
val.as_integer();   // Some(42)
val.as_float();     // Some(42.0)
val.as_str();       // None
val.as_array();     // None
val.as_object();    // None
```

### JSON 转换

```rust
let val = Value::object(HashMap::from([
    ("name".to_string(), Value::string("Alice")),
]));

let json = val.to_json_string();
let parsed = Value::from_json_str(r#"{"name":"Alice"}"#)?;
```

### From 实现

```rust
let v1: Value = true.into();
let v2: Value = 42i64.into();
let v3: Value = 3.14f64.into();
let v4: Value = "hello".into();
let v5: Value = vec![Value::integer(1)].into();
```

---

## 便捷宏 (后端)

### object! 宏

构建对象：

```rust
use wae_types::{Value, object};

let obj = object! {
    "name" => "Alice",
    "age" => 30,
    "active" => true,
};
```

### array! 宏

构建数组：

```rust
use wae_types::{Value, array};

let arr = array![1, 2, 3, "hello", true];
```

---

## 工具函数

### format_slug

格式化显示名称或 slug：

```rust
pub fn format_slug(name: &str) -> String {
    name.to_lowercase().replace(' ', "-")
}
```

```rust
use wae_types::format_slug;

let slug = format_slug("Hello World");
assert_eq!(slug, "hello-world");
```

### truncate_str

截断字符串用于日志输出：

```rust
pub fn truncate_str(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len])
    }
}
```

```rust
use wae_types::truncate_str;

let truncated = truncate_str("这是一段很长的文本内容", 5);
assert_eq!(truncated, "这是一段很长的...");
```

---

## Decimal 类型 (后端)

后端重新导出 `rust_decimal` 的 `Decimal` 类型和 `dec!` 宏，用于精确的货币计算：

```rust
pub use rust_decimal::Decimal;
pub use rust_decimal_macros::dec;
```

```rust
use wae_types::{Decimal, dec};

let price: Decimal = dec!(19.99);
let quantity: Decimal = dec!(3);
let total = price * quantity;

assert_eq!(total, dec!(59.97));
```
