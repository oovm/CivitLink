//! 元编译器转换器模块
//! 实现 gg-core 编译器转换器概念，将 .gscript 文件转换为 JSON 输出

use std::path::{Path, PathBuf};

use gg_core::{GError, GErrorKind, GResult};

use crate::compiler::ScriptCompiler;

/// 剧本转换器，将 .gscript 文件编译并序列化为 JSON
///
/// 读取 .gscript 文件，编译为 StorySequence，
/// 序列化为 JSON 写入输出目录，返回输出文件路径列表。
pub struct ScriptTransformer;

impl ScriptTransformer {
    /// 执行转换
    ///
    /// 读取 input_path 指向的 .gscript 文件或目录，
    /// 编译为 StorySequence，序列化为 JSON 写入 output_dir，
    /// 返回所有输出文件路径列表。
    pub fn transform(
        input_path: &Path,
        output_dir: &Path,
    ) -> GResult<Vec<PathBuf>> {
        if input_path.is_dir() {
            Self::transform_directory(input_path, output_dir)
        } else {
            Self::transform_file(input_path, output_dir)
        }
    }

    /// 转换单个 .gscript 文件
    fn transform_file(
        input_path: &Path,
        output_dir: &Path,
    ) -> GResult<Vec<PathBuf>> {
        let sequence = ScriptCompiler::compile_file(input_path)?;

        std::fs::create_dir_all(output_dir).map_err(|e| GError {
            kind: GErrorKind::Io,
            message: format!(
                "Failed to create output directory '{}': {}",
                output_dir.display(),
                e
            ),
        })?;

        let stem = input_path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("output");

        let output_path = output_dir.join(format!("{}.json", stem));

        let json = serde_json::to_string_pretty(&sequence).map_err(|e| GError {
            kind: GErrorKind::Other,
            message: format!("Failed to serialize sequence to JSON: {}", e),
        })?;

        std::fs::write(&output_path, json).map_err(|e| GError {
            kind: GErrorKind::Io,
            message: format!(
                "Failed to write output file '{}': {}",
                output_path.display(),
                e
            ),
        })?;

        Ok(vec![output_path])
    }

    /// 转换目录下所有 .gscript 文件
    fn transform_directory(
        input_dir: &Path,
        output_dir: &Path,
    ) -> GResult<Vec<PathBuf>> {
        let db = ScriptCompiler::compile_directory(input_dir)?;

        std::fs::create_dir_all(output_dir).map_err(|e| GError {
            kind: GErrorKind::Io,
            message: format!(
                "Failed to create output directory '{}': {}",
                output_dir.display(),
                e
            ),
        })?;

        let mut output_paths = Vec::new();

        for (file_name, sequence) in &db.sequences {
            let output_path = output_dir.join(format!("{}.json", file_name));

            let json = serde_json::to_string_pretty(sequence).map_err(|e| GError {
                kind: GErrorKind::Other,
                message: format!("Failed to serialize sequence '{}' to JSON: {}", file_name, e),
            })?;

            std::fs::write(&output_path, json).map_err(|e| GError {
                kind: GErrorKind::Io,
                message: format!(
                    "Failed to write output file '{}': {}",
                    output_path.display(),
                    e
                ),
            })?;

            output_paths.push(output_path);
        }

        Ok(output_paths)
    }
}
