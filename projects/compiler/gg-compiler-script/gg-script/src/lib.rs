#![feature(new_range_api)]
#![warn(missing_docs)]

//! GG 引擎脚本编译模块
//! 使用 oak-valkyrie 前端将 Valkyrie 源码编译为字节码模块

pub mod compiler;
pub mod type_checker;

use std::{collections::HashMap, path::Path};

use gg_bytecode::{format::BytecodeModule, reader::BytecodeReader, writer::BytecodeWriter};
use gg_core::{GError, GErrorKind, GResult};
use gg_ir::{IrModule, TargetPlatform};
use oak_core::{Builder, SourceText};
use oak_valkyrie::{ValkyrieBuilder, ValkyrieLanguage};
use serde::{Deserialize, Serialize};
use sha1::{Digest, Sha1};

use crate::{compiler::ValkyrieCompiler, type_checker::TypeChecker};

/// 编译结果，包含字节码模块和类型诊断信息
pub struct CompilationResult {
    /// 编译生成的字节码模块
    pub bytecode: BytecodeModule,
    /// 类型检查产生的诊断信息
    pub diagnostics: Vec<type_checker::TypeDiagnostic>,
}

/// 类型检查结果，包含类型环境和诊断信息
pub struct TypeCheckResult {
    /// 类型环境，包含所有变量和函数的类型信息
    pub env: type_checker::TypeEnvironment,
    /// 类型检查产生的诊断信息
    pub diagnostics: Vec<type_checker::TypeDiagnostic>,
}

/// 缓存的脚本条目，记录源码哈希与对应的字节码模块
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CachedScriptEntry {
    /// 源码的 SHA1 哈希值
    pub source_hash: String,
    /// 编译生成的字节码模块
    pub bytecode: BytecodeModule,
}

/// 脚本编译缓存，支持基于源码哈希的增量编译
///
/// 维护模块路径到缓存条目的映射，以及模块间的依赖关系图。
/// 当源码未变化时可直接返回缓存的字节码，避免重复编译；
/// 当依赖的模块发生变化时，自动使受影响模块的缓存失效。
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ScriptCache {
    /// 模块路径到缓存条目的映射
    entries: HashMap<String, CachedScriptEntry>,
    /// 模块路径到其依赖模块路径列表的映射
    dependencies: HashMap<String, Vec<String>>,
}

impl ScriptCache {
    /// 创建空的脚本编译缓存
    pub fn new() -> Self {
        Self::default()
    }

    /// 查询缓存中是否存在指定路径且哈希匹配的条目
    pub fn get(&self, path: &str, source_hash: &str) -> Option<&BytecodeModule> {
        self.entries.get(path).and_then(|entry| if entry.source_hash == source_hash { Some(&entry.bytecode) } else { None })
    }

    /// 插入或更新缓存条目，同时记录依赖关系
    pub fn insert(&mut self, path: &str, source_hash: String, bytecode: BytecodeModule, deps: Vec<String>) {
        self.entries.insert(path.to_string(), CachedScriptEntry { source_hash, bytecode });
        self.dependencies.insert(path.to_string(), deps);
    }

    /// 使指定模块及其所有依赖方的缓存失效
    ///
    /// 当模块发生变化时，不仅移除自身的缓存，还会递归移除所有依赖该模块的缓存条目。
    pub fn invalidate(&mut self, path: &str) {
        self.entries.remove(path);
        self.dependencies.remove(path);

        let mut to_invalidate = Vec::new();
        for (module_path, deps) in &self.dependencies {
            if deps.iter().any(|d| d == path) {
                to_invalidate.push(module_path.clone());
            }
        }

        for dep_path in to_invalidate {
            self.invalidate(&dep_path);
        }
    }

    /// 将缓存持久化到磁盘文件
    ///
    /// 使用 bincode 序列化格式将整个缓存写入指定路径。
    pub fn persist_to_disk(&self, path: &Path) -> GResult<()> {
        let data = bincode::serde::encode_to_vec(self, bincode::config::standard())
            .map_err(|e| GError { kind: GErrorKind::Io, message: format!("Failed to serialize script cache: {}", e) })?;
        std::fs::write(path, data)
            .map_err(|e| GError { kind: GErrorKind::Io, message: format!("Failed to write script cache to disk: {}", e) })
    }

