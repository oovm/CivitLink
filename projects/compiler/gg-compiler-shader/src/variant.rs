#![warn(missing_docs)]

//! Shader 变体系统模块
//!
//! 提供着色器变体的定义、收集和编译功能，
//! 支持通过 `@variant` 注解声明条件关键字，
//! 并生成所有关键字组合的编译产物。

use gg_core::{GError, GErrorKind, GResult};
use naga;
use oak_core::{Builder, SourceText, parser::ParseSession};
use oak_valkyrie::{
    ValkyrieBuilder, ValkyrieLanguage,
    ast::{ShaderDeclaration, StatementNode, TermExpression},
};
use rustc_hash::FxHashMap;

use crate::{
    artifact::{ShaderArtifact, ShaderModuleEntry},
    lower::{EnabledKeywords, GslLowerer},
};

/// 单个着色器变体定义
///
/// 表示一个通过 `@variant` 注解声明的着色器变体，
/// 包含变体名称和条件关键字列表。
#[derive(Debug, Clone)]
pub struct ShaderVariant {
    /// 变体名称
    pub name: String,
    /// 条件关键字列表
    pub keywords: Vec<String>,
    /// 启用的特性列表
    pub enabled_features: Vec<String>,
}

/// 变体集合
///
/// 从着色器声明中收集的所有变体定义，
/// 支持生成所有关键字组合。
#[derive(Debug, Clone)]
pub struct VariantCollection {
    /// 收集到的变体列表
    pub variants: Vec<ShaderVariant>,
}

impl VariantCollection {
    /// 创建空的变体集合
    pub fn new() -> Self {
        Self { variants: Vec::new() }
    }

    /// 从 shader 声明的 annotations 中收集 `@variant` 注解
    ///
    /// 遍历 `shader.annotations`，找到 name 为 "variant" 的 Attribute，
    /// 从其 args 中提取字符串参数作为关键字。
    /// 每个 `@variant` 注解创建一个 `ShaderVariant`，
    /// name 从第一个参数提取，keywords 包含所有参数。
    pub fn collect_from_shader(shader: &ShaderDeclaration) -> Self {
        let mut variants = Vec::new();

        for attr in &shader.annotations {
            if attr.name.name.to_lowercase() == "variant" {
                let keywords: Vec<String> = attr.args.iter().filter_map(|arg| expr_to_string(arg)).collect();

                if keywords.is_empty() {
                    continue;
                }

                let name = keywords[0].clone();
                variants.push(ShaderVariant { name, keywords, enabled_features: Vec::new() });
            }
        }

        Self { variants }
    }

    /// 生成所有关键字组合
    ///
    /// 如果有 N 个变体，每个变体有一个关键字，
    /// 则生成 2^N 种组合（包括空组合）。
    /// 例如，2 个变体关键字 "A" 和 "B" 生成:
    /// `[[], ["A"], ["B"], ["A", "B"]]`
    pub fn generate_combinations(&self) -> Vec<Vec<String>> {
        let keywords: Vec<&String> = self.variants.iter().filter_map(|v| v.keywords.first()).collect();

        let n = keywords.len();
        if n == 0 {
            return vec![vec![]];
        }

        let total = 1usize << n;
        let mut combinations = Vec::with_capacity(total);

        for mask in 0..total {
            let mut combo = Vec::new();
            for i in 0..n {
                if mask & (1 << i) != 0 {
                    combo.push(keywords[i].clone());
                }
            }
            combinations.push(combo);
        }

        combinations
    }
}

impl Default for VariantCollection {
    fn default() -> Self {
        Self::new()
    }
}

/// 着色器变体编译产物
///
/// 包含基础着色器产物和所有变体组合对应的编译产物。
#[derive(Debug)]
pub struct ShaderVariantArtifact {
    /// 基础着色器产物
    pub base: ShaderArtifact,
    /// 变体组合及其对应的编译产物
    pub variants: Vec<(Vec<String>, ShaderArtifact)>,
}

/// 变体编译器
///
/// 编译着色器源码中的所有变体，
/// 为每个关键字组合生成对应的编译产物。
pub struct VariantCompiler {
    /// 是否启用优化
    pub optimize: bool,
}

