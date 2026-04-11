use gg_core::{GError, GResult};

/// 平台通用运行时接口
///
/// 为各平台提供运行时环境的统一接口。
pub trait PlatformRuntime {
    /// 初始化平台运行时
    fn initialize(&mut self) -> GResult<()>;

    /// 启动平台运行时
    fn start(&mut self) -> GResult<()>;

    /// 停止平台运行时
    fn stop(&mut self) -> GResult<()>;

    /// 清理平台运行时
    fn cleanup(&mut self) -> GResult<()>;

    /// 检查运行时是否正在运行
    fn is_running(&self) -> bool;

    /// 获取平台名称
    fn platform_name(&self) -> &str;

    /// 获取平台版本
    fn platform_version(&self) -> &str;
}
