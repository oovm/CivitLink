use gg_core::{GError, GResult};

use crate::{PlatformFileSystem, PlatformInput, PlatformThread, PlatformTime, PlatformWindow};

/// 平台通用服务工厂
///
/// 为各平台提供服务创建的统一接口。
pub trait PlatformServices {
    /// 创建文件系统服务
    fn create_file_system(&self) -> GResult<Box<dyn PlatformFileSystem>>;

    /// 创建输入服务
    fn create_input(&self) -> GResult<Box<dyn PlatformInput>>;

    /// 创建线程服务
    fn create_thread(&self) -> GResult<Box<dyn PlatformThread>>;

    /// 创建时间服务
    fn create_time(&self) -> GResult<Box<dyn PlatformTime>>;

    /// 创建窗口服务
    fn create_window(&self) -> GResult<Box<dyn PlatformWindow>>;
}
