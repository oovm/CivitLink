//! 增量编译模块
//! 提供基于文件哈希和依赖感知的增量编译功能，只重新编译发生变化的文件

use std::{
    collections::{HashMap, HashSet},
    path::Path,
};

use gg_core::{GError, GErrorKind, GResult};
use serde::{Deserialize, Serialize};

use crate::compiler::{
    GalgameCompiler,
    ir::{DialogueDB, StorySequence},
};

/// 文件编译缓存条目
#[derive(Debug, Clone, Serialize, Deserialize)]
struct CacheEntry {
    /// 文件内容哈希
    hash: u64,
    /// 该文件依赖的其他文件路径
    dependencies: HashSet<String>,
}

/// 增量编译缓存
#[derive(Debug, Clone, Serialize, Deserialize)]
struct IncrementalCache {
    /// 文件路径到缓存条目的映射
    entries: HashMap<String, CacheEntry>,
}

impl IncrementalCache {
    /// 创建空的增量编译缓存
    fn new() -> Self {
        Self { entries: HashMap::new() }
    }
}

/// 增量编译器，通过缓存文件哈希避免重复编译
///
/// 维护已编译序列的缓存和文件哈希映射，
/// 只在文件内容发生变化时才重新编译。
/// 支持依赖感知重编译和缓存持久化。
pub struct IncrementalCompiler {
    /// 文件路径到内容哈希的映射
    pub file_hashes: HashMap<String, u64>,
    /// 已编译的序列缓存
    pub compiled_sequences: HashMap<String, StorySequence>,
    /// 文件依赖关系映射
    dependencies: HashMap<String, HashSet<String>>,
    /// 缓存目录路径
    cache_dir: Option<std::path::PathBuf>,
}

impl IncrementalCompiler {
    /// 创建新的增量编译器
    pub fn new() -> Self {
        Self { file_hashes: HashMap::new(), compiled_sequences: HashMap::new(), dependencies: HashMap::new(), cache_dir: None }
    }

    /// 设置缓存目录
    ///
    /// 编译产物和哈希缓存将持久化到该目录。
    pub fn with_cache_dir(mut self, dir: impl Into<std::path::PathBuf>) -> Self {
        self.cache_dir = Some(dir.into());
        self
    }

    /// 增量编译目录
    ///
    /// 只重新编译哈希变化的文件，返回完整的 DialogueDB
    /// （包含未变化的缓存结果 + 新编译的结果）。
    /// 支持依赖感知：当被依赖文件变更时，依赖方也会被重新编译。
    pub fn compile_directory(&mut self, dir: &Path) -> GResult<DialogueDB> {
        self.load_cache_from_disk();

        let mut db = DialogueDB::new();

        let entries = std::fs::read_dir(dir).map_err(|e| GError {
            kind: GErrorKind::Io,
            message: format!("Failed to read directory '{}': {}", dir.display(), e),
        })?;

        let mut current_files: HashMap<String, std::path::PathBuf> = HashMap::new();

        for entry in entries {
            let entry = entry
                .map_err(|e| GError { kind: GErrorKind::Io, message: format!("Failed to read directory entry: {}", e) })?;

            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) == Some("galgame") {
                let file_name = path.file_stem().and_then(|s| s.to_str()).unwrap_or("unknown").to_string();
                current_files.insert(file_name, path);
            }
        }

        let mut stale_keys: Vec<String> = Vec::new();
        for key in self.compiled_sequences.keys() {
            if !current_files.contains_key(key) {
                stale_keys.push(key.clone());
            }
        }
        for key in &stale_keys {
            self.compiled_sequences.remove(key);
            self.file_hashes.remove(key);
            self.dependencies.remove(key);
        }

        let mut changed_files: HashSet<String> = HashSet::new();

