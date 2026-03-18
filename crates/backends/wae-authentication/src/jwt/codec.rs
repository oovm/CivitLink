//! JWT 编解码模块

use base64::{Engine as _, engine::general_purpose};
use hmac::{Hmac, Mac as _};
use serde::{Deserialize, Serialize};
use sha2::{Sha256, Sha384, Sha512};
use std::fmt;
use wae_types::{WaeError, WaeErrorKind};

/// JWT 编解码错误
#[derive(Debug)]
pub enum JwtCodecError {
    /// 无效的令牌格式
    InvalidFormat,

    /// Base64 解码失败
    Base64Error(base64::DecodeError),

    /// JSON 序列化/反序列化失败
    JsonError(serde_json::Error),

    /// 无效的签名
    InvalidSignature,

    /// 无效的算法
    InvalidAlgorithm,

    /// 密钥错误
    KeyError,
}

impl fmt::Display for JwtCodecError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            JwtCodecError::InvalidFormat => write!(f, "invalid token format"),
            JwtCodecError::Base64Error(e) => write!(f, "base64 decode error: {}", e),
            JwtCodecError::JsonError(e) => write!(f, "json error: {}", e),
            JwtCodecError::InvalidSignature => write!(f, "invalid signature"),
            JwtCodecError::InvalidAlgorithm => write!(f, "invalid algorithm"),
            JwtCodecError::KeyError => write!(f, "key error"),
        }
    }
}

impl std::error::Error for JwtCodecError {}

impl From<base64::DecodeError> for JwtCodecError {
    fn from(err: base64::DecodeError) -> Self {
        JwtCodecError::Base64Error(err)
    }
}

impl From<serde_json::Error> for JwtCodecError {
    fn from(err: serde_json::Error) -> Self {
        JwtCodecError::JsonError(err)
    }
}

impl From<JwtCodecError> for WaeError {
    fn from(err: JwtCodecError) -> Self {
        match err {
            JwtCodecError::InvalidFormat => WaeError::invalid_token("malformed token"),
            JwtCodecError::Base64Error(_) => WaeError::invalid_token("invalid base64"),
            JwtCodecError::JsonError(_) => WaeError::invalid_token("invalid json"),
            JwtCodecError::InvalidSignature => WaeError::invalid_signature(),
            JwtCodecError::InvalidAlgorithm => WaeError::new(WaeErrorKind::InvalidAlgorithm),
            JwtCodecError::KeyError => WaeError::new(WaeErrorKind::KeyError),
        }
    }
}

/// JWT 结果类型
pub type JwtCodecResult<T> = Result<T, JwtCodecError>;

/// JWT 头部
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JwtHeader {
    /// 算法
    pub alg: String,
    /// 类型
    pub typ: String,
}

impl JwtHeader {
    /// 创建新的 JWT 头部
    pub fn new(alg: impl Into<String>) -> Self {
        Self { alg: alg.into(), typ: "JWT".to_string() }
    }
}

/// Base64URL 编码（URL_SAFE_NO_PAD）
pub fn base64url_encode(input: &[u8]) -> String {
    general_purpose::URL_SAFE_NO_PAD.encode(input)
}

/// Base64URL 解码（URL_SAFE_NO_PAD）
pub fn base64url_decode(input: &str) -> JwtCodecResult<Vec<u8>> {
    Ok(general_purpose::URL_SAFE_NO_PAD.decode(input)?)
}

/// 计算 HMAC 签名
pub fn hmac_sign(algorithm: &str, secret: &[u8], data: &[u8]) -> JwtCodecResult<Vec<u8>> {
    match algorithm {
        "HS256" => {
            let mut mac = Hmac::<Sha256>::new_from_slice(secret).map_err(|_| JwtCodecError::KeyError)?;
            mac.update(data);
            Ok(mac.finalize().into_bytes().to_vec())
        }
        "HS384" => {
            let mut mac = Hmac::<Sha384>::new_from_slice(secret).map_err(|_| JwtCodecError::KeyError)?;
            mac.update(data);
            Ok(mac.finalize().into_bytes().to_vec())
        }
        "HS512" => {
            let mut mac = Hmac::<Sha512>::new_from_slice(secret).map_err(|_| JwtCodecError::KeyError)?;
            mac.update(data);
            Ok(mac.finalize().into_bytes().to_vec())
        }
        _ => Err(JwtCodecError::InvalidAlgorithm),
    }
}

/// 验证 HMAC 签名
pub fn hmac_verify(algorithm: &str, secret: &[u8], data: &[u8], signature: &[u8]) -> JwtCodecResult<bool> {
    match algorithm {
        "HS256" => {
            let mut mac = Hmac::<Sha256>::new_from_slice(secret).map_err(|_| JwtCodecError::KeyError)?;
            mac.update(data);
            mac.verify_slice(signature).map_err(|_| JwtCodecError::InvalidSignature)?;
            Ok(true)
        }
        "HS384" => {
            let mut mac = Hmac::<Sha384>::new_from_slice(secret).map_err(|_| JwtCodecError::KeyError)?;
            mac.update(data);
            mac.verify_slice(signature).map_err(|_| JwtCodecError::InvalidSignature)?;
            Ok(true)
        }
        "HS512" => {
            let mut mac = Hmac::<Sha512>::new_from_slice(secret).map_err(|_| JwtCodecError::KeyError)?;
            mac.update(data);
            mac.verify_slice(signature).map_err(|_| JwtCodecError::InvalidSignature)?;
            Ok(true)
        }
        _ => Err(JwtCodecError::InvalidAlgorithm),
    }
}

/// 编码 JWT 令牌
pub fn encode_jwt<T: Serialize>(header: &JwtHeader, claims: &T, secret: &[u8]) -> JwtCodecResult<String> {
    let header_json = serde_json::to_string(header)?;
    let claims_json = serde_json::to_string(claims)?;

    let header_b64 = base64url_encode(header_json.as_bytes());
    let claims_b64 = base64url_encode(claims_json.as_bytes());

    let message = format!("{}.{}", header_b64, claims_b64);
    let signature = hmac_sign(&header.alg, secret, message.as_bytes())?;
    let signature_b64 = base64url_encode(&signature);

    Ok(format!("{}.{}", message, signature_b64))
}

/// 解码 JWT 令牌
pub fn decode_jwt<T: for<'de> Deserialize<'de>>(token: &str, secret: &[u8], validate_signature: bool) -> JwtCodecResult<T> {
    let parts: Vec<&str> = token.split('.').collect();
    if parts.len() != 3 {
        return Err(JwtCodecError::InvalidFormat);
    }

    let header_b64 = parts[0];
    let claims_b64 = parts[1];
    let signature_b64 = parts[2];

    let header_bytes = base64url_decode(header_b64)?;
    let header: JwtHeader = serde_json::from_slice(&header_bytes)?;

    let claims_bytes = base64url_decode(claims_b64)?;
    let claims: T = serde_json::from_slice(&claims_bytes)?;

    if validate_signature {
        let message = format!("{}.{}", header_b64, claims_b64);
        let signature = base64url_decode(signature_b64)?;
        hmac_verify(&header.alg, secret, message.as_bytes(), &signature)?;
    }

    Ok(claims)
}
