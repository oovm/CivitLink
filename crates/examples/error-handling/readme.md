# WAE 错误处理示例

本示例展示了如何在 WAE 框架中使用统一的错误处理机制。

## 功能特性

- **统一的错误类型定义**：使用 `WaeError` 和 `WaeErrorKind` 提供标准化的错误类型
- **错误分类**：通过 `ErrorCategory` 将错误分类，便于映射到 HTTP 状态码
- **HTTP 错误转换**：使用 `HttpError` 将 `WaeError` 转换为 HTTP 响应
- **错误响应格式化**：`ErrorResponse` 提供标准化的 JSON 错误响应格式
- **便捷错误转换**：`ErrorExt` trait 提供便捷的错误转换方法

## 快速开始

```bash
cd e:\灵之镜有限公司\wae\examples\error-handling
cargo run
```

## 核心概念

### WaeError

`WaeError` 是 WAE 框架的核心错误类型，包含：
- `kind`：具体的错误类型（`WaeErrorKind`）
- 错误分类方法（`category()`）
- 国际化支持（`i18n_key()` 和 `i18n_data()`）

### HttpError

`HttpError` 包装 `WaeError` 以提供 HTTP 响应转换功能：
- 自动根据错误分类设置 HTTP 状态码
- 转换为 `Response<Body>`

### ErrorResponse

标准化的错误响应格式：
```json
{
  "success": false,
  "code": "wae.error.validation.required",
  "message": "Validation: wae.error.validation.required",
  "details": { "field": "name" }
}
```

## 示例代码

### 创建验证错误

```rust
use wae_types::WaeError;

if user.name.is_empty() {
    return Err(WaeError::required("name"));
}
```

### 创建资源未找到错误

```rust
use wae_types::WaeError;

Err(WaeError::not_found("user", id.to_string()))
```

### 转换为 HTTP 错误

```rust
use wae_https::error::{HttpError, HttpResult};

async fn get_user(id: u64) -> HttpResult<User> {
    let user = find_user(id)?;
    Ok(user)
}
```

### 使用 ErrorExt 便捷方法

```rust
use wae_https::error::ErrorExt;

let result: WaeResult<()> = some_operation();
let http_result = result.bad_request();
```

## 错误分类

| 分类 | HTTP 状态码 | 说明 |
|------|------------|------|
| Validation | 400 | 验证错误 |
| Auth | 401 | 认证错误 |
| Permission | 403 | 权限错误 |
| NotFound | 404 | 资源未找到 |
| Conflict | 409 | 资源冲突 |
| RateLimited | 429 | 限流 |
| Internal | 500 | 内部错误 |

## 更多信息

- 查看 `wae-types` 文档了解完整的错误类型定义
- 查看 `wae-https` 文档了解 HTTP 错误处理
