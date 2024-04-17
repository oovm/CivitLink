//! Redis 缓存驱动实现
//!
//! 提供基于 Redis 的缓存驱动，使用原生 Redis 命令进行缓存操作。

use crate::{CacheDriver, CacheStats, CacheValue};
use gg_core::{GError, GErrorKind, GResult};
use std::time::Duration;

/// Redis 连接配置
///
/// 包含建立 Redis 连接所需的全部参数。
pub struct RedisConfig {
    /// Redis 服务器地址
    pub host: String,
    /// Redis 服务器端口
    pub port: u16,
    /// 数据库编号
    pub db: u8,
    /// 认证密码
    pub password: Option<String>,
}

impl Default for RedisConfig {
    fn default() -> Self {
        Self { host: "127.0.0.1".to_string(), port: 6379, db: 0, password: None }
    }
}

/// Redis 缓存驱动
///
/// 基于 Redis 的缓存实现，使用原生 Redis 命令进行缓存操作，
/// 支持批量操作、递增递减和 TTL 刷新。
pub struct RedisCacheDriver {
    /// Redis 连接
    conn: redis::Connection,
    /// 键前缀
    prefix: String,
    /// 命中次数
    hits: u64,
    /// 未命中次数
    misses: u64,
}

impl RedisCacheDriver {
    /// 创建新的 Redis 缓存驱动
    ///
    /// 根据配置连接到 Redis 服务器。
    ///
    /// # 参数
    ///
    /// - `config`: Redis 连接配置
    pub fn new(config: &RedisConfig) -> GResult<Self> {
        let password_part = match &config.password {
            Some(pw) => format!(":{}@", pw),
            None => String::new(),
        };
        let conn_str = format!("redis://{}{}:{}/{}", password_part, config.host, config.port, config.db);
        let conn = redis::Client::open(conn_str)
            .map_err(|e| GError {
                kind: GErrorKind::Io,
                message: format!("创建 Redis 客户端失败: {}", e),
            })?
            .get_connection()
            .map_err(|e| GError {
                kind: GErrorKind::Io,
                message: format!("连接 Redis 服务器失败: {}", e),
            })?;
        Ok(Self { conn, prefix: String::new(), hits: 0, misses: 0 })
    }

    /// 使用键前缀创建 Redis 缓存驱动
    ///
    /// # 参数
    ///
    /// - `config`: Redis 连接配置
    /// - `prefix`: 键名前缀
    pub fn with_prefix(config: &RedisConfig, prefix: &str) -> GResult<Self> {
        let mut driver = Self::new(config)?;
        driver.prefix = prefix.to_string();
        Ok(driver)
    }

    /// 获取带前缀的完整键名
    fn full_key(&self, key: &str) -> String {
        if self.prefix.is_empty() {
            key.to_string()
        }
        else {
            format!("{}:{}", self.prefix, key)
        }
    }

    /// 执行 Redis 命令
    fn execute_cmd(&mut self, cmd: &mut redis::Cmd) -> GResult<redis::Value> {
        cmd.query(&mut self.conn).map_err(|e| GError {
            kind: GErrorKind::Io,
            message: format!("执行 Redis 命令失败: {}", e),
        })
    }
}

/// 将 CacheValue 序列化为 Redis 兼容的字符串
fn cache_value_to_string(value: &CacheValue) -> String {
    match value {
        CacheValue::Null => "null".to_string(),
        CacheValue::Integer(n) => format!("i:{}", n),
        CacheValue::Real(f) => format!("f:{}", f),
        CacheValue::Text(s) => format!("s:{}", s),
        CacheValue::Blob(b) => format!("b:{}", base64_encode(b)),
        CacheValue::Bool(b) => format!("bool:{}", b),
    }
}

