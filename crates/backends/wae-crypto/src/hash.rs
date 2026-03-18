//! 哈希算法模块

use crate::error::CryptoResult;
use sha1::Sha1;
use sha2::{Digest, Sha256, Sha384, Sha512};

/// 哈希算法
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HashAlgorithm {
    /// SHA-1
    SHA1,
    /// SHA-256
    SHA256,
    /// SHA-384
    SHA384,
    /// SHA-512
    SHA512,
}

/// 计算哈希值
pub fn hash(algorithm: HashAlgorithm, data: &[u8]) -> CryptoResult<Vec<u8>> {
    match algorithm {
        HashAlgorithm::SHA1 => {
            let mut hasher = Sha1::new();
            hasher.update(data);
            Ok(hasher.finalize().to_vec())
        }
        HashAlgorithm::SHA256 => {
            let mut hasher = Sha256::new();
            hasher.update(data);
            Ok(hasher.finalize().to_vec())
        }
        HashAlgorithm::SHA384 => {
            let mut hasher = Sha384::new();
            hasher.update(data);
            Ok(hasher.finalize().to_vec())
        }
        HashAlgorithm::SHA512 => {
            let mut hasher = Sha512::new();
            hasher.update(data);
            Ok(hasher.finalize().to_vec())
        }
    }
}
