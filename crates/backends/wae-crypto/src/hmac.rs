//! HMAC 签名和验证模块

use crate::error::{CryptoError, CryptoResult};
use hmac::{Hmac, Mac as _};
use sha1::Sha1;
use sha2::{Sha256, Sha384, Sha512};

/// HMAC 算法
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HmacAlgorithm {
    /// HMAC-SHA1
    SHA1,
    /// HMAC-SHA256
    SHA256,
    /// HMAC-SHA384
    SHA384,
    /// HMAC-SHA512
    SHA512,
}

/// 计算 HMAC 签名
pub fn hmac_sign(algorithm: HmacAlgorithm, secret: &[u8], data: &[u8]) -> CryptoResult<Vec<u8>> {
    match algorithm {
        HmacAlgorithm::SHA1 => {
            let mut mac = Hmac::<Sha1>::new_from_slice(secret).map_err(|_| CryptoError::KeyError)?;
            mac.update(data);
            Ok(mac.finalize().into_bytes().to_vec())
        }
        HmacAlgorithm::SHA256 => {
            let mut mac = Hmac::<Sha256>::new_from_slice(secret).map_err(|_| CryptoError::KeyError)?;
            mac.update(data);
            Ok(mac.finalize().into_bytes().to_vec())
        }
        HmacAlgorithm::SHA384 => {
            let mut mac = Hmac::<Sha384>::new_from_slice(secret).map_err(|_| CryptoError::KeyError)?;
            mac.update(data);
            Ok(mac.finalize().into_bytes().to_vec())
        }
        HmacAlgorithm::SHA512 => {
            let mut mac = Hmac::<Sha512>::new_from_slice(secret).map_err(|_| CryptoError::KeyError)?;
            mac.update(data);
            Ok(mac.finalize().into_bytes().to_vec())
        }
    }
}

/// 验证 HMAC 签名
pub fn hmac_verify(algorithm: HmacAlgorithm, secret: &[u8], data: &[u8], signature: &[u8]) -> CryptoResult<bool> {
    match algorithm {
        HmacAlgorithm::SHA1 => {
            let mut mac = Hmac::<Sha1>::new_from_slice(secret).map_err(|_| CryptoError::KeyError)?;
            mac.update(data);
            mac.verify_slice(signature).map_err(|_| CryptoError::InvalidSignature)?;
            Ok(true)
        }
        HmacAlgorithm::SHA256 => {
            let mut mac = Hmac::<Sha256>::new_from_slice(secret).map_err(|_| CryptoError::KeyError)?;
            mac.update(data);
            mac.verify_slice(signature).map_err(|_| CryptoError::InvalidSignature)?;
            Ok(true)
        }
        HmacAlgorithm::SHA384 => {
            let mut mac = Hmac::<Sha384>::new_from_slice(secret).map_err(|_| CryptoError::KeyError)?;
            mac.update(data);
            mac.verify_slice(signature).map_err(|_| CryptoError::InvalidSignature)?;
            Ok(true)
        }
        HmacAlgorithm::SHA512 => {
            let mut mac = Hmac::<Sha512>::new_from_slice(secret).map_err(|_| CryptoError::KeyError)?;
            mac.update(data);
            mac.verify_slice(signature).map_err(|_| CryptoError::InvalidSignature)?;
            Ok(true)
        }
    }
}
