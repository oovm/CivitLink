//! .vx 文件编译模块
//! 将 .vx 单文件组件编译为字节码产物

use gg_bytecode::BytecodeWriter;
use gg_compiler_core::{
    artifact::{Artifact, ArtifactKey, ArtifactSet},
    context::{BuildContext, DiagnosticLevel},
    transformer::Transformer,
};
use gg_core::{GError, GErrorKind, GResult};
use gg_script::ScriptCompiler;

/// .vx 源码产物类型名称
const VX_SOURCE_TYPE: &str = "vx_source";
/// 字节码模块产物类型名称
const BYTECODE_MODULE_TYPE: &str = "bytecode_module";
/// 模板产物类型名称
const VX_TEMPLATE_TYPE: &str = "vx_template";
/// 样式产物类型名称
const VX_STYLE_TYPE: &str = "vx_style";

/// .vx 文件解析结果
pub struct VxFile {
    /// 模板部分内容
    pub template: Option<String>,
    /// 脚本部分内容（Valkyrie 代码）
    pub script: Option<String>,
    /// 样式部分内容
    pub style: Option<String>,
}

/// .vx 文件解析器
/// 将 .vx 文件内容可靠地解析为 VxFile 结构
pub struct VxParser;

impl VxParser {
    /// 解析 .vx 文件内容为 VxFile 结构
    pub fn parse(content: &str) -> GResult<VxFile> {
        let template = Self::extract_section(content, "template")?;
        let script = Self::extract_section(content, "script")?;
        let style = Self::extract_section(content, "style")?;

        if template.is_none() && script.is_none() && style.is_none() {
            return Err(GError {
                kind: GErrorKind::Other,
                message: "No valid section found in .vx file".to_string(),
            });
        }

        Ok(VxFile {
            template,
            script,
            style,
        })
    }

    /// 从内容中提取指定标签的区块内容
    fn extract_section(content: &str, tag_name: &str) -> GResult<Option<String>> {
        let opening_tag = format!("<{}>", tag_name);
        let closing_tag = format!("</{}>", tag_name);
        let tag_prefix = format!("<{}", tag_name);

        let open_pos = match content.find(&opening_tag) {
            Some(pos) => pos,
            None => return Ok(None),
        };

        let content_start = open_pos + opening_tag.len();
        let mut depth: usize = 1;
        let mut search_from = content_start;

        while depth > 0 {
            let next_open = Self::find_tag_open(content, &tag_prefix, search_from);
            let next_close = content[search_from..]
                .find(&closing_tag)
                .map(|p| search_from + p);

            match (next_open, next_close) {
                (Some(op), Some(cp)) => {
                    if op < cp {
                        depth += 1;
                        search_from = op + tag_prefix.len();
                    } else {
                        depth -= 1;
                        if depth == 0 {
                            return Ok(Some(content[content_start..cp].to_string()));
                        }
                        search_from = cp + closing_tag.len();
                    }
                }
                (None, Some(cp)) => {
                    depth -= 1;
                    if depth == 0 {
                        return Ok(Some(content[content_start..cp].to_string()));
                    }
                    search_from = cp + closing_tag.len();
                }
                (Some(_), None) | (None, None) => {
                    return Err(GError {
                        kind: GErrorKind::Other,
                        message: format!("Unclosed <{}> tag", tag_name),
                    });
                }
            }
        }

        Ok(None)
    }

    /// 从指定位置开始查找标签前缀的起始位置
    fn find_tag_open(content: &str, tag_prefix: &str, from: usize) -> Option<usize> {
        let mut search_from = from;
        while let Some(pos) = content[search_from..].find(tag_prefix) {
            let abs_pos = search_from + pos;
            let after_start = abs_pos + tag_prefix.len();
            if after_start >= content.len() {
                return None;
            }
            let next_char = content.as_bytes()[after_start];
            if matches!(next_char, b'>' | b' ' | b'\t' | b'\n' | b'\r' | b'/') {
                return Some(abs_pos);
            }
            search_from = abs_pos + tag_prefix.len();
        }
        None
    }
}

