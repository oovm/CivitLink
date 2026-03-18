# wae-storage

存储服务模块，提供统一的文件上传、下载、删除等操作。本文档将前后端存储功能串联讲解，帮助开发者理解完整的存储流程。

## 概述

存储模块采用前后端分离架构，后端提供预签名 URL 生成服务，前端使用预签名 URL 直接上传文件到对象存储。这种架构避免了文件经过后端服务器中转，提高了上传效率并降低了服务器负载。

支持三种存储提供商：腾讯云 COS、阿里云 OSS、本地文件系统存储。

---

## 快速开始

### 前端上传文件

```typescript
import { createStorageClient } from "@wae/storage";

const storage = createStorageClient({
    baseUrl: "https://api.example.com",
});

const file = document.querySelector('input[type="file"]').files[0];
const key = `uploads/${Date.now()}-${file.name}`;

const result = await storage.uploadFile(file, key, (progress) => {
    console.log(`上传进度: ${progress.percentage}%`);
});

console.log('文件访问地址:', result.accessUrl);
```

### 后端生成预签名 URL

```rust
use wae::storage::{CosProvider, StorageProvider, StorageConfig, StorageProviderType};

let config = StorageConfig {
    provider: StorageProviderType::Cos,
    secret_id: std::env::var("COS_SECRET_ID")?,
    secret_key: std::env::var("COS_SECRET_KEY")?,
    bucket: "my-bucket".to_string(),
    region: "ap-guangzhou".to_string(),
    endpoint: None,
    cdn_url: None,
};

let provider = CosProvider;
let upload_url = provider.get_presigned_put_url("uploads/file.jpg", &config)?;
let access_url = provider.sign_url("uploads/file.jpg", &config)?;
```

---

## 上传流程

### 流程图

```
前端                                后端                    对象存储
 │                                   │                           │
 │  1. 请求预签名上传 URL             │                           │
 │  ──────────────────────────────▶  │                           │
 │  POST /storage/presign-upload     │                           │
 │  { key, contentType }             │                           │
 │                                   │  2. 生成预签名 URL         │
 │                                   │  ───────────────────────▶ │
 │                                   │  ◀─────────────────────── │
 │  ◀──────────────────────────────  │  返回预签名 URL            │
 │  { uploadUrl, accessUrl }         │                           │
 │                                   │                           │
 │  3. 直接上传文件到对象存储          │                           │
 │  ─────────────────────────────────────────────────────────▶   │
 │  PUT {uploadUrl}                  │                           │
 │  Content-Type: {contentType}      │                           │
 │  Body: file binary                │                           │
 │  ◀────────────────────────────────────────────────────────   │
 │  200 OK                           │                           │
 │                                   │                           │
 │  4. 使用 accessUrl 访问文件        │                           │
```

### 后端 API 实现

```rust
use wae::storage::{CosProvider, StorageProvider, StorageConfig, StorageProviderType};
use axum::{Router, routing::post, Json};

async fn presign_upload(
    Json(options): Json<UploadOptions>,
) -> Result<Json<PresignedUploadResult>, StorageError> {
    let config = StorageConfig {
        provider: StorageProviderType::Cos,
        secret_id: std::env::var("COS_SECRET_ID")?,
        secret_key: std::env::var("COS_SECRET_KEY")?,
        bucket: "my-bucket".to_string(),
        region: "ap-guangzhou".to_string(),
        endpoint: None,
        cdn_url: Some("https://cdn.example.com".to_string()),
    };
    
    let provider = CosProvider;
    let upload_url = provider.get_presigned_put_url(&options.key, &config)?;
    let access_url = provider.sign_url(&options.key, &config)?;
    
    Ok(Json(PresignedUploadResult {
        upload_url: upload_url.to_string(),
        access_url: access_url.to_string(),
        expires_at: chrono::Utc::now().timestamp() + 3600,
    }))
}

let app = Router::new()
    .route("/storage/presign-upload", post(presign_upload));
```

