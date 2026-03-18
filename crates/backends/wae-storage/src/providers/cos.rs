//! 腾讯云 COS 存储提供商实现

use crate::{StorageConfig, StorageProvider, StorageResult};

use chrono::Utc;
use hmac::Hmac;
use sha1::Sha1;
use std::collections::HashSet;
use url::Url;
use wae_types::{WaeError, hex_encode, url_encode};

/// 腾讯云 COS 存储提供商
///
/// 实现腾讯云对象存储服务的签名和预签名 URL 生成
pub struct CosProvider;

/// 使用 HMAC-SHA1 算法计算消息认证码
fn hmac_sha1(key: &[u8], msg: &str) -> Vec<u8> {
    use hmac::Mac;
    type HmacSha1 = Hmac<Sha1>;
    let mut mac = HmacSha1::new_from_slice(key).expect("HMAC can take key of any size");
    mac.update(msg.as_bytes());
    mac.finalize().into_bytes().to_vec()
}

/// 使用 HMAC-SHA1 算法计算消息认证码并返回十六进制字符串
fn hmac_sha1_hex(key: &[u8], msg: &str) -> String {
    hex_encode(&hmac_sha1(key, msg))
}

/// 对字符串进行 COS 规范的 URL 编码
fn cos_encode(s: &str) -> String {
    let encoded = url_encode(s);
    ensure_lowercase_hex(&encoded)
}

/// 确保百分号编码中的十六进制字符为小写
fn ensure_lowercase_hex(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    let mut chars = s.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '%' {
            result.push(c);
            if let Some(h1) = chars.next() {
                result.push(h1.to_ascii_lowercase());
            }
            if let Some(h2) = chars.next() {
                result.push(h2.to_ascii_lowercase());
            }
        }
        else {
            result.push(c);
        }
    }
    result
}

/// 计算字符串的 SHA1 哈希值并返回十六进制字符串
fn sha1_hex(msg: &str) -> String {
    use sha1::Digest;
    let mut hasher = Sha1::new();
    hasher.update(msg.as_bytes());
    hex_encode(&hasher.finalize())
}

