#![warn(missing_docs)]

//! GG 引擎 WGPU 渲染后端
//!
//! 基于 WGPU 实现跨平台 2D 图形渲染，
//! 支持精灵绘制、文本渲染、矩形绘制和场景过渡动画。

/// 字形缓存模块
pub mod glyph_cache;
/// 渲染管线模块
pub mod pipeline;
/// WGPU 渲染器模块
pub mod renderer;
/// 着色器源码模块
pub mod shader;
/// 纹理缓存模块
pub mod texture_cache;

pub use renderer::WgpuRenderer;
