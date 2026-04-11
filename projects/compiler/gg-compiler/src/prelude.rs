//! GG 编译器核心 prelude 模块
//! 导出最常用的编译器核心类型

pub use crate::{
    artifact::{Artifact, ArtifactKey, ArtifactSet},
    context::{BuildConfig, BuildContext, Diagnostic, DiagnosticLevel, SourceLocation},
    ir::{IrFunction, IrModule, IrValue, OpCode},
    pipeline::Pipeline,
    transformer::Transformer,
};