impl StorageProvider for CosProvider {
    fn sign_url(&self, path: &str, config: &StorageConfig) -> StorageResult<Url> {
        if path.is_empty() {
            return Err(WaeError::invalid_params("path", "Empty path"));
        }

        if path.starts_with("http://") || path.starts_with("https://") {
            return Url::parse(path).map_err(|e| WaeError::invalid_params("path", e.to_string()));
        }

        let base_url_str =
            config.cdn_url.clone().unwrap_or_else(|| format!("https://{}.cos.{}.myqcloud.com", config.bucket, config.region));

        let base_url = if base_url_str.starts_with("http") {
            Url::parse(&base_url_str)
        }
        else {
            Url::parse(&format!("https://{}", base_url_str))
        }
        .map_err(|e| WaeError::invalid_params("base_url", e.to_string()))?;

        let mut url =
            base_url.join(path.trim_start_matches('/')).map_err(|e| WaeError::invalid_params("path", e.to_string()))?;

        let http_method = "get";
        let http_uri = ensure_lowercase_hex(url.path());
        let host = url.host_str().unwrap_or("").to_lowercase();

        let key_time = {
            use std::time::{SystemTime, UNIX_EPOCH};
            let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
            format!("{};{}", now, now + 7200)
        };

        let original_params: Vec<(String, String)> = url.query_pairs().map(|(k, v)| (k.to_string(), v.to_string())).collect();

        let mut params_for_sign: Vec<(String, String)> =
            original_params.iter().map(|(k, v)| (k.to_lowercase(), v.to_string())).collect();

        params_for_sign.sort_by(|a, b| a.0.cmp(&b.0));

        let mut params_list = Vec::new();
        let mut params_kv = Vec::new();
        let mut seen_keys = HashSet::new();

        for (k_lower, v) in &params_for_sign {
            let k_encoded = cos_encode(k_lower);

            if seen_keys.insert(k_lower.clone()) {
                params_list.push(k_encoded.clone());
            }

            params_kv.push(format!("{}={}", k_encoded, cos_encode(v)));
        }

        let http_params = params_kv.join("&");
        let param_list_str = params_list.join(";");
        let http_headers = if host.is_empty() { "".to_string() } else { format!("host={}", host) };

        let http_string = format!("{}\n{}\n{}\n{}\n", http_method, http_uri, http_params, http_headers);
        let http_string_hash = sha1_hex(&http_string);
        let string_to_sign = format!("sha1\n{}\n{}\n", key_time, http_string_hash);

        let sign_key = hmac_sha1_hex(config.secret_key.as_bytes(), &key_time);
        let signature = hmac_sha1_hex(sign_key.as_bytes(), &string_to_sign);

        let mut final_query_parts = Vec::new();

        final_query_parts.push("q-sign-algorithm=sha1".to_string());
        final_query_parts.push(format!("q-ak={}", cos_encode(&config.secret_id)));
        final_query_parts.push(format!("q-sign-time={}", cos_encode(&key_time)));
        final_query_parts.push(format!("q-key-time={}", cos_encode(&key_time)));
        final_query_parts.push(format!("q-header-list={}", if host.is_empty() { "" } else { "host" }));
        final_query_parts.push(format!("q-url-param-list={}", cos_encode(&param_list_str)));
        final_query_parts.push(format!("q-signature={}", signature));

        for (k, v) in &original_params {
            if v.is_empty() {
                final_query_parts.push(k.to_string());
            }
            else {
                final_query_parts.push(format!("{}={}", cos_encode(k), cos_encode(v)));
            }
        }

        let final_query = final_query_parts.join("&");
        url.set_query(Some(&final_query));

        Ok(url)
    }

    fn get_presigned_put_url(&self, key: &str, config: &StorageConfig) -> StorageResult<Url> {
        let host = config.endpoint.clone().unwrap_or_else(|| format!("{}.cos.{}.myqcloud.com", config.bucket, config.region));

        let now = Utc::now().timestamp();
        let start_time = (now / 3600) * 3600;
        let end_time = start_time + 7200;
        let key_time = format!("{};{}", start_time, end_time);

        let base_url = if host.starts_with("http") { Url::parse(&host) } else { Url::parse(&format!("https://{}", host)) }
            .map_err(|e| WaeError::invalid_params("base_url", e.to_string()))?;

        let mut url = base_url.join(key.trim_start_matches('/')).map_err(|e| WaeError::invalid_params("url", e.to_string()))?;

        let host_only = url.host_str().unwrap_or("").to_lowercase();

        let http_method = "put";
        let http_uri = ensure_lowercase_hex(url.path());
        let http_params = "";
        let http_headers = format!("host={}", host_only);
        let http_string = format!("{}\n{}\n{}\n{}\n", http_method, http_uri, http_params, http_headers);

        let http_string_hash = sha1_hex(&http_string);
        let string_to_sign = format!("sha1\n{}\n{}\n", key_time, http_string_hash);

        let sign_key = hmac_sha1_hex(config.secret_key.as_bytes(), &key_time);
        let signature = hmac_sha1_hex(sign_key.as_bytes(), &string_to_sign);

        let mut query_parts = Vec::new();
        query_parts.push("q-sign-algorithm=sha1".to_string());
        query_parts.push(format!("q-ak={}", cos_encode(&config.secret_id)));
        query_parts.push(format!("q-sign-time={}", cos_encode(&key_time)));
        query_parts.push(format!("q-key-time={}", cos_encode(&key_time)));
        query_parts.push("q-header-list=host".to_string());
        query_parts.push("q-url-param-list=".to_string());
        query_parts.push(format!("q-signature={}", signature));

        url.set_query(Some(&query_parts.join("&")));

        Ok(url)
    }
}