/// 从 Redis 字符串反序列化为 CacheValue
fn string_to_cache_value(s: &str) -> GResult<CacheValue> {
    if s == "null" {
        return Ok(CacheValue::Null);
    }
    if let Some(val) = s.strip_prefix("i:") {
        return val.parse::<i64>().map(CacheValue::Integer).map_err(|e| GError {
            kind: GErrorKind::Runtime,
            message: format!("解析整数值失败: {}", e),
        });
    }
    if let Some(val) = s.strip_prefix("f:") {
        return val.parse::<f64>().map(CacheValue::Real).map_err(|e| GError {
            kind: GErrorKind::Runtime,
            message: format!("解析浮点数值失败: {}", e),
        });
    }
    if let Some(val) = s.strip_prefix("s:") {
        return Ok(CacheValue::Text(val.to_string()));
    }
    if let Some(val) = s.strip_prefix("b:") {
        return base64_decode(val).map(CacheValue::Blob).map_err(|e| GError {
            kind: GErrorKind::Runtime,
            message: format!("解析二进制数据失败: {}", e),
        });
    }
    if let Some(val) = s.strip_prefix("bool:") {
        return val.parse::<bool>().map(CacheValue::Bool).map_err(|e| GError {
            kind: GErrorKind::Runtime,
            message: format!("解析布尔值失败: {}", e),
        });
    }
    Err(GError {
        kind: GErrorKind::Runtime,
        message: format!("无法识别的缓存值格式: {}", s),
    })
}

/// 简易 Base64 编码
fn base64_encode(data: &[u8]) -> String {
    const CHARS: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut result = String::new();
    for chunk in data.chunks(3) {
        let b0 = chunk[0] as u32;
        let b1 = if chunk.len() > 1 { chunk[1] as u32 } else { 0 };
        let b2 = if chunk.len() > 2 { chunk[2] as u32 } else { 0 };
        let triple = (b0 << 16) | (b1 << 8) | b2;
        result.push(CHARS[((triple >> 18) & 0x3F) as usize] as char);
        result.push(CHARS[((triple >> 12) & 0x3F) as usize] as char);
        if chunk.len() > 1 {
            result.push(CHARS[((triple >> 6) & 0x3F) as usize] as char);
        }
        else {
            result.push('=');
        }
        if chunk.len() > 2 {
            result.push(CHARS[(triple & 0x3F) as usize] as char);
        }
        else {
            result.push('=');
        }
    }
    result
}

/// 简易 Base64 解码
fn base64_decode(input: &str) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    const CHARS: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let input = input.trim_end_matches('=');
    let mut result = Vec::new();
    let mut buffer: u32 = 0;
    let mut bits = 0u32;
    for ch in input.chars() {
        let val = CHARS.iter().position(|&c| c as char == ch).ok_or("无效的 Base64 字符")? as u32;
        buffer = (buffer << 6) | val;
        bits += 6;
        if bits >= 8 {
            bits -= 8;
            result.push((buffer >> bits) as u8);
        }
    }
    Ok(result)
}

impl CacheDriver for RedisCacheDriver {
    fn get(&mut self, key: &str) -> GResult<Option<CacheValue>> {
        let full_key = self.full_key(key);
        let result: Option<String> = redis::cmd("GET")
            .arg(&full_key)
            .query(&mut self.conn)
            .map_err(|e| GError {
                kind: GErrorKind::Io,
                message: format!("Redis GET 命令失败: {}", e),
            })?;
        match result {
            Some(s) => {
                self.hits += 1;
                string_to_cache_value(&s)
            }
            None => {
                self.misses += 1;
                Ok(None)
            }
        }
    }

    fn set(&mut self, key: &str, value: CacheValue, ttl: Option<Duration>) -> GResult<()> {
        let full_key = self.full_key(key);
        let value_str = cache_value_to_string(&value);
        match ttl {
            Some(duration) => {
                redis::cmd("SETEX")
                    .arg(&full_key)
                    .arg(duration.as_secs() as u64)
                    .arg(&value_str)
                    .query(&mut self.conn)
                    .map_err(|e| GError {
                        kind: GErrorKind::Io,
                        message: format!("Redis SETEX 命令失败: {}", e),
                    })?;
            }
            None => {
                redis::cmd("SET")
                    .arg(&full_key)
                    .arg(&value_str)
                    .query(&mut self.conn)
                    .map_err(|e| GError {
                        kind: GErrorKind::Io,
                        message: format!("Redis SET 命令失败: {}", e),
                    })?;
            }
        }
        Ok(())
    }

