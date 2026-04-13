//! GG Editor UI 系统
//!
//! 基于 Valkyrie Widget 的编辑器UI系统，参考 Unity UI Toolkit 设计理念
//! 为编辑器提供高性能、声明式的UI开发体验

#![warn(missing_docs)]

/// 组件系统模块
pub mod components;
/// 编辑器 DPI 管理模块
pub mod dpi;
/// 事件系统模块
pub mod events;
/// Flexbox 布局引擎模块
pub mod layout;
/// 渲染系统模块
pub mod renderer;
/// 状态管理模块
pub mod state;
/// 样式系统模块
pub mod styles;
/// 文本渲染引擎模块
pub mod text_render;
/// 主题系统模块
pub mod theme;
/// Uber-Shader 模块
pub mod uber_shader;
/// UsageHints GPU 驱动变换模块
pub mod usage_hints;

pub use gg_ui::{
    DirtyFlag, DynamicWidget, EventContext, EventPhase, FlexDirection, GuiEvent, GuiFactory, GuiRenderer, GuiRuntime, Key,
    KeyModifiers, MouseButton, UsageHint, UsageHints, Widget, WidgetLifecycle,
};

pub use dpi::EditorDpiManager;
pub use text_render::{TextCache, TextCacheKey, TextRenderEngine, TextVertex};
pub use theme::{EditorTheme, EditorThemeProvider, default_dark_theme, default_light_theme};