### 前端完整上传示例

```typescript
import { createStorageClient } from "@wae/storage";

const storage = createStorageClient({
    baseUrl: "https://api.example.com",
    timeout: 30000,
});

async function uploadImage(file: File) {
    const key = `images/${Date.now()}-${file.name}`;
    
    const result = await storage.uploadFile(file, key, (progress) => {
        console.log(`上传进度: ${progress.loaded}/${progress.total} (${progress.percentage}%)`);
    });
    
    console.log("上传成功!");
    console.log("访问地址:", result.accessUrl);
    console.log("过期时间:", new Date(result.expiresAt));
    
    return result;
}

async function uploadJsonData() {
    const jsonData = JSON.stringify({ name: "test", value: 123 });
    
    const result = await storage.uploadData(
        jsonData,
        "data/config.json",
        "application/json"
    );
    
    console.log("JSON 数据已上传:", result.accessUrl);
}

async function uploadWithRetry(file: File, maxRetries = 3) {
    for (let i = 0; i < maxRetries; i++) {
        try {
            return await storage.uploadFile(file, `uploads/${file.name}`);
        } catch (error) {
            if (i === maxRetries - 1) throw error;
            console.log(`上传失败，第 ${i + 1} 次重试...`);
            await new Promise(resolve => setTimeout(resolve, 1000 * (i + 1)));
        }
    }
}
```

### 前端客户端方法

| 方法 | 说明 |
|------|------|
| `uploadFile(file, key, onProgress?)` | 上传文件，支持进度回调 |
| `uploadData(data, key, contentType?)` | 上传字符串或 ArrayBuffer |
| `getAccessUrl(key, expiresIn?)` | 获取带签名的访问 URL |
| `getFileInfo(key)` | 获取文件元信息 |
| `deleteFile(key)` | 删除文件 |

---

## 下载流程

### 下载文件

```typescript
async function downloadImage() {
    const blob = await storage.downloadFile("images/photo.png");
    
    const url = URL.createObjectURL(blob);
    const img = document.createElement("img");
    img.src = url;
    document.body.appendChild(img);
}

async function loadConfig() {
    const config = await storage.downloadAsJson<{ apiUrl: string }>("data/config.json");
    console.log("API URL:", config.apiUrl);
}

async function downloadAsText() {
    const text = await storage.downloadAsText('data/readme.txt');
    console.log(text);
}
```

### 获取访问 URL

```typescript
async function getTemporaryUrl() {
    const accessUrl = await storage.getAccessUrl('uploads/image.png', 3600);
    console.log('一小时内有效的访问地址:', accessUrl);
    
    return accessUrl;
}

async function getPublicUrl() {
    const info = await storage.getFileInfo("uploads/document.pdf");
    console.log("文件大小:", info.size);
    console.log("内容类型:", info.contentType);
    console.log("最后修改:", new Date(info.lastModified));
}
```

### 后端生成签名 URL

```rust
let url = provider.sign_url("uploads/image.png", &config)?;

let url_with_expiry = provider.sign_url_with_expiry("uploads/image.png", &config, 7200)?;
```

---

## 服务商配置

### 腾讯云 COS

```rust
use wae::storage::{CosProvider, StorageProvider, StorageConfig, StorageProviderType};

let config = StorageConfig {
    provider: StorageProviderType::Cos,
    secret_id: "your-secret-id".to_string(),
    secret_key: "your-secret-key".to_string(),
    bucket: "your-bucket-1250000000".to_string(),
    region: "ap-guangzhou".to_string(),
    endpoint: None,
    cdn_url: None,
};

let provider = CosProvider;
let url = provider.sign_url("path/to/file.jpg", &config)?;
```

**配置说明：**
- `bucket`: 存储桶名称，格式为 `bucketname-appid`
- `region`: 地域，如 `ap-guangzhou`、`ap-shanghai`
- `secret_id` / `secret_key`: 从腾讯云控制台获取

