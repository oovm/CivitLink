use gg_core::{GError, GResult};

/// 平台通用窗口接口
///
/// 为各平台提供窗口操作的统一接口。
pub trait PlatformWindow {
    /// 创建窗口
    fn create(&mut self, title: &str, width: u32, height: u32) -> GResult<()>;

    /// 显示窗口
    fn show(&mut self) -> GResult<()>;

    /// 隐藏窗口
    fn hide(&mut self) -> GResult<()>;

    /// 关闭窗口
    fn close(&mut self) -> GResult<()>;

    /// 设置窗口标题
    fn set_title(&mut self, title: &str) -> GResult<()>;

    /// 设置窗口大小
    fn set_size(&mut self, width: u32, height: u32) -> GResult<()>;

    /// 获取窗口大小
    fn size(&self) -> (u32, u32);

    /// 设置窗口位置
    fn set_position(&mut self, x: i32, y: i32) -> GResult<()>;

    /// 获取窗口位置
    fn position(&self) -> (i32, i32);

    /// 设置窗口是否可调整大小
    fn set_resizable(&mut self, resizable: bool) -> GResult<()>;

    /// 获取窗口是否可调整大小
    fn is_resizable(&self) -> bool;

    /// 处理窗口事件
    fn process_events(&mut self) -> GResult<()>;

    /// 检查窗口是否正在运行
    fn is_running(&self) -> bool;
}
