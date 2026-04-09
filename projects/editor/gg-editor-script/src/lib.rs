#![warn(missing_docs)]

//! GG 剧本编辑器面板
//! 提供剧本节点图编辑功能

pub mod graph;
pub mod panel;
pub mod templates;

pub use graph::{NodeGraph, NodeGraphEntry};
pub use panel::ScriptEditorPanel;
pub use templates::ScriptTemplate;
