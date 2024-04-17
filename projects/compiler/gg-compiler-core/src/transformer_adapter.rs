//! IR 优化转换器适配器模块
//! 将 IrOptimizer 适配为 Transformer trait，使其可嵌入编译流水线

use gg_core::{GError, GErrorKind, GResult};
use gg_ir::{IrModule, pass::IrOptimizer};

use crate::{
    artifact::{Artifact, ArtifactKey, ArtifactSet},
    context::BuildContext,
    transformer::Transformer,
};

/// IR 优化转换器，将 IrOptimizer 适配为 Transformer
pub struct IrOptimizeTransformer {
    /// 转换器名称
    transformer_name: String,
    /// IR 优化器
    optimizer: IrOptimizer,
}

impl IrOptimizeTransformer {
    /// 创建新的 IR 优化转换器
    pub fn new(name: &str, optimizer: IrOptimizer) -> Self {
        Self { transformer_name: name.to_string(), optimizer }
    }
}

/// IR 模块在 ArtifactSet 中的产物类型名称
const IR_MODULE_TYPE: &str = "ir_module";

impl Transformer for IrOptimizeTransformer {
    fn name(&self) -> &str {
        &self.transformer_name
    }

    fn input_keys(&self) -> Vec<ArtifactKey> {
        vec![ArtifactKey::new(IR_MODULE_TYPE, "*")]
    }

    fn output_keys(&self) -> Vec<ArtifactKey> {
        vec![ArtifactKey::new(IR_MODULE_TYPE, "*")]
    }

    fn transform(&self, inputs: &ArtifactSet, _context: &mut BuildContext) -> GResult<ArtifactSet> {
        let mut output = ArtifactSet::new();

        for key in inputs.keys() {
            if key.type_name != IR_MODULE_TYPE {
                continue;
            }

            let artifact = inputs.get(&key).ok_or_else(|| GError {
                kind: GErrorKind::Other,
                message: format!("Artifact not found for key: {}/{}", key.type_name, key.id),
            })?;

            let module: IrModule = serde_json::from_slice(&artifact.data)
                .map_err(|e| GError { kind: GErrorKind::Other, message: format!("Failed to deserialize IrModule: {}", e) })?;

            let mut module = module;
            self.optimizer.optimize(&mut module)?;

            let data = serde_json::to_vec(&module)
                .map_err(|e| GError { kind: GErrorKind::Other, message: format!("Failed to serialize IrModule: {}", e) })?;

            output.insert(Artifact::new(key.clone(), data));
        }

        Ok(output)
    }
}