        for (file_name, path) in &current_files {
            let content = std::fs::read_to_string(path).map_err(|e| GError {
                kind: GErrorKind::Io,
                message: format!("Failed to read file '{}': {}", path.display(), e),
            })?;

            let hash = Self::compute_hash(&content);

            let needs_recompile = match self.file_hashes.get(file_name) {
                Some(&old_hash) => old_hash != hash,
                None => true,
            };

            if needs_recompile {
                changed_files.insert(file_name.clone());
                let compiler = GalgameCompiler::new();
                let ir = compiler.parse(&content)?;
                let sequence = StorySequence { nodes: ir.dialogues };

                let mut deps = HashSet::new();
                for import in &ir.imports {
                    deps.insert(import.clone());
                }
                self.dependencies.insert(file_name.clone(), deps);

                self.compiled_sequences.insert(file_name.clone(), sequence);
                self.file_hashes.insert(file_name.clone(), hash);
            }
        }

        let dependents = self.find_dependents(&changed_files);
        for dep_name in &dependents {
            if let Some(path) = current_files.get(dep_name) {
                let content = std::fs::read_to_string(path).map_err(|e| GError {
                    kind: GErrorKind::Io,
                    message: format!("Failed to read file '{}': {}", path.display(), e),
                })?;

                let compiler = GalgameCompiler::new();
                let ir = compiler.parse(&content)?;
                let sequence = StorySequence { nodes: ir.dialogues };
                let hash = Self::compute_hash(&content);

                self.compiled_sequences.insert(dep_name.clone(), sequence);
                self.file_hashes.insert(dep_name.clone(), hash);
            }
        }

        for (file_name, sequence) in &self.compiled_sequences {
            for node in &sequence.nodes {
                db.all_node_ids.insert(node.id.clone());
            }
            db.sequences.insert(file_name.clone(), sequence.clone());
        }

        self.save_cache_to_disk();

        Ok(db)
    }

    /// 查找依赖于指定文件的文件集合
    fn find_dependents(&self, changed_files: &HashSet<String>) -> HashSet<String> {
        let mut dependents = HashSet::new();
        for (file_name, deps) in &self.dependencies {
            for dep in deps {
                if changed_files.contains(dep) {
                    dependents.insert(file_name.clone());
                }
            }
        }
        dependents
    }

    /// 使指定文件的缓存失效
    ///
    /// 下次编译时将重新编译该文件。
    pub fn invalidate(&mut self, path: &str) {
        self.file_hashes.remove(path);
        self.compiled_sequences.remove(path);
    }

    /// 使所有缓存失效
    ///
    /// 下次编译时将重新编译所有文件。
    pub fn invalidate_all(&mut self) {
        self.file_hashes.clear();
        self.compiled_sequences.clear();
        self.dependencies.clear();
    }

    /// 从磁盘加载缓存
    fn load_cache_from_disk(&mut self) {
        if let Some(ref cache_dir) = self.cache_dir {
            let cache_path = cache_dir.join("galgame_incremental_cache.json");
            if cache_path.exists() {
                if let Ok(content) = std::fs::read_to_string(&cache_path) {
                    if let Ok(cache) = serde_json::from_str::<IncrementalCache>(&content) {
                        for (name, entry) in cache.entries {
                            self.file_hashes.insert(name.clone(), entry.hash);
                            self.dependencies.insert(name, entry.dependencies);
                        }
                    }
                }
            }
        }
    }

    /// 保存缓存到磁盘
    fn save_cache_to_disk(&self) {
        if let Some(ref cache_dir) = self.cache_dir {
            let _ = std::fs::create_dir_all(cache_dir);

            let mut cache = IncrementalCache::new();
            for (name, &hash) in &self.file_hashes {
                let deps = self.dependencies.get(name).cloned().unwrap_or_default();
                cache.entries.insert(name.clone(), CacheEntry { hash, dependencies: deps });
            }

            let cache_path = cache_dir.join("galgame_incremental_cache.json");
            if let Ok(content) = serde_json::to_string_pretty(&cache) {
                let _ = std::fs::write(cache_path, content);
            }
        }
    }

    /// 计算字符串内容的 FNV-1a 哈希值
    fn compute_hash(content: &str) -> u64 {
        let mut hash: u64 = 0xcbf29ce484222325;
        for byte in content.bytes() {
            hash ^= byte as u64;
            hash = hash.wrapping_mul(0x100000001b3);
        }
        hash
    }
}

impl Default for IncrementalCompiler {
    fn default() -> Self {
        Self::new()
    }
}