    /// 从磁盘文件加载缓存
    ///
    /// 使用 bincode 反序列化从指定路径读取缓存数据。
    pub fn load_from_disk(path: &Path) -> GResult<Self> {
        let data = std::fs::read(path)
            .map_err(|e| GError { kind: GErrorKind::Io, message: format!("Failed to read script cache from disk: {}", e) })?;
        let (cache, _): (ScriptCache, _) = bincode::serde::decode_from_slice(&data, bincode::config::standard())
            .map_err(|e| GError { kind: GErrorKind::Io, message: format!("Failed to deserialize script cache: {}", e) })?;
        Ok(cache)
    }

    /// 返回缓存中的条目数量
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// 返回缓存是否为空
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// 检查指定路径是否在缓存中
    pub fn contains(&self, path: &str) -> bool {
        self.entries.contains_key(path)
    }
}

/// 计算源码的 SHA1 哈希值，返回十六进制字符串
fn compute_source_hash(source: &str) -> String {
    let mut hasher = Sha1::new();
    hasher.update(source.as_bytes());
    let result = hasher.finalize();
    result.iter().map(|b| format!("{:02x}", b)).collect()
}

/// 从源码中提取 `using module_name` 依赖声明
///
/// 扫描源码的每一行，匹配以 `using` 关键字开头的声明语句，
/// 提取模块名称作为依赖项。
fn extract_dependencies(source: &str) -> Vec<String> {
    let mut deps = Vec::new();
    for line in source.lines() {
        let trimmed = line.trim();
        if let Some(rest) = trimmed.strip_prefix("using ") {
            let module_name = rest.trim().trim_end_matches(';').trim();
            if !module_name.is_empty() {
                deps.push(module_name.to_string());
            }
        }
    }
    deps
}

/// 脚本编译器，将 Valkyrie 源码编译为字节码模块
///
/// 编译管线：源码 → ValkyrieBuilder → ValkyrieRoot AST → ValkyrieCompiler → IrModule → 优化 → BytecodeModule
pub struct ScriptCompiler {
    /// 是否启用 IR 优化
    pub optimize: bool,
    /// 目标平台
    pub target_platform: Option<TargetPlatform>,
}

impl ScriptCompiler {
    /// 创建新的脚本编译器（默认启用优化）
    pub fn new() -> Self {
        Self { optimize: true, target_platform: None }
    }

    /// 创建不启用优化的脚本编译器
    pub fn no_optimize() -> Self {
        Self { optimize: false, target_platform: None }
    }

    /// 创建带有目标平台的脚本编译器
    pub fn with_target(target: TargetPlatform) -> Self {
        Self { optimize: true, target_platform: Some(target) }
    }

    /// 编译 Valkyrie 脚本源码为字节码模块
    ///
    /// 通过 ValkyrieBuilder 将源码解析为 AST，再通过 ValkyrieCompiler 编译为 IR，
    /// 可选地执行 IR 优化，最后序列化为 BytecodeModule。
    pub fn compile(&self, source: &str, module_name: &str) -> GResult<BytecodeModule> {
        let mut ir_module = self.compile_to_ir(source, module_name)?;

        if self.optimize {
            let mut optimizer = gg_ir::default_optimizer();
            optimizer.optimize(&mut ir_module)?;
        }

        let bytecode_data = BytecodeWriter::write(&ir_module)?;
        BytecodeReader::read(&bytecode_data)
    }

    /// 增量编译 Valkyrie 脚本源码为字节码模块
    ///
    /// 首先计算源码的 SHA1 哈希值，然后检查缓存中是否存在相同路径和哈希的条目。
    /// 如果缓存命中，直接返回缓存的字节码模块；否则执行完整编译流程，
    /// 并将结果连同依赖关系写入缓存。
    pub fn compile_incremental(&self, source: &str, path: &str, cache: &mut ScriptCache) -> GResult<BytecodeModule> {
        let source_hash = compute_source_hash(source);

        if let Some(bytecode) = cache.get(path, &source_hash) {
            return Ok(bytecode.clone());
        }

        let deps = extract_dependencies(source);
        let bytecode = self.compile(source, path)?;
        cache.insert(path, source_hash, bytecode.clone(), deps);

        Ok(bytecode)
    }

