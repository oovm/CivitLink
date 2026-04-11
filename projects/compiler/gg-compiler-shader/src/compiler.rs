//! GG Shader 编译器公共 API
//!
//! 提供从 gs 源码到 naga IR 的完整编译管线，
//! 以及 naga IR 的序列化/反序列化和验证功能。

use std::hash::{Hash, Hasher};

use gg_core::{GError, GErrorKind, GResult};
use naga;
use rustc_hash::{FxHashMap, FxHasher};

use oak_core::{Builder, SourceText, parser::ParseSession};
use oak_valkyrie::{ValkyrieBuilder, ValkyrieLanguage};

use crate::{
    artifact::{ShaderArtifact, ShaderModuleEntry},
    lower::GslLowerer,
};

/// GG Shader 编译器
///
/// 提供 gs 源码到 naga IR 的完整编译管线。
/// 支持编译、序列化、反序列化和验证操作。
/// 内置编译缓存，相同源码不会重复编译。
pub struct ShaderCompiler {
    /// 是否启用优化
    pub optimize: bool,
    /// 编译缓存（源码哈希 → 编译产物）
    cache: FxHashMap<u64, ShaderArtifact>,
}

impl ShaderCompiler {
    /// 创建新的编译器（默认启用优化）
    pub fn new() -> Self {
        Self { optimize: true, cache: FxHashMap::default() }
    }

    /// 创建不启用优化的编译器
    pub fn no_optimize() -> Self {
        Self { optimize: false, cache: FxHashMap::default() }
    }

    /// 计算源码的哈希值
    fn source_hash(source: &str) -> u64 {
        let mut hasher = FxHasher::default();
        source.hash(&mut hasher);
        hasher.finish()
    }

    /// 编译 gs 源码为着色器产物
    ///
    /// 解析 gs 源码，将其转换为 naga IR 中间表示，
    /// 同时提取渲染状态和回退信息。
    /// 编译源码中的所有着色器块，忽略命名空间和 micro 函数。
    /// 相同源码会命中缓存，直接返回之前的编译结果。
    pub fn compile(&mut self, source: &str) -> GResult<ShaderArtifact> {
        let hash = Self::source_hash(source);
        if let Some(cached) = self.cache.get(&hash) {
            return Ok(cached.clone());
        }

        let language = ValkyrieLanguage::default().with_shader_support();
        let builder = ValkyrieBuilder::new(&language);
        let source_text = SourceText::new(source);
        let mut cache = ParseSession::<ValkyrieLanguage>::default();
        let diagnostics = builder.build(&source_text, &[], &mut cache);

        let root = diagnostics
            .result
            .map_err(|e| GError { kind: GErrorKind::Other, message: format!("解析 shader 源码失败: {:?}", e) })?;

        let entries: Vec<ShaderModuleEntry> = root
            .items
            .iter()
            .filter_map(|item| match item {
                oak_valkyrie::ast::StatementNode::Shader(shader) => {
                    let mut lowerer = GslLowerer::new();
                    match lowerer.lower(&*shader) {
                        Ok((module, render_states, fallback)) => {
                            if let Err(e) = self.validate(&module) {
                                return Some(Err(e));
                            }
                            let name = shader.name.name.clone();
                            let kind = shader.kind.name.clone();
                            Some(Ok(ShaderModuleEntry { name, kind, module, render_states, fallback }))
                        }
                        Err(e) => Some(Err(e)),
                    }
                }
                _ => None,
            })
            .collect::<GResult<Vec<_>>>()?;

        let artifact = ShaderArtifact { modules: entries };
        self.cache.insert(hash, artifact.clone());
        Ok(artifact)
    }

    /// 编译 gs 源码为序列化的二进制数据
    ///
    /// 等价于 `compile()` 后调用 `ShaderArtifact::serialize()`。
    pub fn compile_to_bytes(&mut self, source: &str) -> GResult<Vec<u8>> {
        let artifact = self.compile(source)?;
        artifact.serialize()
    }

    /// 编译 gs 源码中的所有着色器变体
    pub fn compile_with_variants(&mut self, source: &str) -> GResult<crate::variant::ShaderVariantArtifact> {
        let variant_compiler = crate::variant::VariantCompiler { optimize: self.optimize };
        variant_compiler.compile_variants(source)
    }

    /// 从序列化的二进制数据加载着色器产物
    pub fn load_from_bytes(&self, bytes: &[u8]) -> GResult<ShaderArtifact> {
        let artifact = ShaderArtifact::deserialize(bytes)?;
        for entry in &artifact.modules {
            self.validate(&entry.module)?;
        }
        Ok(artifact)
    }

    /// 编译 gs 源码中的第一个着色器块
    pub fn compile_first(&mut self, source: &str) -> GResult<ShaderModuleEntry> {
        self.compile(source)?
            .into_first_module()
            .ok_or_else(|| GError { kind: GErrorKind::Other, message: "gs 源码中未找到着色器块".to_string() })
    }

    /// 验证 naga Module 的正确性
    pub fn validate(&self, module: &naga::Module) -> GResult<()> {
        let mut validator = naga::valid::Validator::new(naga::valid::ValidationFlags::all(), naga::valid::Capabilities::all());
        validator
            .validate(module)
            .map_err(|e| GError { kind: GErrorKind::Other, message: format!("naga Module 验证失败: {:?}", e) })?;
        Ok(())
    }

    /// 清除编译缓存
    pub fn clear_cache(&mut self) {
        self.cache.clear();
    }
}

impl Default for ShaderCompiler {
    fn default() -> Self {
        Self::new()
    }
}
