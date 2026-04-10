//! GG Shader 编译器公共 API
//!
//! 提供从 gs 源码到 naga IR 的完整编译管线，
//! 以及 naga IR 的序列化/反序列化和验证功能。

use gg_core::{GError, GErrorKind, GResult};
use naga;

use crate::lower::GslLowerer;
use crate::parser::GslParser;
use crate::serialize;

/// GG Shader 编译器
///
/// 提供 gs 源码到 naga IR 的完整编译管线。
/// 支持编译、序列化、反序列化和验证操作。
pub struct GgShaderCompiler {
    /// 是否启用优化
    pub optimize: bool,
}

impl GgShaderCompiler {
    /// 创建新的编译器（默认启用优化）
    pub fn new() -> Self {
        Self { optimize: true }
    }

    /// 创建不启用优化的编译器
    pub fn no_optimize() -> Self {
        Self { optimize: false }
    }

    /// 编译 gs 源码为 naga Module
    ///
    /// 解析 gs 源码，将其转换为 naga IR 中间表示。
    /// 仅编译第一个着色器块，忽略命名空间和 micro 函数。
    pub fn compile(&self, source: &str) -> GResult<naga::Module> {
        let shader_file = GslParser::parse(source)?;

        let shader = shader_file.shaders.first().ok_or_else(|| GError {
            kind: GErrorKind::Other,
            message: "gs 源码中未找到着色器块".to_string(),
        })?;

        let mut lowerer = GslLowerer::new();
        let module = lowerer.lower(shader)?;

        self.validate(&module)?;

        Ok(module)
    }

    /// 编译 gs 源码为序列化的二进制数据
    ///
    /// 等价于 `compile()` 后调用 `serialize_module()`。
    pub fn compile_to_bytes(&self, source: &str) -> GResult<Vec<u8>> {
        let module = self.compile(source)?;
        serialize::serialize_module(&module)
    }

    /// 从序列化的二进制数据加载 naga Module
    pub fn load_from_bytes(&self, bytes: &[u8]) -> GResult<naga::Module> {
        let module = serialize::deserialize_module(bytes)?;
        self.validate(&module)?;
        Ok(module)
    }

    /// 验证 naga Module 的正确性
    pub fn validate(&self, module: &naga::Module) -> GResult<()> {
        let mut validator = naga::valid::Validator::new(
            naga::valid::ValidationFlags::all(),
            naga::valid::Capabilities::all(),
        );
        validator.validate(module).map_err(|e| GError {
            kind: GErrorKind::Other,
            message: format!("naga Module 验证失败: {:?}", e),
        })?;
        Ok(())
    }
}

impl Default for GgShaderCompiler {
    fn default() -> Self {
        Self::new()
    }
}
