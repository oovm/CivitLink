#![warn(missing_docs)]

//! GG 引擎 Galgame Schema 模块
//!
//! 提供 Galgame 引擎所需的所有 ECS 组件、资源和清单类型定义。
//! 作为 schema 类型的唯一权威来源，供引擎、插件和编辑器共同依赖。

pub mod components;
pub mod manifest;
pub mod resources;
