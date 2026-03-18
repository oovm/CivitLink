//! Azure Blob Storage 存储提供商实现

use crate::{StorageConfig, StorageProvider, StorageResult};

use base64::{Engine as _, engine::general_purpose::STANDARD};
use chrono::{Duration, Utc};
use hmac::{Hmac, Mac};
use sha2::Sha256;
use url::Url;
use wae_types::WaeError;

/// Azure Blob Storage 存储提供商
///
/// 实现 Azure Blob Storage 对象存储服务的签名和预签名 URL 生成
pub struct AzureBlobProvider;

impl AzureBlobProvider {
    /// 使用 HMAC-SHA256 算法计算消息认证码
    fn hmac_sha256(key: &[u8], data: &[u8]) -> Vec<u8> {
        let mut mac = Hmac::<Sha256>::new_from_slice(key).expect("HMAC can take key of any size");
        mac.update(data);
        mac.finalize().into_bytes().to_vec()
    }
}

impl StorageProvider for AzureBlobProvider {
    fn sign_url(&self, path: &str, config: &StorageConfig) -> StorageResult<Url> {
        if path.is_empty() {
            return Err(WaeError::invalid_params("path", "Empty path"));
        }

        if path.starts_with("http://") || path.starts_with("https://") {
            return Url::parse(path).map_err(|e| WaeError::invalid_params("path", e.to_string()));
        }

        let clean_path = path.trim_start_matches('/');
        let now = Utc::now();
        let _expiration = (now + Duration::seconds(3600)).timestamp();
        let x_ms_version = "2023-11-03";
        let _x_ms_date = now.format("%a, %d %b %Y %H:%M:%S GMT").to_string();

        let host_str = config.cdn_url.clone().unwrap_or_else(|| {
            if let Some(endpoint) = &config.endpoint {
                endpoint.clone()
            }
            else {
                format!("https://{}.blob.core.windows.net/{}", config.secret_id, config.bucket)
            }
        });

        let host_url =
            if host_str.starts_with("http") { Url::parse(&host_str) } else { Url::parse(&format!("https://{}", host_str)) }
                .map_err(|e| WaeError::invalid_params("host", e.to_string()))?;

        let mut url = host_url.join(clean_path).map_err(|e| WaeError::invalid_params("path", e.to_string()))?;

        let _method = "r";
        let start = "";
        let now = Utc::now();
        let expiry = (now + Duration::seconds(3600)).format("%Y-%m-%dT%H:%M:%SZ").to_string();
        let resource = "b";
        let permissions = "r";
        let version = x_ms_version;

        let account_key = STANDARD
            .decode(&config.secret_key)
            .map_err(|e| WaeError::invalid_params("secret_key", format!("Invalid base64: {}", e)))?;

        let resource_path = format!("/{}/{}/{}", config.secret_id, config.bucket, clean_path);
        let string_to_sign = format!(
            "{}\n{}\n{}\n{}\n{}\n{}\n{}\n{}\n{}\n{}\n{}\n{}\n{}\n{}\n",
            permissions, start, expiry, resource_path, "", "", "", "", "", "", "", version, "", ""
        );

        let signature = Self::hmac_sha256(&account_key, string_to_sign.as_bytes());
        let signature_base64 = STANDARD.encode(signature);

        url.query_pairs_mut()
            .append_pair("sv", version)
            .append_pair("sr", resource)
            .append_pair("sp", permissions)
            .append_pair("se", &expiry)
            .append_pair("sig", &signature_base64);

        Ok(url)
    }

    fn get_presigned_put_url(&self, key: &str, config: &StorageConfig) -> StorageResult<Url> {
        let clean_key = key.trim_start_matches('/');
        let now = Utc::now();
        let _expiration = (now + Duration::seconds(900)).timestamp();
        let x_ms_version = "2023-11-03";

        let host_str = config
            .endpoint
            .clone()
            .unwrap_or_else(|| format!("https://{}.blob.core.windows.net/{}", config.secret_id, config.bucket));

        let host_url =
            if host_str.starts_with("http") { Url::parse(&host_str) } else { Url::parse(&format!("https://{}", host_str)) }
                .map_err(|e| WaeError::invalid_params("host", e.to_string()))?;

        let mut url = host_url.join(clean_key).map_err(|e| WaeError::invalid_params("key", e.to_string()))?;

        let permissions = "w";
        let resource = "b";
        let version = x_ms_version;
        let expiry = (Utc::now() + Duration::seconds(900)).format("%Y-%m-%dT%H:%M:%SZ").to_string();
        let resource_path = format!("/{}/{}/{}", config.secret_id, config.bucket, clean_key);

        let account_key = STANDARD
            .decode(&config.secret_key)
            .map_err(|e| WaeError::invalid_params("secret_key", format!("Invalid base64: {}", e)))?;

        let string_to_sign = format!(
            "{}\n{}\n{}\n{}\n{}\n{}\n{}\n{}\n{}\n{}\n{}\n{}\n{}\n{}\n",
            permissions, "", expiry, resource_path, "", "", "", "", "", "", "", version, "", ""
        );

        let signature = Self::hmac_sha256(&account_key, string_to_sign.as_bytes());
        let signature_base64 = STANDARD.encode(signature);

        url.query_pairs_mut()
            .append_pair("sv", version)
            .append_pair("sr", resource)
            .append_pair("sp", permissions)
            .append_pair("se", &expiry)
            .append_pair("sig", &signature_base64);

        Ok(url)
    }
}