/// .vx 文件转换器
/// 将 .vx 文件编译为字节码产物
pub struct VxTransformer {
    /// 是否启用 IR 优化
    pub optimize: bool,
}

impl VxTransformer {
    /// 创建新的 .vx 文件转换器（默认启用优化）
    pub fn new() -> Self {
        Self { optimize: true }
    }

    /// 创建不启用优化的 .vx 文件转换器
    pub fn no_optimize() -> Self {
        Self { optimize: false }
    }
}

impl Default for VxTransformer {
    fn default() -> Self {
        Self::new()
    }
}

impl Transformer for VxTransformer {
    fn name(&self) -> &str {
        "vx"
    }

    fn input_keys(&self) -> Vec<ArtifactKey> {
        vec![ArtifactKey::new(VX_SOURCE_TYPE, "*")]
    }

    fn output_keys(&self) -> Vec<ArtifactKey> {
        vec![
            ArtifactKey::new(BYTECODE_MODULE_TYPE, "*"),
            ArtifactKey::new(VX_TEMPLATE_TYPE, "*"),
            ArtifactKey::new(VX_STYLE_TYPE, "*"),
        ]
    }

    fn transform(&self, inputs: &ArtifactSet, context: &mut BuildContext) -> GResult<ArtifactSet> {
        let mut output = ArtifactSet::new();
        let compiler = if self.optimize {
            ScriptCompiler::new()
        } else {
            ScriptCompiler::no_optimize()
        };

        for key in inputs.keys() {
            if key.type_name != VX_SOURCE_TYPE {
                continue;
            }

            let artifact = match inputs.get(&key) {
                Some(a) => a,
                None => {
                    context.add_diagnostic(
                        DiagnosticLevel::Warning,
                        self.name(),
                        &format!(
                            "Artifact not found for key: {}/{}",
                            key.type_name, key.id
                        ),
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
                        &format!(
                            "Failed to decode source as UTF-8 for '{}': {}",
                            key.id, e
                        ),
                    );
                    continue;
                }
            };

            let vx_file = match VxParser::parse(&source) {
                Ok(f) => f,
                Err(e) => {
                    context.add_diagnostic(
                        DiagnosticLevel::Error,
                        self.name(),
                        &format!("Failed to parse .vx file '{}': {}", key.id, e),
                    );
                    continue;
                }
            };

            if let Some(ref script_content) = vx_file.script {
                let module_name = &key.id;
                match compiler.compile_to_ir(script_content, module_name) {
                    Ok(ir_module) => match BytecodeWriter::write(&ir_module) {
                        Ok(bytecode_data) => {
                            let output_key =
                                ArtifactKey::new(BYTECODE_MODULE_TYPE, &key.id);
                            output.insert(Artifact::new(output_key, bytecode_data));
                        }
                        Err(e) => {
                            context.add_diagnostic(
                                DiagnosticLevel::Error,
                                self.name(),
                                &format!(
                                    "Failed to serialize bytecode for '{}': {}",
                                    key.id, e
                                ),
                            );
                        }
                    },
                    Err(e) => {
                        context.add_diagnostic(
                            DiagnosticLevel::Error,
                            self.name(),
                            &format!(
                                "Failed to compile script in '{}': {}",
                                key.id, e
                            ),
                        );
                    }
                }
            }

            if let Some(ref template_content) = vx_file.template {
                let output_key = ArtifactKey::new(VX_TEMPLATE_TYPE, &key.id);
                output.insert(Artifact::new(
                    output_key,
                    template_content.as_bytes().to_vec(),
                ));
            }

            if let Some(ref style_content) = vx_file.style {
                let output_key = ArtifactKey::new(VX_STYLE_TYPE, &key.id);
                output.insert(Artifact::new(
                    output_key,
                    style_content.as_bytes().to_vec(),
                ));
            }
        }

        Ok(output)
    }
}
