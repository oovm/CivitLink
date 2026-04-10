//! 增量编译模块
//! 提供基于文件哈希的增量编译功能，只重新编译发生变化的文件

use std::{collections::HashMap, path::Path};

use gg_core::{GError, GErrorKind, GResult};

use crate::compiler::{DialogueDB, ScriptCompiler, StorySequence};

/// 增量编译器，通过缓存文件哈希避免重复编译
///
/// 维护已编译序列的缓存和文件哈希映射，
/// 只在文件内容发生变化时才重新编译。
pub struct IncrementalCompiler {
    /// 文件路径到内容哈希的映射
    pub file_hashes: HashMap<String, u64>,
    /// 已编译的序列缓存
    pub compiled_sequences: HashMap<String, StorySequence>,
}

impl IncrementalCompiler {
    /// 创建新的增量编译器
    pub fn new() -> Self {
        Self { file_hashes: HashMap::new(), compiled_sequences: HashMap::new() }
    }

    /// 增量编译目录
    ///
    /// 只重新编译哈希变化的文件，返回完整的 DialogueDB
    /// （包含未变化的缓存结果 + 新编译的结果）。
    pub fn compile_directory(&mut self, dir: &Path) -> GResult<DialogueDB> {
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
            if path.extension().and_then(|e| e.to_str()) == Some("gscript") {
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
        for key in stale_keys {
            self.compiled_sequences.remove(&key);
            self.file_hashes.remove(&key);
        }

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
                let sequence = ScriptCompiler::compile_file(path)?;
                self.compiled_sequences.insert(file_name.clone(), sequence);
                self.file_hashes.insert(file_name.clone(), hash);
            }
        }

        for (file_name, sequence) in &self.compiled_sequences {
            for node in &sequence.nodes {
                db.all_node_ids.insert(node.id.clone());
            }
            db.sequences.insert(file_name.clone(), sequence.clone());
        }

        Ok(db)
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
    }

    /// 计算字符串内容的简单哈希值
    fn compute_hash(content: &str) -> u64 {
        let mut hash: u64 = 5381;
        for byte in content.bytes() {
            hash = hash.wrapping_mul(33).wrapping_add(byte as u64);
        }
        hash
    }
}

impl Default for IncrementalCompiler {
    fn default() -> Self {
        Self::new()
    }
}
