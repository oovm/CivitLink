#![warn(missing_docs)]

//! GG 编辑器壳程序框架
//! 提供编辑器面板 trait 和注册机制

pub mod panel;
pub mod shell;

pub use panel::{EditorPanel, PanelContext, PanelData};
pub use shell::EditorShell;