    fn delete(&mut self, key: &str) -> GResult<bool> {
        let full_key = self.full_key(key);
        let deleted: i64 = redis::cmd("DEL")
            .arg(&full_key)
            .query(&mut self.conn)
            .map_err(|e| GError {
                kind: GErrorKind::Io,
                message: format!("Redis DEL 命令失败: {}", e),
            })?;
        Ok(deleted > 0)
    }

    fn exists(&mut self, key: &str) -> GResult<bool> {
        let full_key = self.full_key(key);
        let exists: bool = redis::cmd("EXISTS")
            .arg(&full_key)
            .query(&mut self.conn)
            .map_err(|e| GError {
                kind: GErrorKind::Io,
                message: format!("Redis EXISTS 命令失败: {}", e),
            })?;
        Ok(exists)
    }

    fn clear(&mut self) -> GResult<()> {
        redis::cmd("FLUSHDB")
            .query(&mut self.conn)
            .map_err(|e| GError {
                kind: GErrorKind::Io,
                message: format!("Redis FLUSHDB 命令失败: {}", e),
            })?;
        Ok(())
    }

    fn get_many(&mut self, keys: &[&str]) -> GResult<Vec<Option<CacheValue>>> {
        if keys.is_empty() {
            return Ok(Vec::new());
        }
        let full_keys: Vec<String> = keys.iter().map(|k| self.full_key(k)).collect();
        let args: Vec<&str> = full_keys.iter().map(|s| s.as_str()).collect();
        let results: Vec<Option<String>> = redis::cmd("MGET")
            .arg(&args)
            .query(&mut self.conn)
            .map_err(|e| GError {
                kind: GErrorKind::Io,
                message: format!("Redis MGET 命令失败: {}", e),
            })?;
        let mut values = Vec::with_capacity(results.len());
        for result in results {
            match result {
                Some(s) => {
                    self.hits += 1;
                    values.push(Some(string_to_cache_value(&s)?));
                }
                None => {
                    self.misses += 1;
                    values.push(None);
                }
            }
        }
        Ok(values)
    }

    fn set_many(&mut self, entries: &[(&str, CacheValue)], ttl: Option<Duration>) -> GResult<()> {
        if entries.is_empty() {
            return Ok(());
        }
        for (key, value) in entries {
            self.set(key, value.clone(), ttl)?;
        }
        Ok(())
    }

    fn increment(&mut self, key: &str, delta: i64) -> GResult<i64> {
        let full_key = self.full_key(key);
        let result: i64 = redis::cmd("INCRBY")
            .arg(&full_key)
            .arg(delta)
            .query(&mut self.conn)
            .map_err(|e| GError {
                kind: GErrorKind::Io,
                message: format!("Redis INCRBY 命令失败: {}", e),
            })?;
        Ok(result)
    }

    fn decrement(&mut self, key: &str, delta: i64) -> GResult<i64> {
        let full_key = self.full_key(key);
        let result: i64 = redis::cmd("DECRBY")
            .arg(&full_key)
            .arg(delta)
            .query(&mut self.conn)
            .map_err(|e| GError {
                kind: GErrorKind::Io,
                message: format!("Redis DECRBY 命令失败: {}", e),
            })?;
        Ok(result)
    }

    fn touch(&mut self, key: &str, ttl: Duration) -> GResult<bool> {
        let full_key = self.full_key(key);
        let result: bool = redis::cmd("EXPIRE")
            .arg(&full_key)
            .arg(ttl.as_secs() as u64)
            .query(&mut self.conn)
            .map_err(|e| GError {
                kind: GErrorKind::Io,
                message: format!("Redis EXPIRE 命令失败: {}", e),
            })?;
        Ok(result)
    }

    fn stats(&self) -> CacheStats {
        CacheStats { hits: self.hits, misses: self.misses }
    }
}
