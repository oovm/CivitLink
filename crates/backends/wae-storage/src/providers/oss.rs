//! 阿里云 OSS 存储提供商实现

use crate::{StorageConfig, StorageProvider, StorageResult};

use base64::{Engine as _, engine::general_purpose::STANDARD};
use chrono::Utc;
use hmac::{Hmac, Mac};
use sha1::Sha1;
use url::Url;
use wae_types::WaeError;

/// 阿里云 OSS 存储提供商
///
/// 实现阿里云对象存储服务的签名和预签名 URL 生成
pub struct OssProvider;

impl StorageProvider for OssProvider {
    fn sign_url(&self, path: &str, config: &StorageConfig) -> StorageResult<Url> {
        if path.is_empty() {
            return Err(WaeError::invalid_params("path", "Empty path"));
        }

        if path.starts_with("http://") || path.starts_with("https://") {
            return Url::parse(path).map_err(|e| WaeError::invalid_params("path", e.to_string()));
        }

        let clean_path = path.trim_start_matches('/');
        let expires = Utc::now().timestamp() + 3600;

        let host_str =
            config.cdn_url.clone().unwrap_or_else(|| format!("https://{}.{}.aliyuncs.com", config.bucket, config.region));

        let host_url =
            if host_str.starts_with("http") { Url::parse(&host_str) } else { Url::parse(&format!("https://{}", host_str)) }
                .map_err(|e| WaeError::invalid_params("host", e.to_string()))?;

        let mut url = host_url.join(clean_path).map_err(|e| WaeError::invalid_params("path", e.to_string()))?;

        let string_to_sign = format!("GET\n\n\n{}\n/{}/{}", expires, config.bucket, clean_path);

        let mut mac =
            Hmac::<Sha1>::new_from_slice(config.secret_key.as_bytes()).map_err(|e| WaeError::internal(e.to_string()))?;
        mac.update(string_to_sign.as_bytes());
        let signature = STANDARD.encode(mac.finalize().into_bytes());

        url.query_pairs_mut()
            .append_pair("OSSAccessKeyId", &config.secret_id)
            .append_pair("Expires", &expires.to_string())
            .append_pair("Signature", &signature);

        Ok(url)
    }

    fn get_presigned_put_url(&self, key: &str, config: &StorageConfig) -> StorageResult<Url> {
        let clean_key = key.trim_start_matches('/');
        let expires = Utc::now().timestamp() + 900;

        let host_str =
            config.endpoint.clone().unwrap_or_else(|| format!("https://{}.{}.aliyuncs.com", config.bucket, config.region));

        let host_url =
            if host_str.starts_with("http") { Url::parse(&host_str) } else { Url::parse(&format!("https://{}", host_str)) }
                .map_err(|e| WaeError::invalid_params("host", e.to_string()))?;

        let mut url = host_url.join(clean_key).map_err(|e| WaeError::invalid_params("key", e.to_string()))?;

        let string_to_sign = format!("PUT\n\n\n{}\n/{}/{}", expires, config.bucket, clean_key);

        let mut mac =
            Hmac::<Sha1>::new_from_slice(config.secret_key.as_bytes()).map_err(|e| WaeError::internal(e.to_string()))?;
        mac.update(string_to_sign.as_bytes());
        let signature = STANDARD.encode(mac.finalize().into_bytes());

        url.query_pairs_mut()
            .append_pair("OSSAccessKeyId", &config.secret_id)
            .append_pair("Expires", &expires.to_string())
            .append_pair("Signature", &signature);

        Ok(url)
    }
}
