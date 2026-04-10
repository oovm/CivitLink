#![warn(missing_docs)]

//! GG 引擎 WGPU 渲染后端
//!
//! 基于 WGPU 实现跨平台 2D 图形渲染，
//! 支持精灵批渲染、文本渲染、几何图形绘制、场景过渡动画、
//! 相机变换和裁剪矩形。

/// 字形纹理图集模块
pub mod glyph_atlas;
/// 字形缓存模块
pub mod glyph_cache;
/// 渲染管线模块
pub mod pipeline;
/// WGPU 渲染器模块
pub mod renderer;
/// 着色器源码模块
pub mod shader;
/// 精灵批渲染模块
pub mod sprite_batch;
/// 纹理缓存模块
pub mod texture_cache;
/// Uniform 缓冲区池模块
pub mod uniform_pool;

pub use renderer::{RenderTarget, WgpuRenderer};
