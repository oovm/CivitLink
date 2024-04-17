//! Valkyrie 脚本转换器模块
//! 将 .valkyrie 脚本编译为字节码模块

use gg_bytecode::BytecodeWriter;
use gg_compiler_core::{
    artifact::{Artifact, ArtifactKey, ArtifactSet},
    context::{BuildContext, DiagnosticLevel},
    transformer::Transformer,
};
use gg_core::GResult;
use gg_script::ScriptCompiler;

/// Valkyrie 脚本源码产物类型名称
const VALKYRIE_SOURCE_TYPE: &str = "valkyrie_source";
/// 字节码模块产物类型名称
const BYTECODE_MODULE_TYPE: &str = "bytecode_module";

/// Valkyrie 脚本转换器
/// 将 .valkyrie 脚本源码编译为字节码模块
pub struct ValkyrieScriptTransformer {
    /// 是否启用 IR 优化
    pub optimize: bool,
}

impl ValkyrieScriptTransformer {
    /// 创建新的 Valkyrie 脚本转换器（默认启用优化）
    pub fn new() -> Self {
        Self { optimize: true }
    }

    /// 创建不启用优化的 Valkyrie 脚本转换器
    pub fn no_optimize() -> Self {
        Self { optimize: false }
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
        let compiler = if self.optimize { ScriptCompiler::new() } else { ScriptCompiler::no_optimize() };

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
            match compiler.compile_to_ir(&source, module_name) {
                Ok(ir_module) => match BytecodeWriter::write(&ir_module) {
                    Ok(bytecode_data) => {
                        let output_key = ArtifactKey::new(BYTECODE_MODULE_TYPE, &key.id);
                        output.insert(Artifact::new(output_key, bytecode_data));
                    }
                    Err(e) => {
                        context.add_diagnostic(
                            DiagnosticLevel::Error,
                            self.name(),
                            &format!("Failed to serialize bytecode for '{}': {}", key.id, e),
                        );
                        continue;
                    }
                },
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
