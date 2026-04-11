//! Widget 编译器主入口模块

use gg_bytecode::BytecodeWriter;
use gg_compiler::{
    artifact::{Artifact, ArtifactKey, ArtifactSet},
    context::{BuildContext, DiagnosticLevel},
    transformer::Transformer,
};
use gg_core::GResult;
use gg_script::ScriptCompiler;
use oak_voc::ast::VxDocument;

use crate::{
    artifact::WidgetArtifact,
    error::WidgetError,
    parser::WidgetParser,
    registry::ComponentRegistry,
    style::{codegen::StyleCodegen, scss::ScssProcessor},
    template::{codegen::TemplateCodegen, validator::TemplateValidator},
};

/// Widget 源码产物类型名称
const WIDGET_SOURCE_TYPE: &str = "widget_source";
/// Widget 产物类型名称
const WIDGET_ARTIFACT_TYPE: &str = "widget_artifact";
/// 模板产物类型名称
const TEMPLATE_BUNDLE_TYPE: &str = "template_bundle";
/// 样式产物类型名称
const STYLE_BUNDLE_TYPE: &str = "style_bundle";

/// Widget 编译器
pub struct WidgetCompiler {
    /// 组件注册表
    registry: ComponentRegistry,
    /// 是否启用 IR 优化
    pub optimize: bool,
}

impl WidgetCompiler {
    /// 创建新的 Widget 编译器
    pub fn new() -> Self {
        Self { registry: ComponentRegistry::with_builtins(), optimize: true }
    }

    /// 创建不启用优化的 Widget 编译器
    pub fn no_optimize() -> Self {
        Self { registry: ComponentRegistry::with_builtins(), optimize: false }
    }

    /// 编译 .widget 文件内容
    pub fn compile(&mut self, source: &str, module_name: &str) -> Result<WidgetArtifact, WidgetError> {
        let widget_file = WidgetParser::parse(source)?;

        let mut template_validator = TemplateValidator::new(&self.registry);
        let template_ir = widget_file.template.as_ref().map(|t| template_validator.validate(t)).transpose()?;

        let style_ir = widget_file
            .style
            .as_ref()
            .map(|s| {
                let mut processor = ScssProcessor::new();
                processor.process(s)
            })
            .transpose()?;

        let script_bytecode = widget_file
            .script
            .as_ref()
            .map(|s| {
                let compiler = if self.optimize { ScriptCompiler::new() } else { ScriptCompiler::no_optimize() };
                compiler
                    .compile_to_ir(s, module_name)
                    .map_err(|e| WidgetError::SemanticError(format!("Script compilation failed: {}", e)))
                    .and_then(|ir| {
                        BytecodeWriter::write(&ir)
                            .map_err(|e| WidgetError::CodegenError(format!("Bytecode generation failed: {}", e)))
                    })
            })
            .transpose()?;

        let template_bundle = template_ir.as_ref().map(|ir| TemplateCodegen::generate(ir)).transpose()?;

        let style_bundle = style_ir.as_ref().map(|ir| StyleCodegen::generate(ir)).transpose()?;

        let meta = crate::artifact::WidgetMeta {
            name: module_name.to_string(),
            version: "0.1.0".to_string(),
            export_name: None,
            compiled_at: std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_secs(),
            compiler_version: env!("CARGO_PKG_VERSION").to_string(),
        };

        Ok(WidgetArtifact {
            meta,
            template: template_bundle.unwrap_or_else(|| crate::artifact::TemplateBundle {
                node_tree: vec![],
                component_refs: std::collections::HashMap::new(),
                bindings: vec![],
                event_handlers: vec![],
                resource_refs: vec![],
            }),
            script: script_bytecode,
            style: style_bundle,
            dependencies: vec![],
            resources: vec![],
        })
    }