impl VariantCompiler {
    /// 创建新的变体编译器
    pub fn new() -> Self {
        Self { optimize: true }
    }

    /// 编译源码中的所有变体
    ///
    /// 步骤：
    /// 1. 使用 oak-valkyrie 解析源码
    /// 2. 找到所有 shader 块
    /// 3. 对每个 shader 块收集变体
    /// 4. 生成所有关键字组合
    /// 5. 对每个组合，使用对应的关键字集合编译 shader（支持条件编译）
    /// 6. 返回 ShaderVariantArtifact
    pub fn compile_variants(&self, source: &str) -> GResult<ShaderVariantArtifact> {
        let language = ValkyrieLanguage::default().with_shader_support();
        let builder = ValkyrieBuilder::new(&language);
        let source_text = SourceText::new(source);
        let mut cache = ParseSession::<ValkyrieLanguage>::default();
        let diagnostics = builder.build(&source_text, &[], &mut cache);

        let root = diagnostics
            .result
            .map_err(|e| GError { kind: GErrorKind::Other, message: format!("解析 gs 源码失败: {:?}", e) })?;

        let shader = root
            .items
            .iter()
            .find_map(|item| match item {
                StatementNode::Shader(shader) => Some(shader),
                _ => None,
            })
            .ok_or_else(|| GError { kind: GErrorKind::Other, message: "gs 源码中未找到着色器块".to_string() })?;

        let collection = VariantCollection::collect_from_shader(shader);

        let mut lowerer = GslLowerer::new();
        let (module, render_states, fallback) = lowerer.lower(shader, &FxHashMap::default())?;

        self.validate_module(&module)?;

        let name = shader.name.name.clone();
        let kind = shader.kind.name.clone();

        let base = ShaderArtifact {
            modules: vec![ShaderModuleEntry {
                name: name.clone(),
                kind: kind.clone(),
                module,
                render_states: render_states.clone(),
                fallback: fallback.clone(),
            }],
        };

        let combinations = collection.generate_combinations();
        let mut variant_artifacts = Vec::new();

        for combo in &combinations {
            if combo.is_empty() {
                continue;
            }

            let keywords: EnabledKeywords = combo.iter().cloned().collect();
            let mut combo_lowerer = GslLowerer::with_keywords(keywords);
            let (combo_module, combo_render_states, combo_fallback) = combo_lowerer.lower(shader, &FxHashMap::default())?;

            self.validate_module(&combo_module).map_err(|e| GError {
                kind: GErrorKind::Other,
                message: format!("变体 {:?} naga Module 验证失败: {}", combo, e.message),
            })?;

            let artifact = ShaderArtifact {
                modules: vec![ShaderModuleEntry {
                    name: name.clone(),
                    kind: kind.clone(),
                    module: combo_module,
                    render_states: combo_render_states,
                    fallback: combo_fallback,
                }],
            };

            variant_artifacts.push((combo.clone(), artifact));
        }

        Ok(ShaderVariantArtifact { base, variants: variant_artifacts })
    }

    /// 验证 naga Module 的正确性
    fn validate_module(&self, module: &naga::Module) -> GResult<()> {
        if !self.optimize {
            return Ok(());
        }

        let mut validator = naga::valid::Validator::new(naga::valid::ValidationFlags::all(), naga::valid::Capabilities::all());
        validator
            .validate(module)
            .map_err(|e| GError { kind: GErrorKind::Other, message: format!("naga Module 验证失败: {:?}", e) })?;
        Ok(())
    }
}

impl Default for VariantCompiler {
    fn default() -> Self {
        Self::new()
    }
}

/// 将 TermExpression 转换为字符串
fn expr_to_string(expr: &TermExpression) -> Option<String> {
    match expr {
        TermExpression::NamePath(np) => {
            if np.parts.len() == 1 {
                Some(np.parts[0].name.clone())
            }
            else {
                Some(np.parts.iter().map(|p| p.name.clone()).collect::<Vec<_>>().join("::"))
            }
        }
        TermExpression::StringLiteral(sl) => Some(
            sl.segments
                .iter()
                .filter_map(|seg| match seg {
                    oak_valkyrie::ast::StringSegment::Text(text_seg) => Some(text_seg.content.as_str()),
                    _ => None,
                })
                .collect(),
        ),
        _ => None,
    }
}
