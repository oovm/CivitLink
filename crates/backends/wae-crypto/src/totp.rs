//! TOTP/HOTP 模块

use crate::{
    base64::base64_encode,
    error::{CryptoError, CryptoResult},
    hmac::{HmacAlgorithm, hmac_sign},
};

/// TOTP 算法
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TotpAlgorithm {
    /// SHA-1
    SHA1,
    /// SHA-256
    SHA256,
    /// SHA-512
    SHA512,
}

impl Default for TotpAlgorithm {
    fn default() -> Self {
        TotpAlgorithm::SHA1
    }
}

/// Base32 字符集
const BASE32_CHARS: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ234567";

/// TOTP 密钥
#[derive(Debug, Clone)]
pub struct TotpSecret {
    bytes: Vec<u8>,
    base32: String,
}

impl TotpSecret {
    /// 生成新的随机密钥
    pub fn generate(length: usize) -> CryptoResult<Self> {
        use rand::rngs::OsRng;
        use rand::RngCore;
        let mut rng = OsRng;
        let mut bytes = vec![0u8; length];
        rng.fill_bytes(&mut bytes);
        Self::from_bytes(&bytes)
    }

    /// 生成推荐长度的密钥（20 字节）
    pub fn generate_default() -> CryptoResult<Self> {
        Self::generate(20)
    }

    /// 从字节创建密钥
    pub fn from_bytes(bytes: &[u8]) -> CryptoResult<Self> {
        let base32 = Self::encode_base32(bytes);
        Ok(Self { bytes: bytes.to_vec(), base32 })
    }

    /// 从 Base32 字符串创建密钥
    pub fn from_base32(s: &str) -> CryptoResult<Self> {
        let bytes = Self::decode_base32(s)?;
        Ok(Self { bytes, base32: s.to_uppercase().replace(" ", "") })
    }

    /// 获取原始字节
    pub fn as_bytes(&self) -> &[u8] {
        &self.bytes
    }

    /// 获取 Base32 编码
    pub fn as_base32(&self) -> &str {
        &self.base32
    }

    /// 获取密钥长度（字节）
    pub fn len(&self) -> usize {
        self.bytes.len()
    }

    /// 检查密钥是否为空
    pub fn is_empty(&self) -> bool {
        self.bytes.is_empty()
    }

    /// 编码为 Base32
    fn encode_base32(data: &[u8]) -> String {
        let mut result = String::new();
        let mut i = 0;
        let n = data.len();

        while i < n {
            let mut word: u64 = 0;
            let mut bits = 0;

            for j in 0..5 {
                if i + j < n {
                    word = (word << 8) | (data[i + j] as u64);
                    bits += 8;
                }
            }

            i += 5;

            while bits >= 5 {
                bits -= 5;
                let index = ((word >> bits) & 0x1F) as usize;
                result.push(BASE32_CHARS[index] as char);
            }

            if bits > 0 {
                let index = ((word << (5 - bits)) & 0x1F) as usize;
                result.push(BASE32_CHARS[index] as char);
            }
        }

        result
    }

    /// 解码 Base32
    fn decode_base32(s: &str) -> CryptoResult<Vec<u8>> {
        let s = s.to_uppercase().replace(" ", "").replace("-", "");
        let mut result = Vec::new();
        let chars: Vec<char> = s.chars().collect();

        let mut i = 0;
        while i < chars.len() {
            let mut word: u64 = 0;
            let mut bits = 0;

            for j in 0..8 {
                if i + j < chars.len() {
                    let val = Self::base32_char_to_value(chars[i + j])?;
                    word = (word << 5) | (val as u64);
                    bits += 5;
                }
            }

            i += 8;

            while bits >= 8 {
                bits -= 8;
                result.push(((word >> bits) & 0xFF) as u8);
            }
        }

        Ok(result)
    }

    /// Base32 字符转值
    fn base32_char_to_value(c: char) -> CryptoResult<u8> {
        match c {
            'A'..='Z' => Ok((c as u8) - b'A'),
            '2'..='7' => Ok((c as u8) - b'2' + 26),
            _ => Err(CryptoError::Base32Error(format!("Invalid character: {}", c))),
        }
    }
}

impl std::fmt::Display for TotpSecret {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.base32)
    }
}

