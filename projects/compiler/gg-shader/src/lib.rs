#![warn(missing_docs)]

//! GG Shader 编译器
//!
//! 实现 GG Shader (gs) 语言的完整编译管线，
//! 将 gs 源码编译为 naga IR 中间表示，
//! 支持跨平台着色器分发（WGSL/HLSL/GLSL/SPIR-V/MSL）。

/// gs 语言 AST 模块
pub mod ast;
/// GG Shader 编译器公共 API
pub mod compiler;
/// gs AST → naga IR 转换模块
pub mod lower;
/// gs 语言解析器模块
pub mod parser;
/// naga IR 序列化模块
pub mod serialize;
