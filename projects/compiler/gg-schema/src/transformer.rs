//! Schema 转换器模块
//! 实现 Transformer trait，将 Schema 编译器集成到编译流水线中

use gg_compiler::{
    artifact::{Artifact, ArtifactKey, ArtifactSet},
    context::BuildContext,
    transformer::Transformer,
};
use gg_core::GResult;

use crate::compiler::SchemaCompiler;

/// Schema 转换器，将 schema_source 产物转换为 schema_ir、schema_bindings、schema_migrations 产物
pub struct SchemaTransformer;

impl Transformer for SchemaTransformer {
    fn name(&self) -> &str {
        "schema"
    }

    fn input_keys(&self) -> Vec<ArtifactKey> {
        vec![ArtifactKey::new("schema_source", "*")]
    }

    fn output_keys(&self) -> Vec<ArtifactKey> {
        vec![
            ArtifactKey::new("schema_ir", "*"),
            ArtifactKey::new("schema_bindings", "*"),
            ArtifactKey::new("schema_migrations", "*"),
        ]
    }

    fn transform(&self, inputs: &ArtifactSet, context: &mut BuildContext) -> GResult<ArtifactSet> {
        let compiler = SchemaCompiler::new();
        let mut outputs = ArtifactSet::new();

        let schema_keys: Vec<ArtifactKey> = inputs.keys().filter(|k| k.type_name == "schema_source").cloned().collect();

        for key in schema_keys {
            let artifact = inputs.get(&key).unwrap();
            let source = String::from_utf8(artifact.data.clone())
                .map_err(|e| gg_core::GError::new(&format!("Schema 源文件 UTF-8 解码失败: {e}")))?;

            match compiler.compile(&source, &key.id) {
                Ok(output) => {
                    let ir_key = ArtifactKey::new("schema_ir", &key.id);
                    outputs.insert(Artifact::new(ir_key, output.ir_data));

                    let bindings_key = ArtifactKey::new("schema_bindings", &key.id);
                    outputs.insert(Artifact::new(bindings_key, output.bindings.into_bytes()));

                    let migrations_data = serde_json::to_vec(&output.migrations)
                        .map_err(|e| gg_core::GError::new(&format!("迁移文件序列化失败: {e}")))?;
                    let migrations_key = ArtifactKey::new("schema_migrations", &key.id);
                    outputs.insert(Artifact::new(migrations_key, migrations_data));
                }
                Err(e) => {
                    context.add_diagnostic(gg_compiler::context::DiagnosticLevel::Error, "schema", &e.to_string());
                }
            }
        }

        Ok(outputs)
    }
}
