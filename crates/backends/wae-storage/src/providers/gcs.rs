//! Google Cloud Storage 存储提供商实现

use crate::{StorageConfig, StorageProvider, StorageResult};

use chrono::Utc;
use hmac::{Hmac, Mac};
use sha2::{Digest, Sha256};
use url::Url;
use wae_types::WaeError;

/// Google Cloud Storage 存储提供商
///
/// 实现 Google Cloud Storage 对象存储服务的签名和预签名 URL 生成
pub struct GcsProvider;

impl GcsProvider {
    /// 将哈希值转换为十六进制字符串
    fn hex_encode(data: &[u8]) -> String {
        data.iter().map(|b| format!("{:02x}", b)).collect()
    }

    /// 计算 SHA256 哈希值
    fn sha256_hash(data: &[u8]) -> Vec<u8> {
        let mut hasher = Sha256::new();
        hasher.update(data);
        hasher.finalize().to_vec()
    }

    /// 使用 HMAC-SHA256 算法计算消息认证码
    fn hmac_sha256(key: &[u8], data: &[u8]) -> Vec<u8> {
        let mut mac = Hmac::<Sha256>::new_from_slice(key).expect("HMAC can take key of any size");
        mac.update(data);
        mac.finalize().into_bytes().to_vec()
    }

    /// 获取签名密钥
    fn get_signature_key(key: &str, date_stamp: &str, region_name: &str, service_name: &str) -> Vec<u8> {
        let k_date = Self::hmac_sha256(format!("GOOG4{}", key).as_bytes(), date_stamp.as_bytes());
        let k_region = Self::hmac_sha256(&k_date, region_name.as_bytes());
        let k_service = Self::hmac_sha256(&k_region, service_name.as_bytes());
        Self::hmac_sha256(&k_service, b"goog4_request")
    }

    /// URI 编码
    fn uri_encode(s: &str, encode_slash: bool) -> String {
        let mut result = String::new();
        for c in s.chars() {
            match c {
                'A'..='Z' | 'a'..='z' | '0'..='9' | '_' | '-' | '~' | '.' => {
                    result.push(c);
                }
                '/' => {
                    if encode_slash {
                        result.push_str("%2F");
                    }
                    else {
                        result.push('/');
                    }
                }
                _ => {
                    let mut buf = [0; 4];
                    let bytes = c.encode_utf8(&mut buf).as_bytes();
                    for byte in bytes {
                        result.push_str(&format!("%{:02X}", byte));
                    }
                }
            }
        }
        result
    }
}

