//! TOTP 密钥管理

use base64::{Engine, engine::general_purpose::STANDARD as BASE64_STANDARD};
use rand::Rng;
use wae_types::{WaeError, WaeErrorKind};

/// TOTP 结果类型
pub type TotpResult<T> = Result<T, WaeError>;

/// Base32 字符集
const BASE32_CHARS: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ234567";

/// TOTP 密钥
#[derive(Debug, Clone)]
pub struct TotpSecret {
    /// 原始字节
    bytes: Vec<u8>,

    /// Base32 编码
    base32: String,
}

impl TotpSecret {
    /// 生成新的随机密钥
    ///
    /// # Arguments
    /// * `length` - 密钥长度 (字节)，推荐 20 字节 (160 位)
    pub fn generate(length: usize) -> TotpResult<Self> {
        let mut bytes = vec![0u8; length];
        rand::rng().fill_bytes(&mut bytes);
        Self::from_bytes(&bytes)
    }

    /// 生成推荐长度的密钥 (20 字节)
    pub fn generate_default() -> TotpResult<Self> {
        Self::generate(20)
    }

    /// 从字节创建密钥
    pub fn from_bytes(bytes: &[u8]) -> TotpResult<Self> {
        let base32 = Self::encode_base32(bytes);
        Ok(Self { bytes: bytes.to_vec(), base32 })
    }

    /// 从 Base32 字符串创建密钥
    pub fn from_base32(s: &str) -> TotpResult<Self> {
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

    /// 获取密钥长度 (字节)
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
    fn decode_base32(s: &str) -> TotpResult<Vec<u8>> {
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
    fn base32_char_to_value(c: char) -> TotpResult<u8> {
        match c {
            'A'..='Z' => Ok((c as u8) - b'A'),
            '2'..='7' => Ok((c as u8) - b'2' + 26),
            _ => Err(WaeError::new(WaeErrorKind::Base32Error { reason: format!("Invalid character: {}", c) })),
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
    /// Base32 格式 (无分隔符)
    Base32,
    /// Base32 格式 (每 4 字符空格分隔)
    Base32Spaced,
    /// 原始字节
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
            SecretFormat::Base64 => BASE64_STANDARD.encode(&self.bytes),
        }
    }
}
