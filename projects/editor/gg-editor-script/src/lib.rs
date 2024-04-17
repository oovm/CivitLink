#![warn(missing_docs)]

//! GG 脚本编辑器面板
//! 提供脚本文件浏览和外部 IDE 集成功能

pub mod ide;
pub mod panel;
pub mod script_file;

pub use ide::{IdeDescriptor, IdeLauncher};
pub use panel::ScriptEditorPanel;
pub use script_file::ScriptFile;