    /// 编译 Valkyrie 脚本源码为字节码模块，同时返回类型诊断信息
    ///
    /// 与 compile 方法相同的编译流程，但额外返回类型检查产生的诊断信息。
    pub fn compile_with_diagnostics(&self, source: &str, module_name: &str) -> GResult<CompilationResult> {
        let (mut ir_module, type_diagnostics) = self.compile_to_ir_with_diagnostics(source, module_name)?;

        if self.optimize {
            let mut optimizer = gg_ir::default_optimizer();
            optimizer.optimize(&mut ir_module)?;
        }

        let bytecode_data = BytecodeWriter::write(&ir_module)?;
        let bytecode = BytecodeReader::read(&bytecode_data)?;

        Ok(CompilationResult { bytecode, diagnostics: type_diagnostics })
    }

    /// 仅执行类型检查，不生成字节码
    ///
    /// 解析源码为 AST 后运行类型检查器，返回类型环境和诊断信息。
    pub fn check_only(&self, source: &str) -> GResult<TypeCheckResult> {
        let language = ValkyrieLanguage::default().with_shader_support();
        let builder = ValkyrieBuilder::new(&language);
        let source_text = SourceText::new(source);
        let mut cache = oak_core::parser::ParseSession::<ValkyrieLanguage>::default();
        let diagnostics = builder.build(&source_text, &[], &mut cache);

        match diagnostics.result {
            Ok(root) => {
                let mut type_checker = TypeChecker::new();
                let type_diagnostics = type_checker.check_root(&root);
                let env = type_checker.env.clone();
                Ok(TypeCheckResult { env, diagnostics: type_diagnostics })
            }
            Err(e) => Err(GError { kind: GErrorKind::Runtime, message: format!("Valkyrie parse error: {}", e) }),
        }
    }

    /// 编译 Valkyrie 脚本源码为 IR 模块（跳过优化和字节码序列化）
    ///
    /// 解析源码为 AST 后，先运行类型检查器生成诊断信息（仅警告，不阻止编译），
    /// 再通过 ValkyrieCompiler 编译为 IR 模块。
    pub fn compile_to_ir(&self, source: &str, module_name: &str) -> GResult<IrModule> {
        let (ir_module, _diagnostics) = self.compile_to_ir_with_diagnostics(source, module_name)?;
        Ok(ir_module)
    }

    /// 编译 Valkyrie 脚本源码为 IR 模块，同时返回类型诊断信息
    ///
    /// 解析源码为 AST 后，先运行类型检查器生成诊断信息，
    /// 再通过 ValkyrieCompiler 编译为 IR 模块，返回 IR 模块和类型诊断信息。
    pub fn compile_to_ir_with_diagnostics(
        &self,
        source: &str,
        module_name: &str,
    ) -> GResult<(IrModule, Vec<type_checker::TypeDiagnostic>)> {
        let language = ValkyrieLanguage::default().with_shader_support();
        let builder = ValkyrieBuilder::new(&language);
        let source_text = SourceText::new(source);
        let mut cache = oak_core::parser::ParseSession::<ValkyrieLanguage>::default();
        let diagnostics = builder.build(&source_text, &[], &mut cache);

        match diagnostics.result {
            Ok(root) => {
                let mut type_checker = TypeChecker::new();
                let type_diagnostics = type_checker.check_root(&root);
                for diag in &type_diagnostics {
                    eprintln!("[type-check] {}", diag);
                }

                let compiler = match self.target_platform {
                    Some(target) => ValkyrieCompiler::with_target(module_name, target),
                    None => ValkyrieCompiler::new(module_name),
                };
                let ir_module = compiler.compile(&root, module_name)?;
                Ok((ir_module, type_diagnostics))
            }
            Err(e) => Err(GError { kind: GErrorKind::Runtime, message: format!("Valkyrie parse error: {}", e) }),
        }
    }
}

impl Default for ScriptCompiler {
    fn default() -> Self {
        Self::new()
    }
}

/// 脚本加载器，从文件系统加载脚本并编译为字节码模块
pub struct ScriptLoader {
    /// 脚本编译器
    compiler: ScriptCompiler,
}

impl ScriptLoader {
    /// 创建新的脚本加载器
    pub fn new() -> Self {
        Self { compiler: ScriptCompiler::new() }
    }

    /// 从文件加载脚本并编译为字节码模块
    pub fn load_file(&self, path: &Path) -> GResult<BytecodeModule> {
        let source = std::fs::read_to_string(path)
            .map_err(|e| GError { kind: GErrorKind::Io, message: format!("Failed to read script file: {}", e) })?;

        let module_name = path.file_stem().and_then(|s| s.to_str()).unwrap_or("unknown");

        self.compiler.compile(&source, module_name)
    }

