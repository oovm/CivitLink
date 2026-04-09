#![warn(missing_docs)]

//! GG 引擎 UI 组件库
//! 提供声明式 UI 组件、布局系统和事件处理

/// UI 事件系统
pub mod event;
/// 布局引擎
pub mod layout;
/// UI 节点树
pub mod node;
/// UI 渲染
pub mod render;
/// UI 样式定义
pub mod style;
/// 控件 trait
pub mod widget;
/// 内置控件
pub mod widgets;

pub use event::{EventSystem, UiEvent};
pub use layout::{LayoutEngine, LayoutResult};
pub use node::{UiNode, UiNodeData, UiNodeId, UiTree};
pub use render::UiRenderer;
pub use style::*;
pub use widget::Widget;
pub use widgets::{Button, ButtonState, Panel, TextBox};