### 阿里云 OSS

```rust
use wae::storage::{OssProvider, StorageProvider, StorageConfig, StorageProviderType};

let config = StorageConfig {
    provider: StorageProviderType::Oss,
    secret_id: "your-access-key-id".to_string(),
    secret_key: "your-access-key-secret".to_string(),
    bucket: "your-bucket".to_string(),
    region: "oss-cn-hangzhou".to_string(),
    endpoint: Some("oss-cn-hangzhou.aliyuncs.com".to_string()),
    cdn_url: None,
};

let provider = OssProvider;
let url = provider.sign_url("path/to/file.jpg", &config)?;
```

**配置说明：**
- `bucket`: 存储桶名称
- `region`: 地域，如 `oss-cn-hangzhou`、`oss-cn-beijing`
- `endpoint`: 可选，自定义 endpoint
- `secret_id` / `secret_key`: AccessKey ID 和 AccessKey Secret

### 本地存储

```rust
use wae::storage::{LocalStorageProvider, StorageProvider, StorageConfig, StorageProviderType};

let config = StorageConfig {
    provider: StorageProviderType::Local,
    secret_id: String::new(),
    secret_key: String::new(),
    bucket: "./uploads".to_string(),
    region: String::new(),
    endpoint: Some("http://localhost:3000/uploads".to_string()),
    cdn_url: None,
};

let provider = LocalStorageProvider;
```

**配置说明：**
- `bucket`: 本地存储目录路径
- `endpoint`: 文件访问的基础 URL

---

## CDN 配置

配置 CDN URL 后，生成的访问 URL 将使用 CDN 域名而非存储桶域名，加速文件访问。

### 后端配置

```rust
let config = StorageConfig {
    provider: StorageProviderType::Cos,
    secret_id: std::env::var("COS_SECRET_ID")?,
    secret_key: std::env::var("COS_SECRET_KEY")?,
    bucket: "my-bucket-1250000000".to_string(),
    region: "ap-guangzhou".to_string(),
    endpoint: None,
    cdn_url: Some("https://cdn.example.com".to_string()),
};

let url = provider.sign_url("uploads/image.png", &config)?;
// 输出: https://cdn.example.com/uploads/image.png?sign=xxx
```

### 前端配置

```typescript
import { createStorageClient } from "@wae/storage";

const storage = createStorageClient({
    baseUrl: "https://api.example.com",
    presignUploadPath: "/api/storage/presign-upload",
    signUrlPath: "/api/storage/sign-url",
    fileInfoPath: "/api/storage/file-info",
    deletePath: "/api/storage/delete",
    timeout: 60000,
    maxRetries: 3,
});
```

---

## 安全最佳实践

### 后端安全

**密钥保护：** 永远不要将 Secret ID 和 Secret Key 硬编码在代码中

```rust
let config = StorageConfig {
    secret_id: std::env::var("COS_SECRET_ID")?,
    secret_key: std::env::var("COS_SECRET_KEY")?,
    // ...
};
```

**文件验证：** 验证上传文件的类型和大小

```rust
async fn presign_upload_safe(
    Json(options): Json<UploadOptions>,
    user: AuthenticatedUser,
) -> Result<Json<PresignedUploadResult>, StorageError> {
    if options.key.contains("..") || options.key.starts_with('/') {
        return Err(StorageError::InvalidParams("Invalid key".into()));
    }
    
    let allowed_content_types = ["image/jpeg", "image/png", "image/gif", "application/pdf"];
    if let Some(ref ct) = options.content_type {
        if !allowed_content_types.contains(&ct.as_str()) {
            return Err(StorageError::InvalidParams("Unsupported content type".into()));
        }
    }
    
    let upload_url = provider.get_presigned_put_url(&options.key, &config)?;
    Ok(Json(PresignedUploadResult { /* ... */ }))
}
```

