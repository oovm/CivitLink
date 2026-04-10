#![warn(missing_docs)]

//! GG 引擎原生渲染后端
//!
//! 基于平台原生 2D API 实现跨平台渲染，
//! 专为编辑器场景优化，支持 CJK 文本渲染和原生窗口集成。
//! 仅支持桌面平台（Windows、macOS、Linux）。

/// 字体管理器模块
pub mod font_manager;

/// 原生渲染器模块
pub mod renderer;

/// 纹理缓存模块
pub mod texture_cache;

pub use font_manager::FontManager;
pub use renderer::NativeRenderer;
pub use texture_cache::NativeTextureCache;
