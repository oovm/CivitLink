#![warn(missing_docs)]

//! GG 引擎清单模块
//! 定义引擎清单格式，提供解析、验证和模板功能

/// 引擎清单类型定义
pub mod manifest;
/// 引擎清单模板
pub mod template;

pub use manifest::{DisplaySection, EngineManifest, EngineSection, GameType, ModulesSection, PlatformEntry, ToolchainSection};
pub use template::{arpg_asset_dirs, arpg_template, custom_template, visual_novel_asset_dirs, visual_novel_template};
