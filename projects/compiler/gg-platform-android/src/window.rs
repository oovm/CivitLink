use gg_core::platform::{Window, WindowConfig, WindowEvent};

/// Android 平台窗口实现
///
/// 为 Android 平台提供窗口操作的具体实现。
pub struct AndroidWindow {
    /// 窗口配置
    config: WindowConfig,
    /// 是否应该关闭
    should_close: bool,
    /// 窗口事件缓冲区
    event_buffer: Vec<WindowEvent>,
}

impl AndroidWindow {
    /// 创建 Android 窗口实例
    pub fn new(config: WindowConfig) -> Self {
        Self { config, should_close: false, event_buffer: Vec::new() }
    }
}

impl Default for AndroidWindow {
    fn default() -> Self {
        Self::new(WindowConfig::default())
    }
}

impl Window for AndroidWindow {
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
        std::mem::take(&mut self.event_buffer)
    }

    fn should_close(&self) -> bool {
        self.should_close
    }
}
