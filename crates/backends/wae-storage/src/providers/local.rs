//! 本地存储提供商实现

use crate::{StorageConfig, StorageProvider, StorageResult};

use url::Url;
use wae_types::WaeError;

/// 本地存储提供商
///
/// 用于本地开发环境的简单存储实现
pub struct LocalStorageProvider;

impl StorageProvider for LocalStorageProvider {
    fn sign_url(&self, path: &str, config: &StorageConfig) -> StorageResult<Url> {
        if path.is_empty() {
            return Err(WaeError::invalid_params("path", "Empty path"));
        }

        if path.starts_with("http://") || path.starts_with("https://") {
            return Url::parse(path).map_err(|e| WaeError::invalid_params("path", e.to_string()));
        }

        let base_url_str = config.cdn_url.as_deref().unwrap_or("http://localhost:3000/uploads");

        let base_url = Url::parse(base_url_str).map_err(|e| WaeError::invalid_params("base_url", e.to_string()))?;
        let url = base_url.join(path.trim_start_matches('/')).map_err(|e| WaeError::invalid_params("url", e.to_string()))?;

        Ok(url)
    }

    fn get_presigned_put_url(&self, key: &str, config: &StorageConfig) -> StorageResult<Url> {
        let base_url_str = config.endpoint.as_deref().unwrap_or("http://localhost:3000/api/upload");

        let base_url = Url::parse(base_url_str).map_err(|e| WaeError::invalid_params("base_url", e.to_string()))?;
        let url = base_url.join(key.trim_start_matches('/')).map_err(|e| WaeError::invalid_params("url", e.to_string()))?;

        Ok(url)
    }
}
