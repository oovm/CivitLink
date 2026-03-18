//! 窗口抽象
//!
//! 提供跨平台的窗口接口。

use super::{PlatformError, PlatformResult};

/// 窗口配置
#[derive(Clone, Debug)]
pub struct WindowConfig {
    /// 窗口标题
    pub title: String,
    /// 窗口宽度
    pub width: u32,
    /// 窗口高度
    pub height: u32,
    /// 是否全屏
    pub fullscreen: bool,
    /// 是否可调整大小
    pub resizable: bool,
    /// 是否可见
    pub visible: bool,
    /// 是否有装饰（标题栏、边框等）
    pub decorated: bool,
}

impl Default for WindowConfig {
    fn default() -> Self {
        Self {
            title: "GWG Game".to_string(),
            width: 800,
            height: 600,
            fullscreen: false,
            resizable: true,
            visible: true,
            decorated: true,
        }
    }
}

/// 窗口大小
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct WindowSize {
    pub width: u32,
    pub height: u32,
}

impl WindowSize {
    /// 创建新的窗口大小
    pub fn new(width: u32, height: u32) -> Self {
        Self { width, height }
    }

    /// 获取宽高比
    pub fn aspect_ratio(&self) -> f64 {
        if self.height == 0 {
            1.0
        } else {
            self.width as f64 / self.height as f64
        }
    }
}

/// 窗口位置
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct WindowPosition {
    pub x: i32,
    pub y: i32,
}

impl WindowPosition {
    /// 创建新的窗口位置
    pub fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }

    /// 居中位置
    pub fn centered() -> Self {
        Self { x: -1, y: -1 }
    }
}

/// 窗口事件
#[derive(Clone, Debug)]
pub enum WindowEvent {
    /// 窗口关闭
    CloseRequested,
    /// 窗口大小改变
    Resized(WindowSize),
    /// 窗口移动
    Moved(WindowPosition),
    /// 窗口获得焦点
    FocusGained,
    /// 窗口失去焦点
    FocusLost,
    /// 窗口最小化
    Minimized,
    /// 窗口最大化
    Maximized,
    /// 窗口恢复
    Restored,
}

/// 窗口 trait
pub trait Window {
    /// 获取窗口配置
    fn config(&self) -> &WindowConfig;

    /// 获取窗口大小
    fn size(&self) -> WindowSize;

    /// 设置窗口大小
    fn set_size(&mut self, size: WindowSize) -> PlatformResult<()>;

    /// 获取窗口位置
    fn position(&self) -> WindowPosition;

    /// 设置窗口位置
    fn set_position(&mut self, position: WindowPosition) -> PlatformResult<()>;

    /// 获取窗口标题
    fn title(&self) -> &str;

    /// 设置窗口标题
    fn set_title(&mut self, title: &str) -> PlatformResult<()>;

    /// 检查窗口是否全屏
    fn is_fullscreen(&self) -> bool;

    /// 设置是否全屏
    fn set_fullscreen(&mut self, fullscreen: bool) -> PlatformResult<()>;

    /// 检查窗口是否可见
    fn is_visible(&self) -> bool;

    /// 设置窗口可见性
    fn set_visible(&mut self, visible: bool) -> PlatformResult<()>;

    /// 交换缓冲区
    fn swap_buffers(&mut self) -> PlatformResult<()>;

    /// 获取窗口事件迭代器
    fn events(&self) -> Box<dyn Iterator<Item = WindowEvent> + '_>;

    /// 清空窗口事件（每帧结束调用）
    fn clear_events(&mut self);
}

/// 空窗口（用于平台不支持窗口的情况）
pub struct NullWindow {
    config: WindowConfig,
    size: WindowSize,
}

impl NullWindow {
    /// 创建新的 NullWindow
    pub fn new(config: WindowConfig) -> Self {
        let size = WindowSize::new(config.width, config.height);
        Self { config, size }
    }
}

impl Default for NullWindow {
    fn default() -> Self {
        Self::new(WindowConfig::default())
    }
}

impl Window for NullWindow {
    fn config(&self) -> &WindowConfig {
        &self.config
    }

    fn size(&self) -> WindowSize {
        self.size
    }

    fn set_size(&mut self, size: WindowSize) -> PlatformResult<()> {
        self.size = size;
        Ok(())
    }

    fn position(&self) -> WindowPosition {
        WindowPosition::new(0, 0)
    }

    fn set_position(&mut self, _position: WindowPosition) -> PlatformResult<()> {
        Ok(())
    }

    fn title(&self) -> &str {
        &self.config.title
    }

    fn set_title(&mut self, title: &str) -> PlatformResult<()> {
        self.config.title = title.to_string();
        Ok(())
    }

    fn is_fullscreen(&self) -> bool {
        self.config.fullscreen
    }

    fn set_fullscreen(&mut self, fullscreen: bool) -> PlatformResult<()> {
        self.config.fullscreen = fullscreen;
        Ok(())
    }

    fn is_visible(&self) -> bool {
        self.config.visible
    }

    fn set_visible(&mut self, visible: bool) -> PlatformResult<()> {
        self.config.visible = visible;
        Ok(())
    }

    fn swap_buffers(&mut self) -> PlatformResult<()> {
        Ok(())
    }

    fn events(&self) -> Box<dyn Iterator<Item = WindowEvent> + '_> {
        Box::new(std::iter::empty())
    }

    fn clear_events(&mut self) {}
}
