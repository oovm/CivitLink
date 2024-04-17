//! Valkyrie 脚本转换器模块
//! 将 .valkyrie 脚本编译为字节码模块

use std::cell::RefCell;

use gg_bytecode::BytecodeWriter;
use gg_compiler::{
    artifact::{Artifact, ArtifactKey, ArtifactSet},
    context::{BuildContext, DiagnosticLevel},
    transformer::Transformer,
};
use gg_core::GResult;
use gg_ir::TargetPlatform;
use gg_script::{ScriptCache, ScriptCompiler, type_checker::DiagnosticSeverity};

/// Valkyrie 脚本源码产物类型名称
const VALKYRIE_SOURCE_TYPE: &str = "valkyrie_source";
/// 字节码模块产物类型名称
const BYTECODE_MODULE_TYPE: &str = "bytecode_module";

/// Valkyrie 脚本转换器
///
/// 将 .valkyrie 脚本源码编译为字节码模块，支持基于源码哈希的增量编译。
/// 内部维护编译缓存，当源码未变化时直接返回缓存的字节码，避免重复编译。
pub struct ValkyrieScriptTransformer {
    /// 是否启用 IR 优化
    pub optimize: bool,
    /// 目标平台
    pub target_platform: Option<TargetPlatform>,
    /// 脚本编译缓存，用于增量编译
    cache: RefCell<ScriptCache>,
}

impl ValkyrieScriptTransformer {
    /// 创建新的 Valkyrie 脚本转换器（默认启用优化）
    pub fn new() -> Self {
        Self { optimize: true, target_platform: None, cache: RefCell::new(ScriptCache::new()) }
    }

    /// 创建不启用优化的 Valkyrie 脚本转换器
    pub fn no_optimize() -> Self {
        Self { optimize: false, target_platform: None, cache: RefCell::new(ScriptCache::new()) }
    }

    /// 创建带有目标平台的 Valkyrie 脚本转换器
    pub fn with_target(target: TargetPlatform) -> Self {
        Self { optimize: true, target_platform: Some(target), cache: RefCell::new(ScriptCache::new()) }
    }

    /// 获取编译缓存的可变引用
    pub fn cache_mut(&mut self) -> &mut ScriptCache {
        self.cache.get_mut()
    }

    /// 从磁盘加载编译缓存
    pub fn load_cache(&mut self, path: &std::path::Path) -> GResult<()> {
        *self.cache.get_mut() = ScriptCache::load_from_disk(path)?;
        Ok(())
    }

    /// 将编译缓存持久化到磁盘
    pub fn persist_cache(&self, path: &std::path::Path) -> GResult<()> {
        self.cache.borrow().persist_to_disk(path)
    }
}

impl Default for ValkyrieScriptTransformer {
    fn default() -> Self {
        Self::new()
    }
}

impl Transformer for ValkyrieScriptTransformer {
    fn name(&self) -> &str {
        "valkyrie_script"
    }

    fn input_keys(&self) -> Vec<ArtifactKey> {
        vec![ArtifactKey::new(VALKYRIE_SOURCE_TYPE, "*")]
    }

    fn output_keys(&self) -> Vec<ArtifactKey> {
        vec![ArtifactKey::new(BYTECODE_MODULE_TYPE, "*")]
    }

    fn transform(&self, inputs: &ArtifactSet, context: &mut BuildContext) -> GResult<ArtifactSet> {
        let mut output = ArtifactSet::new();
        let compiler = match (self.optimize, self.target_platform) {
            (true, Some(target)) => ScriptCompiler::with_target(target),
            (true, None) => ScriptCompiler::new(),
            (false, Some(target)) => {
                let mut c = ScriptCompiler::with_target(target);
                c.optimize = false;
                c
            }
            (false, None) => ScriptCompiler::no_optimize(),
        };

        for key in inputs.keys() {
            if key.type_name != VALKYRIE_SOURCE_TYPE {
                continue;
            }

            let artifact = match inputs.get(&key) {
                Some(a) => a,
                None => {
                    context.add_diagnostic(
                        DiagnosticLevel::Warning,
                        self.name(),
                        &format!("Artifact not found for key: {}/{}", key.type_name, key.id),
                    );
                    continue;
                }
            };

            let source = match String::from_utf8(artifact.data.clone()) {
                Ok(s) => s,
                Err(e) => {
                    context.add_diagnostic(
                        DiagnosticLevel::Error,
                        self.name(),
                        &format!("Failed to decode source as UTF-8 for '{}': {}", key.id, e),
                    );
                    continue;
                }
            };

            let module_name = &key.id;
            let mut cache = self.cache.borrow_mut();
            match compiler.compile_incremental(&source, module_name, &mut cache) {
                Ok(bytecode_module) => {
                    let bytecode_data = BytecodeWriter::write_module(&bytecode_module);
                    match bytecode_data {
                        Ok(data) => {
                            let output_key = ArtifactKey::new(BYTECODE_MODULE_TYPE, &key.id);
                            output.insert(Artifact::new(output_key, data));
                        }
                        Err(e) => {
                            context.add_diagnostic(
                                DiagnosticLevel::Error,
                                self.name(),
                                &format!("Failed to serialize bytecode for '{}': {}", key.id, e),
                            );
                            continue;
                        }
                    }
                }
                Err(e) => {
                    context.add_diagnostic(
                        DiagnosticLevel::Error,
                        self.name(),
                        &format!("Failed to compile '{}': {}", key.id, e),
                    );
                    continue;
                }
            }
        }

        Ok(output)
    }
}
