#![warn(missing_docs)]

//! GG 引擎 HTML 渲染后端
//!
//! 将 widget.md 编译成 HTML + WASM，支持在浏览器中渲染编辑器界面。
//! 通过 wasm-bindgen 和 web-sys 与浏览器 API 交互，实现跨平台的 web 渲染。
//! 专为编辑器场景优化，支持 CJK 文本渲染和浏览器窗口集成。
//! 支持 Web 平台。

/// 字体管理器模块
pub mod font_manager;
/// HTML 渲染器模块
pub mod renderer;
/// HTML 纹理缓存模块
pub mod texture_cache;

pub use font_manager::FontManager;
pub use renderer::{HtmlRenderTarget, HtmlRenderer};
pub use texture_cache::HtmlTextureCache;
