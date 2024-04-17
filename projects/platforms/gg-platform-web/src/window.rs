#![warn(missing_docs)]

use gg_core::platform::{Window, WindowConfig, WindowEvent};

/// Web 平台窗口实现
///
/// 基于 HTML Canvas 元素提供 Web 环境的窗口管理能力。
pub struct WebWindow {
    /// 窗口配置
    config: WindowConfig,
    /// 是否应该关闭
    should_close: bool,
}

impl WebWindow {
    /// 创建新的 Web 窗口实例
    pub fn new(config: WindowConfig) -> Self {
        Self { config, should_close: false }
    }
}

impl Window for WebWindow {
    fn size(&self) -> (u32, u32) {
        #[cfg(target_arch = "wasm32")]
        {
            if let Some(document) = web_sys::window().and_then(|w| w.document()) {
                if let Some(canvas) = document.get_element_by_id("gg-canvas") {
                    if let Some(canvas) = canvas.dyn_ref::<web_sys::HtmlCanvasElement>() {
                        return (canvas.width(), canvas.height());
                    }
                }
            }
        }
        (self.config.width, self.config.height)
    }

    fn set_size(&mut self, width: u32, height: u32) {
        self.config.width = width;
        self.config.height = height;
        #[cfg(target_arch = "wasm32")]
        {
            if let Some(document) = web_sys::window().and_then(|w| w.document()) {
                if let Some(canvas) = document.get_element_by_id("gg-canvas") {
                    if let Some(canvas) = canvas.dyn_ref::<web_sys::HtmlCanvasElement>() {
                        canvas.set_width(width);
                        canvas.set_height(height);
                    }
                }
            }
        }
    }

    fn set_title(&mut self, title: &str) {
        self.config.title = title.to_string();
        #[cfg(target_arch = "wasm32")]
        {
            if let Some(document) = web_sys::window().and_then(|w| w.document()) {
                document.set_title(title);
            }
        }
    }

    fn poll_events(&mut self) -> Vec<WindowEvent> {
        Vec::new()
    }

    fn should_close(&self) -> bool {
        self.should_close
    }
}
