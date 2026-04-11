#![warn(missing_docs)]

//! GG 编辑器场景视图模块
//! 提供通用场景视图基类和视口管理功能

pub mod base;
pub mod components;
pub mod view;
pub mod viewport;

pub use base::{BaseSceneView, SceneEntity, SceneEntityKind};
pub use components::{RectRenderer, SpriteRenderer, Transform2D};
pub use view::SceneView;
pub use viewport::ViewportState;
