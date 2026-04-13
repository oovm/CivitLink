#![warn(missing_docs)]

//! GG Galgame 引擎的数据结构定义

pub mod components;
pub mod manifest;
pub mod resources;

pub use components::*;
pub use manifest::*;
pub use resources::*;

pub use pleroma::GameState;