    /// 从字符串加载脚本并编译为字节码模块
    pub fn load_string(&self, source: &str, module_name: &str) -> GResult<BytecodeModule> {
        self.compiler.compile(source, module_name)
    }

    /// 从文件加载脚本并编译为 IR 模块
    pub fn load_file_as_ir(&self, path: &Path) -> GResult<IrModule> {
        let source = std::fs::read_to_string(path)
            .map_err(|e| GError { kind: GErrorKind::Io, message: format!("Failed to read script file: {}", e) })?;

        let module_name = path.file_stem().and_then(|s| s.to_str()).unwrap_or("unknown");

        self.compiler.compile_to_ir(&source, module_name)
    }
}

impl Default for ScriptLoader {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn simple_valkyrie_source() -> &'static str {
        "micro main() { let x = 1; }"
    }

    fn modified_valkyrie_source() -> &'static str {
        "micro main() { let x = 2; }"
    }

    fn source_with_dependency() -> &'static str {
        "using math\nmicro main() { let x = 1; }"
    }

    fn source_with_multiple_dependencies() -> &'static str {
        "using math\nusing physics\nmicro main() { let x = 1; }"
    }

    #[test]
    fn test_compute_source_hash_deterministic() {
        let hash1 = compute_source_hash("hello world");
        let hash2 = compute_source_hash("hello world");
        assert_eq!(hash1, hash2);
    }

    #[test]
    fn test_compute_source_hash_different_sources() {
        let hash1 = compute_source_hash("hello");
        let hash2 = compute_source_hash("world");
        assert_ne!(hash1, hash2);
    }

    #[test]
    fn test_extract_dependencies_no_deps() {
        let deps = extract_dependencies("micro main() { let x = 1; }");
        assert!(deps.is_empty());
    }

    #[test]
    fn test_extract_dependencies_single() {
        let deps = extract_dependencies("using math;\nmicro main() { let x = 1; }");
        assert_eq!(deps, vec!["math"]);
    }

    #[test]
    fn test_extract_dependencies_multiple() {
        let deps = extract_dependencies("using math;\nusing physics;\nmicro main() { let x = 1; }");
        assert_eq!(deps, vec!["math", "physics"]);
    }

    #[test]
    fn test_extract_dependencies_with_whitespace() {
        let deps = extract_dependencies("  using   utils  ;  \nmicro main() {}");
        assert_eq!(deps, vec!["utils"]);
    }

    #[test]
    fn test_cache_hit() {
        let compiler = ScriptCompiler::no_optimize();
        let mut cache = ScriptCache::new();

        let source = simple_valkyrie_source();
        let result1 = compiler.compile_incremental(source, "test_module", &mut cache).unwrap();
        let result2 = compiler.compile_incremental(source, "test_module", &mut cache).unwrap();

        assert_eq!(result1.name, result2.name);
        assert_eq!(cache.len(), 1);
    }

    #[test]
    fn test_cache_miss_different_source() {
        let compiler = ScriptCompiler::no_optimize();
        let mut cache = ScriptCache::new();

        let source1 = simple_valkyrie_source();
        let source2 = modified_valkyrie_source();

        let result1 = compiler.compile_incremental(source1, "test_module", &mut cache).unwrap();
        assert_eq!(cache.len(), 1);

        let result2 = compiler.compile_incremental(source2, "test_module", &mut cache).unwrap();
        assert_eq!(cache.len(), 1);

        assert_eq!(result1.name, result2.name);
    }

    #[test]
    fn test_cache_invalidation() {
        let mut cache = ScriptCache::new();

        let hash1 = compute_source_hash("source1");
        let hash2 = compute_source_hash("source2");

        let compiler = ScriptCompiler::no_optimize();
        let bytecode1 = compiler.compile("micro main() { let x = 1; }", "mod_a").unwrap();
        let bytecode2 = compiler.compile("micro main() { let x = 2; }", "mod_b").unwrap();

        cache.insert("mod_a", hash1, bytecode1, vec!["mod_b".to_string()]);
        cache.insert("mod_b", hash2, bytecode2, vec![]);

        assert!(cache.contains("mod_a"));
        assert!(cache.contains("mod_b"));

        cache.invalidate("mod_b");

        assert!(!cache.contains("mod_b"));
        assert!(!cache.contains("mod_a"));
    }

    #[test]
    fn test_cache_invalidation_no_dependents() {
        let mut cache = ScriptCache::new();

        let hash = compute_source_hash("source");
        let compiler = ScriptCompiler::no_optimize();
        let bytecode = compiler.compile("micro main() { let x = 1; }", "mod_a").unwrap();

        cache.insert("mod_a", hash, bytecode, vec![]);

        assert!(cache.contains("mod_a"));

        cache.invalidate("mod_a");

        assert!(!cache.contains("mod_a"));
    }

    #[test]
    fn test_cache_persistence() {
        let dir = tempfile::tempdir().unwrap();
        let cache_path = dir.path().join("cache.bin");

        let mut cache = ScriptCache::new();
        let hash = compute_source_hash("source");
        let compiler = ScriptCompiler::no_optimize();
        let bytecode = compiler.compile("micro main() { let x = 1; }", "test_mod").unwrap();

        cache.insert("test_mod", hash, bytecode.clone(), vec!["dep1".to_string()]);

        cache.persist_to_disk(&cache_path).unwrap();

        let loaded = ScriptCache::load_from_disk(&cache_path).unwrap();

        assert_eq!(loaded.len(), 1);
        assert!(loaded.contains("test_mod"));

        let loaded_entry = loaded.entries.get("test_mod").unwrap();
        assert_eq!(loaded_entry.source_hash, hash);
        assert_eq!(loaded_entry.bytecode.name, bytecode.name);

        let loaded_deps = loaded.dependencies.get("test_mod").unwrap();
        assert_eq!(loaded_deps, &vec!["dep1".to_string()]);
    }

    #[test]
    fn test_cache_persistence_roundtrip() {
        let dir = tempfile::tempdir().unwrap();
        let cache_path = dir.path().join("cache_roundtrip.bin");

        let mut cache = ScriptCache::new();
        let compiler = ScriptCompiler::no_optimize();

        let source = simple_valkyrie_source();
        let result = compiler.compile_incremental(source, "test_mod", &mut cache).unwrap();

        cache.persist_to_disk(&cache_path).unwrap();

        let loaded = ScriptCache::load_from_disk(&cache_path).unwrap();

        let loaded_entry = loaded.entries.get("test_mod").unwrap();
        assert_eq!(loaded_entry.bytecode.name, result.name);
    }

    #[test]
    fn test_cache_load_missing_file() {
        let result = ScriptCache::load_from_disk(Path::new("/nonexistent/path/cache.bin"));
        assert!(result.is_err());
    }

    #[test]
    fn test_dependency_tracking() {
        let compiler = ScriptCompiler::no_optimize();
        let mut cache = ScriptCache::new();

        let source = source_with_dependency();
        compiler.compile_incremental(source, "dependent_mod", &mut cache).unwrap();

        let deps = cache.dependencies.get("dependent_mod").unwrap();
        assert_eq!(deps, &vec!["math"]);
    }

    #[test]
    fn test_dependency_tracking_multiple() {
        let compiler = ScriptCompiler::no_optimize();
        let mut cache = ScriptCache::new();

        let source = source_with_multiple_dependencies();
        compiler.compile_incremental(source, "dependent_mod", &mut cache).unwrap();

        let deps = cache.dependencies.get("dependent_mod").unwrap();
        assert_eq!(deps, &vec!["math", "physics"]);
    }

    #[test]
    fn test_cascade_invalidation() {
        let mut cache = ScriptCache::new();

        let compiler = ScriptCompiler::no_optimize();
        let bytecode_a = compiler.compile("micro main() { let x = 1; }", "mod_a").unwrap();
        let bytecode_b = compiler.compile("micro main() { let x = 2; }", "mod_b").unwrap();
        let bytecode_c = compiler.compile("micro main() { let x = 3; }", "mod_c").unwrap();

        let hash_a = compute_source_hash("a");
        let hash_b = compute_source_hash("b");
        let hash_c = compute_source_hash("c");

        cache.insert("mod_a", hash_a, bytecode_a, vec!["mod_b".to_string()]);
        cache.insert("mod_b", hash_b, bytecode_b, vec!["mod_c".to_string()]);
        cache.insert("mod_c", hash_c, bytecode_c, vec![]);

        assert_eq!(cache.len(), 3);

        cache.invalidate("mod_c");

        assert!(!cache.contains("mod_c"));
        assert!(!cache.contains("mod_b"));
        assert!(!cache.contains("mod_a"));
        assert_eq!(cache.len(), 0);
    }
}
