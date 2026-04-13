//! Flexbox 布局引擎模块
//!
//! 自行实现核心 Flexbox 算法，不依赖外部 yoga crate。
//! 支持主轴/交叉轴布局、flex-grow/shrink、justify-content、align-items、
//! flex-wrap、gap 间距、padding/margin 等特性。
//! 布局计算仅在脏标记触发时执行，静止时不会重算。

mod engine;
mod style_mapper;
mod types;

pub use engine::FlexLayoutEngine;
pub use style_mapper::map_uss_to_layout_style;
pub use types::{AlignItems, FlexWrap, JustifyContent, LayoutEdge, LayoutNode, LayoutResult, LayoutStyle};
