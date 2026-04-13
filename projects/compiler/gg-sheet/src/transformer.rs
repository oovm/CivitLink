//! 配置表编译器 Transformer 集成模块
//! 将 SheetCompiler 适配为 gg-compiler 的 Transformer trait 实现

use std::path::PathBuf;

use gg_compiler::{
    artifact::{Artifact, ArtifactKey, ArtifactSet},
    context::{BuildContext, DiagnosticLevel},
    transformer::Transformer,
};
use gg_core::GResult;

use crate::compiler::SheetCompiler;

/// 配置表编译器 Transformer
///
/// 将 SheetCompiler 适配为 gg-compiler 的 Transformer trait 实现，
/// 使其可以接入编译流水线。
pub struct SheetTransformer {
    /// 配置表目录
    sheet_dir: PathBuf,
    /// 输出目录
    output_dir: PathBuf,
}

impl SheetTransformer {
    /// 创建新的配置表 Transformer
    pub fn new(sheet_dir: impl Into<PathBuf>, output_dir: impl Into<PathBuf>) -> Self {
        Self { sheet_dir: sheet_dir.into(), output_dir: output_dir.into() }
    }
}

impl Transformer for SheetTransformer {
    fn name(&self) -> &str {
        "sheet"
    }

    fn input_keys(&self) -> Vec<ArtifactKey> {
        vec![ArtifactKey::new("sheet_source", "*")]
    }

    fn output_keys(&self) -> Vec<ArtifactKey> {
        vec![ArtifactKey::new("sheet_script", "*"), ArtifactKey::new("sheet_von", "*")]
    }

    fn transform(&self, _inputs: &ArtifactSet, context: &mut BuildContext) -> GResult<ArtifactSet> {
        let mut compiler = SheetCompiler::new(&self.sheet_dir, &self.output_dir);

        let files = compiler.scan()?;

        if files.is_empty() {
            context.add_diagnostic(
                DiagnosticLevel::Warning,
                "sheet",
                &format!("配置表目录 '{}' 中没有找到配置表文件", self.sheet_dir.display()),
            );
            return Ok(ArtifactSet::new());
        }

        compiler.compile()?;

        let mut output = ArtifactSet::new();

        for path in &files {
            let table_name = path.file_stem().and_then(|s| s.to_str()).unwrap_or("unknown");

            let script_path = self.output_dir.join(format!("{}Table.script", table_name));
            if script_path.exists() {
                let data = std::fs::read(&script_path)?;
                output.insert(Artifact::new(ArtifactKey::new("sheet_script", table_name), data));
            }

            let von_path = self.output_dir.join(format!("{}Table.von", table_name));
            if von_path.exists() {
                let data = std::fs::read(&von_path)?;
                output.insert(Artifact::new(ArtifactKey::new("sheet_von", table_name), data));
            }
        }

        context.add_diagnostic(DiagnosticLevel::Info, "sheet", &format!("成功编译 {} 个配置表文件", files.len()));

        Ok(output)
    }
}
