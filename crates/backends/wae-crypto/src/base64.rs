//! Base64 编解码模块

use crate::error::CryptoResult;
use base64::{Engine as _, engine::general_purpose};

/// 标准 Base64 编码
pub fn base64_encode(input: &[u8]) -> String {
    general_purpose::STANDARD.encode(input)
}

/// 标准 Base64 解码
pub fn base64_decode(input: &str) -> CryptoResult<Vec<u8>> {
    Ok(general_purpose::STANDARD.decode(input)?)
}

/// Base64URL 编码（URL 安全无填充）
pub fn base64url_encode(input: &[u8]) -> String {
    general_purpose::URL_SAFE_NO_PAD.encode(input)
}

/// Base64URL 解码（URL 安全无填充）
pub fn base64url_decode(input: &str) -> CryptoResult<Vec<u8>> {
    Ok(general_purpose::URL_SAFE_NO_PAD.decode(input)?)
}
