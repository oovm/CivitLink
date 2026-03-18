#![warn(missing_docs)]
//! WAE Crypto - 统一加密抽象模块

pub mod base64;
pub mod error;
pub mod hash;
pub mod hmac;
pub mod password;
pub mod totp;

pub use base64::{base64_decode, base64_encode, base64url_decode, base64url_encode};
pub use error::{CryptoError, CryptoResult};
pub use hash::{HashAlgorithm, hash};
pub use hmac::{HmacAlgorithm, hmac_sign, hmac_verify};
pub use password::{PasswordAlgorithm, PasswordHasher, PasswordHasherConfig};
pub use totp::{TotpAlgorithm, TotpSecret, generate_hotp, generate_totp, verify_hotp, verify_totp};
