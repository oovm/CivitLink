//! 场景编辑器基础模块
//!
//! 提供场景视图的核心类型、命令、剪贴板和上下文菜单功能。

pub mod clipboard;
pub mod commands;
pub mod context_menu;
pub mod types;
pub mod view;
pub mod view_panel;
pub mod view_render;

pub use clipboard::{ClipboardEntity, EntitySnapshot};
pub use commands::{CreateEntityCommand, DeleteEntityCommand, TransformCommand};
pub use context_menu::SceneContextMenu;
pub use types::{
    GizmoState, SceneEntity, SceneEntityKind, SelectionBox, TransformGizmo, TransformKind, TransformValue,
};
pub use view::BaseSceneView;

pub use crate::scene_view::SceneView;
