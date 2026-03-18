//! WAE Storage - 存储服务抽象层
//!
//! 提供统一的存储服务抽象，支持 COS、OSS、S3 和本地存储。
//!
//! 深度融合 tokio 运行时，支持：
//! - 异步上传/下载
//! - 流式处理
//! - 零拷贝优化
#![warn(missing_docs)]
pub mod providers;

use serde::{Deserialize, Serialize};
pub use wae_types::{WaeError, WaeResult};

use url::Url;

/// 存储操作结果类型
pub type StorageResult<T> = WaeResult<T>;

/// 存储服务提供商类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StorageProviderType {
    /// 腾讯云 COS
    Cos,
    /// Google Cloud Storage
    Gcs,
    /// 阿里云 OSS
    Oss,
    /// AWS S3
    S3,
    /// 本地存储
    Local,
    /// Azure Blob Storage
    AzureBlob,
}

/// 存储服务配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageConfig {
    /// 存储服务提供商类型
    pub provider: StorageProviderType,
    /// 访问密钥 ID
    pub secret_id: String,
    /// 访问密钥
    pub secret_key: String,
    /// 存储桶名称
    pub bucket: String,
    /// 地域
    pub region: String,
    /// 自定义端点
    pub endpoint: Option<String>,
    /// CDN 加速域名
    pub cdn_url: Option<String>,
}

/// 存储服务提供商 trait
///
/// 定义存储服务的基本操作接口
pub trait StorageProvider: Send + Sync {
    /// 获取上传预签名 URL
    fn get_presigned_put_url(&self, key: &str, config: &StorageConfig) -> StorageResult<Url>;

    /// 获取带签名的访问 URL
    fn sign_url(&self, path: &str, config: &StorageConfig) -> StorageResult<Url>;
}

pub use providers::{
    azure_blob::AzureBlobProvider, cos::CosProvider, gcs::GcsProvider, local::LocalStorageProvider, oss::OssProvider,
    s3::S3Provider,
};

/// 存储服务
///
/// 提供统一的存储服务入口
pub struct StorageService;

impl StorageService {
    /// 根据提供商类型获取对应的存储提供商实例
    pub fn get_provider(provider_type: &StorageProviderType) -> Box<dyn StorageProvider> {
        match provider_type {
            StorageProviderType::Cos => Box::new(CosProvider),
            StorageProviderType::Gcs => Box::new(GcsProvider),
            StorageProviderType::Oss => Box::new(OssProvider),
            StorageProviderType::S3 => Box::new(S3Provider),
            StorageProviderType::Local => Box::new(LocalStorageProvider),
            StorageProviderType::AzureBlob => Box::new(AzureBlobProvider),
        }
    }

    /// 获取带签名的访问 URL
    pub fn sign_url(path: &str, config: &StorageConfig) -> StorageResult<Url> {
        let provider = Self::get_provider(&config.provider);
        provider.sign_url(path, config)
    }

    /// 获取上传预签名 URL
    pub fn get_presigned_put_url(key: &str, config: &StorageConfig) -> StorageResult<Url> {
        let provider = Self::get_provider(&config.provider);
        provider.get_presigned_put_url(key, config)
    }
}
