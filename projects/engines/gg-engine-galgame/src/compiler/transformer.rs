//! Galgame 编译转换器模块
//! 实现 gg-compiler 的 Transformer trait，将 .galgame 源码转换为字节码

use gg_compiler_core::{
    artifact::{Artifact, ArtifactKey, ArtifactSet},
    context::BuildContext,
    transformer::Transformer,
};
use gg_core::{GError, GErrorKind, GResult};

use crate::compiler::GalgameCompiler;

/// Galgame 编译转换器
/// 将 galgame_source 产物编译为 galgame_bytecode 产物
pub struct GalgameTransformer;

impl Transformer for GalgameTransformer {
    fn name(&self) -> &str {
        "galgame"
    }

    fn input_keys(&self) -> Vec<ArtifactKey> {
        vec![ArtifactKey::new("galgame_source", "*")]
    }

    fn output_keys(&self) -> Vec<ArtifactKey> {
        vec![ArtifactKey::new("galgame_bytecode", "*")]
    }

    fn transform(&self, inputs: &ArtifactSet, _context: &mut BuildContext) -> GResult<ArtifactSet> {
        let mut output = ArtifactSet::new();
        let compiler = GalgameCompiler::new();

        for key in inputs.keys() {
            if key.type_name != "galgame_source" {
                continue;
            }

            let artifact = inputs.get(&key).ok_or_else(|| GError {
                kind: GErrorKind::Other,
                message: format!("Artifact not found for key: {}/{}", key.type_name, key.id),
            })?;

            let source = std::str::from_utf8(&artifact.data).map_err(|e| GError {
                kind: GErrorKind::Other,
                message: format!("Failed to decode galgame source as UTF-8: {}", e),
            })?;

            let bytecode = compiler.compile(source, &key.id).map_err(|e| GError {
                kind: GErrorKind::Other,
                message: format!("Failed to compile galgame source '{}': {}", key.id, e),
            })?;

            let output_key = ArtifactKey::new("galgame_bytecode", key.id.clone());
            output.insert(Artifact::new(output_key, bytecode));
        }

        Ok(output)
    }
}
