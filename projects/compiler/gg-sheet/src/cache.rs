//! 配置表编译缓存模块
//! 提供编译哈希缓存和依赖图的持久化存储

use std::{
    collections::{HashMap, HashSet},
    path::{Path, PathBuf},
};

use crate::dependency::SheetDependencyGraph;

/// 缓存文件魔数
const CACHE_MAGIC: &[u8; 8] = b"GGSHEET\0";

/// 缓存文件格式版本号
const CACHE_VERSION: u32 = 1;

/// 缓存文件头
#[derive(serde::Serialize, serde::Deserialize)]
struct CacheHeader {
    /// 魔数
    magic: [u8; 8],
    /// 版本号
    version: u32,
    /// 数据区 CRC32 校验和
    crc32: u32,
}

/// 计算 CRC32 校验和
///
/// 使用 ISO 3309 标准多项式 0xEDB88320
fn compute_crc32(data: &[u8]) -> u32 {
    let mut crc: u32 = 0xFFFFFFFF;
    for &byte in data {
        crc ^= byte as u32;
        for _ in 0..8 {
            if crc & 1 == 1 {
                crc = (crc >> 1) ^ 0xEDB88320;
            }
            else {
                crc >>= 1;
            }
        }
    }
    !crc
}

/// 配置表编译缓存
///
/// 将文件哈希和依赖图持久化到磁盘，支持增量编译的缓存恢复
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SheetCache {
    /// 文件路径到内容哈希的映射
    pub file_hashes: HashMap<PathBuf, u64>,
    /// 表间依赖图
    pub dependency_graph: SheetDependencyGraph,
}

impl SheetCache {
    /// 创建空的编译缓存
    pub fn new() -> Self {
        Self { file_hashes: HashMap::new(), dependency_graph: SheetDependencyGraph::new() }
    }

    /// 缓存文件名
    const CACHE_FILE_NAME: &'static str = ".sheet-cache";

    /// 从输出目录加载缓存
    ///
    /// 读取 .sheet-cache 文件并反序列化。优先尝试二进制格式，若失败则回退到旧文本格式。
    /// 如果文件不存在或格式错误，返回空缓存。
    pub fn load(output_dir: &Path) -> Self {
        let cache_path = output_dir.join(Self::CACHE_FILE_NAME);
        if !cache_path.exists() {
            return Self::new();
        }

        let bytes = match std::fs::read(&cache_path) {
            Ok(b) => b,
            Err(_) => return Self::new(),
        };

        if bytes.len() >= CACHE_MAGIC.len() && &bytes[..CACHE_MAGIC.len()] == CACHE_MAGIC {
            Self::load_binary(&bytes)
        }
        else {
            match std::str::from_utf8(&bytes) {
                Ok(text) => Self::deserialize_legacy(text),
                Err(_) => Self::new(),
            }
        }
    }

    /// 从二进制格式数据加载缓存
    fn load_binary(data: &[u8]) -> Self {
        let header_size = 8 + 4 + 4;
        if data.len() < header_size {
            return Self::new();
        }

        let magic: [u8; 8] = data[..8].try_into().unwrap_or([0; 8]);
        if &magic != CACHE_MAGIC {
            return Self::new();
        }

        let version = u32::from_le_bytes(data[8..12].try_into().unwrap_or([0; 4]));
        if version != CACHE_VERSION {
            return Self::new();
        }

        let stored_crc32 = u32::from_le_bytes(data[12..16].try_into().unwrap_or([0; 4]));
        let payload = &data[header_size..];
        let computed_crc32 = compute_crc32(payload);
        if stored_crc32 != computed_crc32 {
            return Self::new();
        }

        match bincode::serde::decode_from_slice::<SheetCache, _>(payload, bincode::config::standard()) {
            Ok((cache, _)) => cache,
            Err(_) => Self::new(),
        }
    }

    /// 保存缓存到输出目录
    ///
    /// 将当前缓存以二进制格式序列化并写入 .sheet-cache 文件，
    /// 包含魔数头、版本号和 CRC32 校验和
    pub fn save(&self, output_dir: &Path) -> std::io::Result<()> {
        let cache_path = output_dir.join(Self::CACHE_FILE_NAME);

        if let Some(parent) = cache_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let payload = bincode::serde::encode_to_vec(self, bincode::config::standard())
            .map_err(|e| std::io::Error::other(e.to_string()))?;

        let crc32 = compute_crc32(&payload);

        let mut output = Vec::with_capacity(8 + 4 + 4 + payload.len());
        output.extend_from_slice(CACHE_MAGIC);
        output.extend_from_slice(&CACHE_VERSION.to_le_bytes());
        output.extend_from_slice(&crc32.to_le_bytes());
        output.extend_from_slice(&payload);

        std::fs::write(&cache_path, output)
    }

    /// 检查指定文件是否已变更
    ///
    /// 比较当前文件哈希与缓存中的哈希，返回 true 表示文件已变更
    pub fn is_file_changed(&self, path: &Path, current_hash: u64) -> bool {
        match self.file_hashes.get(path) {
            Some(&cached_hash) => cached_hash != current_hash,
            None => true,
        }
    }

    /// 更新文件哈希
    pub fn update_file_hash(&mut self, path: PathBuf, hash: u64) {
        self.file_hashes.insert(path, hash);
    }

    /// 从文本格式反序列化缓存（旧格式兼容）
    ///
    /// 格式：
    /// ```text
    /// [hashes]
    /// path1:hash1
    /// path2:hash2
    /// [dependencies]
    /// table1:dep1,dep2
    /// table2:dep3
    /// ```
    fn deserialize_legacy(content: &str) -> Self {
        let mut cache = Self::new();
        let mut section = "";

        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }

            if trimmed == "[hashes]" {
                section = "hashes";
                continue;
            }
            if trimmed == "[dependencies]" {
                section = "dependencies";
                continue;
            }

            match section {
                "hashes" => {
                    if let Some((path_str, hash_str)) = trimmed.rsplit_once(':') {
                        if let Ok(hash) = hash_str.parse::<u64>() {
                            cache.file_hashes.insert(PathBuf::from(path_str), hash);
                        }
                    }
                }
                "dependencies" => {
                    if let Some((table, deps_str)) = trimmed.split_once(':') {
                        let deps: HashSet<String> = if deps_str.is_empty() {
                            HashSet::new()
                        }
                        else {
                            deps_str.split(',').map(|s| s.to_string()).collect()
                        };
                        if !deps.is_empty() {
                            cache.dependency_graph.dependencies.insert(table.to_string(), deps);
                        }
                    }
                }
                _ => {}
            }
        }

        cache
    }
}

impl Default for SheetCache {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compute_crc32_empty() {
        let crc = compute_crc32(&[]);
        assert_eq!(crc, 0x00000000);
    }

    #[test]
    fn test_compute_crc32_known() {
        let data = b"123456789";
        let crc = compute_crc32(data);
        assert_eq!(crc, 0xCBF43926);
    }

    #[test]
    fn test_compute_crc32_consistency() {
        let data = b"hello world";
        let crc1 = compute_crc32(data);
        let crc2 = compute_crc32(data);
        assert_eq!(crc1, crc2);
    }
}
