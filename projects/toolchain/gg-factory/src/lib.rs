#![warn(missing_docs)]

//! GG 引擎工厂模块
//! 根据引擎清单生成引擎项目代码

/// 引擎代码生成器模块
pub mod generator;

pub use generator::EngineFactory;