/// 密钥格式化选项
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SecretFormat {
    /// Base32 格式（无分隔符）
    Base32,
    /// Base32 格式（每 4 字符空格分隔）
    Base32Spaced,
    /// 原始字节（十六进制）
    Raw,
    /// Base64 格式
    Base64,
}

impl TotpSecret {
    /// 格式化密钥
    pub fn format(&self, format: SecretFormat) -> String {
        match format {
            SecretFormat::Base32 => self.base32.clone(),
            SecretFormat::Base32Spaced => self
                .base32
                .as_bytes()
                .chunks(4)
                .map(|chunk| std::str::from_utf8(chunk).unwrap_or(""))
                .collect::<Vec<_>>()
                .join(" "),
            SecretFormat::Raw => self.bytes.iter().map(|b| format!("{:02x}", b)).collect(),
            SecretFormat::Base64 => base64_encode(&self.bytes),
        }
    }
}

/// 动态截取函数
fn dynamic_truncate(hmac_result: &[u8], digits: u32) -> u32 {
    let offset = (hmac_result.last().unwrap() & 0x0F) as usize;
    let binary = ((hmac_result[offset] as u32 & 0x7F) << 24)
        | ((hmac_result[offset + 1] as u32 & 0xFF) << 16)
        | ((hmac_result[offset + 2] as u32 & 0xFF) << 8)
        | (hmac_result[offset + 3] as u32 & 0xFF);

    let power = 10u32.pow(digits);
    binary % power
}

/// 计算 HMAC
fn compute_hmac(algorithm: TotpAlgorithm, key: &[u8], counter: &[u8]) -> CryptoResult<Vec<u8>> {
    let hmac_alg = match algorithm {
        TotpAlgorithm::SHA1 => HmacAlgorithm::SHA1,
        TotpAlgorithm::SHA256 => HmacAlgorithm::SHA256,
        TotpAlgorithm::SHA512 => HmacAlgorithm::SHA512,
    };
    hmac_sign(hmac_alg, key, counter)
}

/// 生成 HOTP
pub fn generate_hotp(secret: &[u8], counter: u64, digits: u32, algorithm: TotpAlgorithm) -> CryptoResult<String> {
    let counter_bytes = counter.to_be_bytes();
    let hmac_result = compute_hmac(algorithm, secret, &counter_bytes)?;
    let code = dynamic_truncate(&hmac_result, digits);
    Ok(format!("{:0width$}", code, width = digits as usize))
}

/// 生成 TOTP
pub fn generate_totp(
    secret: &[u8],
    timestamp: u64,
    time_step: u64,
    digits: u32,
    algorithm: TotpAlgorithm,
) -> CryptoResult<String> {
    let counter = timestamp / time_step;
    generate_hotp(secret, counter, digits, algorithm)
}

/// 验证 TOTP 码
pub fn verify_totp(
    secret: &[u8],
    code: &str,
    timestamp: u64,
    time_step: u64,
    digits: u32,
    algorithm: TotpAlgorithm,
    window: u32,
) -> CryptoResult<bool> {
    let current_counter = timestamp / time_step;

    for i in -(window as i64)..=(window as i64) {
        let counter = (current_counter as i64 + i) as u64;
        let expected = generate_hotp(secret, counter, digits, algorithm)?;

        if constant_time_compare(code, &expected) {
            return Ok(true);
        }
    }

    Ok(false)
}

/// 验证 HOTP 码
pub fn verify_hotp(secret: &[u8], code: &str, counter: u64, digits: u32, algorithm: TotpAlgorithm) -> CryptoResult<bool> {
    let expected = generate_hotp(secret, counter, digits, algorithm)?;
    Ok(constant_time_compare(code, &expected))
}

/// 常量时间比较（防止时序攻击）
fn constant_time_compare(a: &str, b: &str) -> bool {
    if a.len() != b.len() {
        return false;
    }

    let a_bytes = a.as_bytes();
    let b_bytes = b.as_bytes();

    let mut result = 0u8;
    for i in 0..a.len() {
        result |= a_bytes[i] ^ b_bytes[i];
    }

    result == 0
}

/// 获取当前时间步
pub fn get_time_step(timestamp: u64, time_step: u64) -> u64 {
    timestamp / time_step
}

/// 获取当前步剩余时间
pub fn get_remaining_seconds(timestamp: u64, time_step: u64) -> u64 {
    time_step - (timestamp % time_step)
}
