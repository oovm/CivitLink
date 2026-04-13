//! 文件缓存驱动实现
//!
//! 提供基于文件系统的缓存驱动，使用 JSON 序列化存储缓存值。

use crate::{CacheDriver, CacheStats, CacheValue};
use gg_core::{GError, GErrorKind, GResult};
use std::{
    fs,
    path::{Path, PathBuf},
    time::{Duration, SystemTime, UNIX_EPOCH},
};

/// 文件缓存条目
///
/// 序列化存储在文件中的缓存数据结构。
#[derive(serde::Serialize, serde::Deserialize)]
struct FileCacheEntry {
    /// 缓存值
    value: CacheValue,
    /// 过期时间戳（UNIX 秒），None 表示永不过期
    expires_at: Option<u64>,
}

/// 文件缓存驱动
///
/// 基于文件系统的缓存实现，将缓存数据以 JSON 格式存储在指定目录中。
/// 支持 TTL 过期和自动清理。
pub struct FileCacheDriver {
    /// 缓存目录路径
    base_dir: PathBuf,
    /// 命中次数
    hits: u64,
    /// 未命中次数
    misses: u64,
}

impl FileCacheDriver {
    /// 创建新的文件缓存驱动
    ///
    /// 如果指定目录不存在，将自动创建。
    ///
    /// # 参数
    ///
    /// - `base_dir`: 缓存文件存储目录
    pub fn new(base_dir: &Path) -> GResult<Self> {
        fs::create_dir_all(base_dir).map_err(|e| GError {
            kind: GErrorKind::Io,
            message: format!("无法创建缓存目录 '{}': {}", base_dir.display(), e),
        })?;
        Ok(Self { base_dir: base_dir.to_path_buf(), hits: 0, misses: 0 })
    }

    /// 将缓存键转换为文件路径
    fn key_to_path(&self, key: &str) -> PathBuf {
        let safe_name = key.replace(['/', '\\', ':', '*', '?', '"', '<', '>', '|'], "_");
        self.base_dir.join(format!("{}.cache", safe_name))
    }

    /// 获取当前 UNIX 时间戳（秒）
    fn current_timestamp() -> u64 {
        SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs()
    }

    /// 清理所有过期缓存文件
    ///
    /// 遍历缓存目录中的所有文件，删除已过期的缓存条目。
    pub fn cleanup_expired(&mut self) -> GResult<usize> {
        let mut removed = 0;
        let entries = fs::read_dir(&self.base_dir)
            .map_err(|e| GError { kind: GErrorKind::Io, message: format!("无法读取缓存目录: {}", e) })?;

        for entry in entries {
            let entry =
                entry.map_err(|e| GError { kind: GErrorKind::Io, message: format!("读取目录条目失败: {}", e) })?;
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) == Some("cache") {
                if let Ok(content) = fs::read_to_string(&path) {
                    if let Ok(cache_entry) = serde_json::from_str::<FileCacheEntry>(&content) {
                        if let Some(expires_at) = cache_entry.expires_at {
                            if Self::current_timestamp() >= expires_at {
                                let _ = fs::remove_file(&path);
                                removed += 1;
                            }
                        }
                    }
                }
            }
        }
        Ok(removed)
    }
}

impl CacheDriver for FileCacheDriver {
    fn get(&mut self, key: &str) -> GResult<Option<CacheValue>> {
        let path = self.key_to_path(key);
        if !path.exists() {
            self.misses += 1;
            return Ok(None);
        }

        let content = fs::read_to_string(&path)
            .map_err(|e| GError { kind: GErrorKind::Io, message: format!("读取缓存文件失败: {}", e) })?;

        let entry: FileCacheEntry = serde_json::from_str(&content)
            .map_err(|e| GError { kind: GErrorKind::Runtime, message: format!("反序列化缓存条目失败: {}", e) })?;

        if let Some(expires_at) = entry.expires_at {
            if Self::current_timestamp() >= expires_at {
                let _ = fs::remove_file(&path);
                self.misses += 1;
                return Ok(None);
            }
        }

        self.hits += 1;
        Ok(Some(entry.value))
    }

    fn set(&mut self, key: &str, value: CacheValue, ttl: Option<Duration>) -> GResult<()> {
        let path = self.key_to_path(key);
        let expires_at = ttl.map(|d| Self::current_timestamp() + d.as_secs());
        let entry = FileCacheEntry { value, expires_at };
        let content = serde_json::to_string(&entry)
            .map_err(|e| GError { kind: GErrorKind::Runtime, message: format!("序列化缓存条目失败: {}", e) })?;
        fs::write(&path, content)
            .map_err(|e| GError { kind: GErrorKind::Io, message: format!("写入缓存文件失败: {}", e) })?;
        Ok(())
    }

    fn delete(&mut self, key: &str) -> GResult<bool> {
        let path = self.key_to_path(key);
        if path.exists() {
            fs::remove_file(&path)
                .map_err(|e| GError { kind: GErrorKind::Io, message: format!("删除缓存文件失败: {}", e) })?;
            Ok(true)
        }
        else {
            Ok(false)
        }
    }

    fn exists(&mut self, key: &str) -> GResult<bool> {
        let path = self.key_to_path(key);
        if !path.exists() {
            return Ok(false);
        }

        let content = fs::read_to_string(&path)
            .map_err(|e| GError { kind: GErrorKind::Io, message: format!("读取缓存文件失败: {}", e) })?;

        let entry: FileCacheEntry = serde_json::from_str(&content)
            .map_err(|e| GError { kind: GErrorKind::Runtime, message: format!("反序列化缓存条目失败: {}", e) })?;

        if let Some(expires_at) = entry.expires_at {
            if Self::current_timestamp() >= expires_at {
                let _ = fs::remove_file(&path);
                return Ok(false);
            }
        }
        Ok(true)
    }

    fn clear(&mut self) -> GResult<()> {
        let entries = fs::read_dir(&self.base_dir)
            .map_err(|e| GError { kind: GErrorKind::Io, message: format!("无法读取缓存目录: {}", e) })?;

        for entry in entries {
            if let Ok(entry) = entry {
                let path = entry.path();
                if path.extension().and_then(|e| e.to_str()) == Some("cache") {
                    let _ = fs::remove_file(&path);
                }
            }
        }
        Ok(())
    }

    fn stats(&self) -> CacheStats {
        CacheStats { hits: self.hits, misses: self.misses }
    }
}