impl StorageProvider for GcsProvider {
    fn sign_url(&self, path: &str, config: &StorageConfig) -> StorageResult<Url> {
        if path.is_empty() {
            return Err(WaeError::invalid_params("path", "Empty path"));
        }

        if path.starts_with("http://") || path.starts_with("https://") {
            return Url::parse(path).map_err(|e| WaeError::invalid_params("path", e.to_string()));
        }

        let clean_path = path.trim_start_matches('/');
        let now = Utc::now();
        let amz_date = now.format("%Y%m%dT%H%M%SZ").to_string();
        let date_stamp = now.format("%Y%m%d").to_string();
        let expires = 3600;

        let host_str = config.cdn_url.clone().unwrap_or_else(|| {
            if let Some(endpoint) = &config.endpoint {
                endpoint.clone()
            }
            else {
                format!("https://{}.storage.googleapis.com", config.bucket)
            }
        });

        let host_url =
            if host_str.starts_with("http") { Url::parse(&host_str) } else { Url::parse(&format!("https://{}", host_str)) }
                .map_err(|e| WaeError::invalid_params("host", e.to_string()))?;

        let mut url = host_url.join(clean_path).map_err(|e| WaeError::invalid_params("path", e.to_string()))?;

        let host = url.host_str().unwrap_or("");

        let method = "GET";
        let canonical_uri = format!("/{}", clean_path);
        let canonical_querystring = format!(
            "X-Goog-Algorithm=GOOG4-RSA-SHA256&X-Goog-Credential={}%2F{}%2F{}%2Fstorage%2Fgoog4_request&X-Goog-Date={}&X-Goog-Expires={}&X-Goog-SignedHeaders=host",
            Self::uri_encode(&config.secret_id, true),
            date_stamp,
            config.region,
            amz_date,
            expires
        );
        let canonical_headers = format!("host:{}\n", host);
        let signed_headers = "host";
        let payload_hash = Self::hex_encode(&Self::sha256_hash(b""));

        let canonical_request = format!(
            "{}\n{}\n{}\n{}\n{}\n{}",
            method, canonical_uri, canonical_querystring, canonical_headers, signed_headers, payload_hash
        );

        let credential_scope = format!("{}/{}/storage/goog4_request", date_stamp, config.region);
        let string_to_sign = format!(
            "GOOG4-RSA-SHA256\n{}\n{}\n{}",
            amz_date,
            credential_scope,
            Self::hex_encode(&Self::sha256_hash(canonical_request.as_bytes()))
        );

        let signing_key = Self::get_signature_key(&config.secret_key, &date_stamp, &config.region, "storage");
        let signature = Self::hex_encode(&Self::hmac_sha256(&signing_key, string_to_sign.as_bytes()));

        url.set_query(Some(&format!("{}&X-Goog-Signature={}", canonical_querystring, signature)));

        Ok(url)
    }

    fn get_presigned_put_url(&self, key: &str, config: &StorageConfig) -> StorageResult<Url> {
        let clean_key = key.trim_start_matches('/');
        let now = Utc::now();
        let amz_date = now.format("%Y%m%dT%H%M%SZ").to_string();
        let date_stamp = now.format("%Y%m%d").to_string();
        let expires = 900;

        let host_str = config.endpoint.clone().unwrap_or_else(|| format!("https://{}.storage.googleapis.com", config.bucket));

        let host_url =
            if host_str.starts_with("http") { Url::parse(&host_str) } else { Url::parse(&format!("https://{}", host_str)) }
                .map_err(|e| WaeError::invalid_params("host", e.to_string()))?;

        let mut url = host_url.join(clean_key).map_err(|e| WaeError::invalid_params("key", e.to_string()))?;

        let host = url.host_str().unwrap_or("");

        let method = "PUT";
        let canonical_uri = format!("/{}", clean_key);
        let canonical_querystring = format!(
            "X-Goog-Algorithm=GOOG4-RSA-SHA256&X-Goog-Credential={}%2F{}%2F{}%2Fstorage%2Fgoog4_request&X-Goog-Date={}&X-Goog-Expires={}&X-Goog-SignedHeaders=host",
            Self::uri_encode(&config.secret_id, true),
            date_stamp,
            config.region,
            amz_date,
            expires
        );
        let canonical_headers = format!("host:{}\n", host);
        let signed_headers = "host";
        let payload_hash = Self::hex_encode(&Self::sha256_hash(b"UNSIGNED-PAYLOAD"));

        let canonical_request = format!(
            "{}\n{}\n{}\n{}\n{}\n{}",
            method, canonical_uri, canonical_querystring, canonical_headers, signed_headers, payload_hash
        );

        let credential_scope = format!("{}/{}/storage/goog4_request", date_stamp, config.region);
        let string_to_sign = format!(
            "GOOG4-RSA-SHA256\n{}\n{}\n{}",
            amz_date,
            credential_scope,
            Self::hex_encode(&Self::sha256_hash(canonical_request.as_bytes()))
        );

        let signing_key = Self::get_signature_key(&config.secret_key, &date_stamp, &config.region, "storage");
        let signature = Self::hex_encode(&Self::hmac_sha256(&signing_key, string_to_sign.as_bytes()));

        url.set_query(Some(&format!("{}&X-Goog-Signature={}", canonical_querystring, signature)));

        Ok(url)
    }
}
