//! 内置着色器加载模块
//!
//! 通过 GG Shader 编译器编译内置 .shader 文件，
//! 返回可用于创建渲染管线的 naga IR Module。

use crate::compiler::ShaderCompiler;
use gg_core::GResult;

/// 编译内置精灵着色器
pub fn load_sprite_shader() -> GResult<naga::Module> {
    let source = include_str!("sprite.shader");
    let mut compiler = ShaderCompiler::new();
    let entry = compiler.compile_first(source)?;
    Ok(entry.module)
}

/// 编译内置批渲染精灵着色器
pub fn load_batch_sprite_shader() -> GResult<naga::Module> {
    let source = include_str!("batch_sprite.shader");
    let mut compiler = ShaderCompiler::new();
    let entry = compiler.compile_first(source)?;
    Ok(entry.module)
}

/// 编译内置过渡着色器
pub fn load_transition_shader() -> GResult<naga::Module> {
    let source = include_str!("transition.shader");
    let mut compiler = ShaderCompiler::new();
    let entry = compiler.compile_first(source)?;
    Ok(entry.module)
}

/// 编译内置圆角矩形着色器
pub fn load_rounded_rect_shader() -> GResult<naga::Module> {
    let source = include_str!("rounded_rect.shader");
    let mut compiler = ShaderCompiler::new();
    let entry = compiler.compile_first(source)?;
    Ok(entry.module)
}

/// 编译内置椭圆着色器
pub fn load_ellipse_shader() -> GResult<naga::Module> {
    let source = include_str!("ellipse.shader");
    let mut compiler = ShaderCompiler::new();
    let entry = compiler.compile_first(source)?;
    Ok(entry.module)
}

/// 编译内置 SDF 文本着色器
pub fn load_sdf_text_shader() -> GResult<naga::Module> {
    let source = include_str!("ui_sdf_text.shader");
    let mut compiler = ShaderCompiler::new();
    let entry = compiler.compile_first(source)?;
    Ok(entry.module)
}

/// 编译内置模糊后处理着色器
pub fn load_blur_shader() -> GResult<naga::Module> {
    let source = include_str!("blur.shader");
    let mut compiler = ShaderCompiler::new();
    let entry = compiler.compile_first(source)?;
    Ok(entry.module)
}

/// 编译内置泛光后处理着色器
pub fn load_bloom_shader() -> GResult<naga::Module> {
    let source = include_str!("bloom.shader");
    let mut compiler = ShaderCompiler::new();
    let entry = compiler.compile_first(source)?;
    Ok(entry.module)
}
