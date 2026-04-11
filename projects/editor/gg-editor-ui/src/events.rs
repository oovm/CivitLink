//! 事件系统模块
//!
//! 处理GUI事件，如鼠标点击、键盘输入等
//! 所有事件类型从 gg-ui 重新导出

pub use gg_ui::{EventContext, EventPhase, GuiEvent, Key, KeyModifiers, MouseButton};
