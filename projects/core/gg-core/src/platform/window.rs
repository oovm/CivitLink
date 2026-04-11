#![warn(missing_docs)]

//! 窗口/显示抽象层
//! 提供跨平台的窗口管理接口

use crate::GResult;

/// 窗口唯一标识符
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct WindowId(
    /// 内部 ID 值
    pub u64,
);

/// 窗口事件
#[derive(Debug, Clone, PartialEq)]
pub enum WindowEvent {
    /// 窗口尺寸变更
    Resized {
        /// 新宽度
        width: u32,
        /// 新高度
        height: u32,
    },
    /// 窗口关闭请求
    CloseRequested,
    /// 窗口获得焦点
    Focused,
    /// 窗口失去焦点
    Unfocused,
    /// 新窗口已创建
    WindowCreated {
        /// 新窗口的 ID
        id: WindowId,
    },
    /// 窗口已销毁
    WindowDestroyed {
        /// 被销毁窗口的 ID
        id: WindowId,
    },
}

/// 窗口管理器事件
///
/// 携带窗口 ID 标识事件来源窗口。
#[derive(Debug, Clone, PartialEq)]
pub struct WindowManagerEvent {
    /// 事件来源窗口 ID
    pub window_id: WindowId,
    /// 窗口事件
    pub event: WindowEvent,
}

/// 窗口配置
#[derive(Debug, Clone)]
pub struct WindowConfig {
    /// 窗口标题
    pub title: String,
    /// 窗口宽度（像素）
    pub width: u32,
    /// 窗口高度（像素）
    pub height: u32,
    /// 是否全屏
    pub fullscreen: bool,
    /// 是否可调整大小
    pub resizable: bool,
}

impl WindowConfig {
    /// 创建新的窗口配置
    pub fn new(title: impl Into<String>, width: u32, height: u32) -> Self {
        Self { title: title.into(), width, height, fullscreen: false, resizable: true }
    }
}

impl Default for WindowConfig {
    fn default() -> Self {
        Self::new("GG Game", 1280, 720)
    }
}

/// 窗口抽象 trait
pub trait Window: Send + Sync + 'static {
    /// 获取窗口尺寸
    fn size(&self) -> (u32, u32);

    /// 设置窗口尺寸
    fn set_size(&mut self, width: u32, height: u32);

    /// 设置窗口标题
    fn set_title(&mut self, title: &str);

    /// 轮询窗口事件
    fn poll_events(&mut self) -> Vec<WindowEvent>;

    /// 检查窗口是否应该关闭
    fn should_close(&self) -> bool;
}

/// 窗口管理器 trait
///
/// 支持创建、销毁和查询多个窗口，为编辑器多面板布局提供基础设施。
pub trait WindowManager: Send + Sync + 'static {
    /// 创建新窗口，返回窗口 ID
    fn create_window(&mut self, config: WindowConfig) -> WindowId;

    /// 销毁指定窗口
    fn destroy_window(&mut self, id: WindowId) -> GResult<()>;

    /// 获取指定窗口的可变引用
    fn get_window(&mut self, id: WindowId) -> Option<&mut dyn Window>;

    /// 轮询所有窗口事件
    fn poll_events(&mut self) -> Vec<WindowManagerEvent>;

    /// 获取管理的窗口数量
    fn window_count(&self) -> usize;
}
