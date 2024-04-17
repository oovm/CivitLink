#![warn(missing_docs)]

//! 平台服务聚合
//! 将文件系统、输入、时间、窗口和线程服务统一管理

use super::{fs::FileSystem, input::Input, thread::RuntimeThread, time::Time, window::Window};

/// 平台服务集合
///
/// 聚合了引擎运行所需的所有平台底层服务，
/// 包括文件系统、输入处理、时间管理、窗口管理和线程调度。
pub struct PlatformServices {
    /// 文件系统服务
    pub file_system: Box<dyn FileSystem>,
    /// 输入服务
    pub input: Box<dyn Input>,
    /// 时间服务
    pub time: Box<dyn Time>,
    /// 窗口服务
    pub window: Box<dyn Window>,
    /// 线程服务
    pub runtime_thread: Box<dyn RuntimeThread>,
}

impl PlatformServices {
    /// 创建包含所有服务的平台服务实例
    pub fn new(
        file_system: Box<dyn FileSystem>,
        input: Box<dyn Input>,
        time: Box<dyn Time>,
        window: Box<dyn Window>,
        runtime_thread: Box<dyn RuntimeThread>,
    ) -> Self {
        Self { file_system, input, time, window, runtime_thread }
    }
}
