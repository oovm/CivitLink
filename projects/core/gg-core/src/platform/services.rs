#![warn(missing_docs)]

//! 平台服务聚合
//! 将文件系统、输入和时间服务统一管理

use super::{fs::FileSystem, input::Input, time::Time};

/// 平台服务集合
///
/// 聚合了引擎运行所需的所有平台底层服务，
/// 包括文件系统、输入处理和时间管理。
pub struct PlatformServices {
    /// 文件系统服务
    pub file_system: Box<dyn FileSystem>,
    /// 输入服务
    pub input: Box<dyn Input>,
    /// 时间服务
    pub time: Box<dyn Time>,
}

impl PlatformServices {
    /// 创建平台服务实例
    pub fn new(file_system: Box<dyn FileSystem>, input: Box<dyn Input>, time: Box<dyn Time>) -> Self {
        Self { file_system, input, time }
    }
}
