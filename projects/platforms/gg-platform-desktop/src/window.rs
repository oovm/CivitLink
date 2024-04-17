#![warn(missing_docs)]

use gg_core::platform::{Window, WindowConfig, WindowEvent};

/// 桌面平台窗口实现
///
/// 基于 `WindowConfig` 维护窗口状态的占位实现。
/// 未来将集成 winit 提供完整的原生窗口管理。
pub struct DesktopWindow {
    /// 窗口配置
    config: WindowConfig,
    /// 是否应该关闭
    should_close: bool,
}

impl DesktopWindow {
    /// 创建新的桌面窗口实例
    pub fn new(config: WindowConfig) -> Self {
        Self { config, should_close: false }
    }
}

impl Window for DesktopWindow {
    fn size(&self) -> (u32, u32) {
        (self.config.width, self.config.height)
    }

    fn set_size(&mut self, width: u32, height: u32) {
        self.config.width = width;
        self.config.height = height;
    }

    fn set_title(&mut self, title: &str) {
        self.config.title = title.to_string();
    }

    fn poll_events(&mut self) -> Vec<WindowEvent> {
        Vec::new()
    }

    fn should_close(&self) -> bool {
        self.should_close
    }
}
