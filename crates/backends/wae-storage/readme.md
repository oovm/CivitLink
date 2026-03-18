# wae-storage

存储模块 - 提供对象存储抽象层。

## 主要功能

- **多后端支持**: 支持 S3、MinIO、本地存储
- **文件操作**: 上传、下载、删除文件
- **预签名 URL**: 生成临时访问链接
- **分片上传**: 大文件分片上传

## 技术栈

- **对象存储**: aws-sdk-s3 (可选)
- **异步运行时**: Tokio

## 使用示例

```rust
use wae_storage::{StorageClient, StorageConfig, StorageBackend};

#[tokio::main]
async fn main() {
    let storage = StorageClient::new(StorageConfig {
        backend: StorageBackend::S3 {
            endpoint: "https://s3.amazonaws.com".to_string(),
            bucket: "my-bucket".to_string(),
            access_key: "access-key".to_string(),
            secret_key: "secret-key".to_string(),
        },
    });
    
    storage.upload("images/avatar.jpg", &file_data).await?;
    
    let data = storage.download("images/avatar.jpg").await?;
    
    let url = storage.presigned_url("images/avatar.jpg", Duration::from_secs(3600)).await?;
}
```

## 本地存储

```rust
let storage = StorageClient::new(StorageConfig {
    backend: StorageBackend::Local {
        root: "/data/storage".to_string(),
    },
});
```

## 支持的后端

| 后端 | 说明 |
|------|------|
| S3 | Amazon S3 |
| MinIO | MinIO 对象存储 |
| Local | 本地文件系统 |
