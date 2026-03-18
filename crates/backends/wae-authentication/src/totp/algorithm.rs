//! TOTP/HOTP 算法实现

use crate::totp::TotpAlgorithm;
use hmac::{Hmac, Mac};
use sha1::Sha1;
use sha2::{Sha256, Sha512};
use wae_types::{WaeError, WaeErrorKind};

/// TOTP 结果类型
pub type TotpResult<T> = Result<T, WaeError>;

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
fn compute_hmac(algorithm: TotpAlgorithm, key: &[u8], counter: &[u8]) -> TotpResult<Vec<u8>> {
    match algorithm {
        TotpAlgorithm::SHA1 => {
            let mut mac = Hmac::<Sha1>::new_from_slice(key)
                .map_err(|e| WaeError::new(WaeErrorKind::HmacError { reason: e.to_string() }))?;
            mac.update(counter);
            Ok(mac.finalize().into_bytes().to_vec())
        }
        TotpAlgorithm::SHA256 => {
            let mut mac = Hmac::<Sha256>::new_from_slice(key)
                .map_err(|e| WaeError::new(WaeErrorKind::HmacError { reason: e.to_string() }))?;
            mac.update(counter);
            Ok(mac.finalize().into_bytes().to_vec())
        }
        TotpAlgorithm::SHA512 => {
            let mut mac = Hmac::<Sha512>::new_from_slice(key)
                .map_err(|e| WaeError::new(WaeErrorKind::HmacError { reason: e.to_string() }))?;
            mac.update(counter);
            Ok(mac.finalize().into_bytes().to_vec())
        }
    }
}

/// 生成 HOTP (HMAC-based One-Time Password)
///
/// # Arguments
/// * `secret` - 密钥字节
/// * `counter` - 计数器值
/// * `digits` - 数字位数
/// * `algorithm` - 哈希算法
pub fn generate_hotp(secret: &[u8], counter: u64, digits: u32, algorithm: TotpAlgorithm) -> TotpResult<String> {
    let counter_bytes = counter.to_be_bytes();
    let hmac_result = compute_hmac(algorithm, secret, &counter_bytes)?;
    let code = dynamic_truncate(&hmac_result, digits);
    Ok(format!("{:0width$}", code, width = digits as usize))
}

/// 生成 TOTP (Time-based One-Time Password)
///
/// # Arguments
/// * `secret` - 密钥字节
/// * `timestamp` - 当前时间戳 (秒)
/// * `time_step` - 时间步长 (秒)
/// * `digits` - 数字位数
/// * `algorithm` - 哈希算法
pub fn generate_totp(
    secret: &[u8],
    timestamp: u64,
    time_step: u64,
    digits: u32,
    algorithm: TotpAlgorithm,
) -> TotpResult<String> {
    let counter = timestamp / time_step;
    generate_hotp(secret, counter, digits, algorithm)
}

/// 验证 TOTP 码
///
/// # Arguments
/// * `secret` - 密钥字节
/// * `code` - 用户输入的验证码
/// * `timestamp` - 当前时间戳 (秒)
/// * `time_step` - 时间步长 (秒)
/// * `digits` - 数字位数
/// * `algorithm` - 哈希算法
/// * `window` - 有效窗口 (前后允许的步数)
pub fn verify_totp(
    secret: &[u8],
    code: &str,
    timestamp: u64,
    time_step: u64,
    digits: u32,
    algorithm: TotpAlgorithm,
    window: u32,
) -> TotpResult<bool> {
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
///
/// # Arguments
/// * `secret` - 密钥字节
/// * `code` - 用户输入的验证码
/// * `counter` - 计数器值
/// * `digits` - 数字位数
/// * `algorithm` - 哈希算法
pub fn verify_hotp(secret: &[u8], code: &str, counter: u64, digits: u32, algorithm: TotpAlgorithm) -> TotpResult<bool> {
    let expected = generate_hotp(secret, counter, digits, algorithm)?;
    Ok(constant_time_compare(code, &expected))
}

/// 常量时间比较 (防止时序攻击)
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