    /// 从预解析的 AST 编译 .widget 文件
    pub fn compile_from_asts(
        &mut self,
        template_ast: Option<&VxDocument>,
        style_ast: Option<&VxDocument>,
        script_source: Option<&str>,
        module_name: &str,
    ) -> Result<WidgetArtifact, WidgetError> {
        let mut template_validator = TemplateValidator::new(&self.registry);
        let template_ir = template_ast.map(|ast| template_validator.validate_from_ast(ast)).transpose()?;

        let mut scss_processor = ScssProcessor::new();
        let style_ir = style_ast.map(|ast| scss_processor.process_from_ast(ast)).transpose()?;

        let script_bytecode = script_source
            .map(|s| {
                let compiler = if self.optimize { ScriptCompiler::new() } else { ScriptCompiler::no_optimize() };
                compiler
                    .compile_to_ir(s, module_name)
                    .map_err(|e| WidgetError::SemanticError(format!("Script compilation failed: {}", e)))
                    .and_then(|ir| {
                        BytecodeWriter::write(&ir)
                            .map_err(|e| WidgetError::CodegenError(format!("Bytecode generation failed: {}", e)))
                    })
            })
            .transpose()?;

        let template_bundle = template_ir.as_ref().map(|ir| TemplateCodegen::generate(ir)).transpose()?;

        let style_bundle = style_ir.as_ref().map(|ir| StyleCodegen::generate(ir)).transpose()?;

        let meta = crate::artifact::WidgetMeta {
            name: module_name.to_string(),
            version: "0.1.0".to_string(),
            export_name: None,
            compiled_at: std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_secs(),
            compiler_version: env!("CARGO_PKG_VERSION").to_string(),
        };

        Ok(WidgetArtifact {
            meta,
            template: template_bundle.unwrap_or_else(|| crate::artifact::TemplateBundle {
                node_tree: vec![],
                component_refs: std::collections::HashMap::new(),
                bindings: vec![],
                event_handlers: vec![],
                resource_refs: vec![],
            }),
            script: script_bytecode,
            style: style_bundle,
            dependencies: vec![],
            resources: vec![],
        })
    }
}

impl Default for WidgetCompiler {
    fn default() -> Self {
        Self::new()
    }
}

/// Widget 转换器
pub struct WidgetTransformer {
    /// 是否启用 IR 优化
    pub optimize: bool,
}

impl WidgetTransformer {
    /// 创建新的 Widget 转换器（默认启用优化）
    pub fn new() -> Self {
        Self { optimize: true }
    }

    /// 创建不启用优化的 Widget 转换器
    pub fn no_optimize() -> Self {
        Self { optimize: false }
    }
}

impl Default for WidgetTransformer {
    fn default() -> Self {
        Self::new()
    }
}

impl Transformer for WidgetTransformer {
    fn name(&self) -> &str {
        "widget"
    }

    fn input_keys(&self) -> Vec<ArtifactKey> {
        vec![ArtifactKey::new(WIDGET_SOURCE_TYPE, "*")]
    }

    fn output_keys(&self) -> Vec<ArtifactKey> {
        vec![
            ArtifactKey::new(WIDGET_ARTIFACT_TYPE, "*"),
            ArtifactKey::new(TEMPLATE_BUNDLE_TYPE, "*"),
            ArtifactKey::new(STYLE_BUNDLE_TYPE, "*"),
        ]
    }

    fn transform(&self, inputs: &ArtifactSet, context: &mut BuildContext) -> GResult<ArtifactSet> {
        let mut output = ArtifactSet::new();
        let mut compiler = WidgetCompiler::new();

        for key in inputs.keys() {
            if key.type_name != WIDGET_SOURCE_TYPE {
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

            match compiler.compile(&source, &key.id) {
                Ok(widget_artifact) => {
                    match widget_artifact.serialize() {
                        Ok(data) => {
                            let artifact_key = ArtifactKey::new(WIDGET_ARTIFACT_TYPE, &key.id);
                            output.insert(Artifact::new(artifact_key, data));
                        }
                        Err(e) => {
                            context.add_diagnostic(
                                DiagnosticLevel::Error,
                                self.name(),
                                &format!("Failed to serialize widget artifact for '{}': {:?}", key.id, e),
                            );
                        }
                    }

                    let template_key = ArtifactKey::new(TEMPLATE_BUNDLE_TYPE, &key.id);
                    output.insert(Artifact::new(template_key, widget_artifact.template.node_tree.clone()));

                    if let Some(ref style_bundle) = widget_artifact.style {
                        let style_key = ArtifactKey::new(STYLE_BUNDLE_TYPE, &key.id);
                        match serde_json::to_vec(style_bundle) {
                            Ok(data) => output.insert(Artifact::new(style_key, data)),
                            Err(e) => {
                                context.add_diagnostic(
                                    DiagnosticLevel::Error,
                                    self.name(),
                                    &format!("Failed to serialize style bundle for '{}': {}", key.id, e),
                                );
                                continue;
                            }
                        };
                    }
                }
                Err(e) => {
                    context.add_diagnostic(
                        DiagnosticLevel::Error,
                        self.name(),
                        &format!("Failed to compile widget '{}': {:?}", key.id, e),
                    );
                }
            }
        }

        Ok(output)
    }
}
