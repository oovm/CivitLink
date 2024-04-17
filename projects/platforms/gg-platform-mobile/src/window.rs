use gg_core::platform::{Window, WindowConfig, WindowEvent};

/// 移动平台窗口实现
///
/// 占位实现，移动平台的窗口由系统管理（全屏应用）。
/// 未来将集成 iOS UIKit 和 Android Activity 的窗口管理。
pub struct MobileWindow {
    /// 窗口配置
    config: WindowConfig,
    /// 是否应该关闭
    should_close: bool,
}

impl MobileWindow {
    /// 创建新的移动窗口实例
    pub fn new(config: WindowConfig) -> Self {
        Self { config, should_close: false }
    }
}

impl Window for MobileWindow {
    fn size(&self) -> (u32, u32) {
        (self.config.width, self.config.height)
    }

    fn set_size(&mut self, width: u32, height: u32) {
        self.config.width = width;
        self.config.height = height;
    }

    fn set_title(&mut self, _title: &str) {}

    fn poll_events(&mut self) -> Vec<WindowEvent> {
        Vec::new()
    }

    fn should_close(&self) -> bool {
        self.should_close
    }
}
