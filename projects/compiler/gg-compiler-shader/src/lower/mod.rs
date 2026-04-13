//! gs AST → naga IR 转换器
//!
//! 将 gs 语言的类型化 AST 转换为 naga IR 中间表示，
//! 支持顶点着色器、片段着色器、计算着色器和 uniforms。

mod converter;
mod types;

pub use converter::{DecoratorInfo, GslLowerer};
pub use types::{
    BOOL_SCALAR, EnabledKeywords, F32_SCALAR, GsBindingDecl, GsEntryPoint, GsProperty, GsRenderState, GsUniformField,
    I32_SCALAR, NamedExpressions, TypeLayoutInfo, TypeSizeAlign, U32_SCALAR, align_offset, calc_std140_layout,
    calc_std430_layout,
};

/// 将 NamePath 转换为字符串
pub fn name_path_to_string(np: &oak_valkyrie::ast::NamePath) -> String {
    np.parts.iter().map(|p| p.name.clone()).collect::<Vec<_>>().join("::")
}