**权限控制：** 确保用户只能访问自己的文件

```rust
async fn presign_upload_with_auth(
    Json(options): Json<UploadOptions>,
    user: AuthenticatedUser,
) -> Result<Json<PresignedUploadResult>, StorageError> {
    let safe_key = format!("users/{}/{}", user.id, options.key);
    let upload_url = provider.get_presigned_put_url(&safe_key, &config)?;
    Ok(Json(PresignedUploadResult { /* ... */ }))
}
```

### 前端安全

**文件类型验证：** 在上传前验证文件类型

```typescript
async function validateAndUpload(file: File) {
    const maxSize = 10 * 1024 * 1024; // 10MB
    const allowedTypes = ["image/jpeg", "image/png", "image/gif"];
    
    if (file.size > maxSize) {
        throw new Error("文件大小超过限制 (最大 10MB)");
    }
    
    if (!allowedTypes.includes(file.type)) {
        throw new Error("不支持的文件类型，仅支持 JPEG、PNG、GIF");
    }
    
    const safeName = file.name.replace(/[^a-zA-Z0-9.-]/g, '_');
    const key = `uploads/${Date.now()}-${safeName}`;
    
    return storage.uploadFile(file, key);
}
```

**错误处理：** 优雅处理上传失败

```typescript
import { CloudError } from 'wae-types';

async function safeUpload(file: File) {
    try {
        return await storage.uploadFile(file, "test.png");
    } catch (error) {
        const cloudError = error as CloudError;
        switch (cloudError.type) {
            case "Authentication":
                console.error("认证失败，请重新登录");
                break;
            case "Network":
                console.error("网络错误，请检查网络连接");
                break;
            default:
                console.error("上传失败:", cloudError.message);
        }
        throw error;
    }
}
```

---

## 常见问题

### Q: 上传大文件时超时怎么办？

增加前端客户端的超时时间：

```typescript
const storage = createStorageClient({
    baseUrl: "https://api.example.com",
    timeout: 300000, // 5 分钟
});
```

对于超大文件（>100MB），建议使用分片上传。

### Q: 如何实现断点续传？

当前版本不支持断点续传。对于大文件，建议：
1. 前端分片后逐片上传
2. 后端合并分片

### Q: 预签名 URL 的有效期是多久？

默认有效期为 1 小时。可以在生成时指定：

```rust
let url = provider.get_presigned_put_url_with_expiry(key, &config, 7200)?; // 2 小时
```

### Q: 如何限制用户只能上传特定类型的文件？

后端验证 `content_type`：

```rust
let allowed = ["image/jpeg", "image/png", "application/pdf"];
if let Some(ref ct) = options.content_type {
    if !allowed.contains(&ct.as_str()) {
        return Err(StorageError::InvalidParams("Unsupported type".into()));
    }
}
```

前端同样验证：

```typescript
const allowedTypes = ["image/jpeg", "image/png", "application/pdf"];
if (!allowedTypes.includes(file.type)) {
    throw new Error("不支持的文件类型");
}
```

### Q: 本地存储适合生产环境吗？

本地存储仅适用于开发测试或单机部署场景。生产环境建议使用 COS 或 OSS，它们提供：
- 高可用性和数据持久性
- CDN 加速
- 完善的权限管理
- 数据备份和恢复

### Q: 如何切换存储服务商？

只需修改 `StorageConfig` 的 `provider` 字段和相关配置：

```rust
let config = StorageConfig {
    provider: StorageProviderType::Oss, // 从 Cos 改为 Oss
    secret_id: "oss-access-key-id".to_string(),
    secret_key: "oss-access-key-secret".to_string(),
    bucket: "oss-bucket".to_string(),
    region: "oss-cn-hangzhou".to_string(),
    // ...
};
```

业务代码无需修改，`StorageProvider` trait 提供了统一的接口。
