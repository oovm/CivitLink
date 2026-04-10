#![warn(missing_docs)]

use std::sync::Arc;

use gg_core::platform::{Window, WindowConfig, WindowEvent};

/// 桌面平台窗口实现
///
/// 基于 winit 提供桌面平台的原生窗口管理能力。
/// 通过内部事件缓冲区接收来自 winit 事件循环的窗口事件，
/// 并在 `poll_events()` 时返回给引擎。
pub struct DesktopWindow {
    /// 窗口配置
    config: WindowConfig,
    /// 是否应该关闭
    should_close: bool,
    /// 窗口事件缓冲区
    event_buffer: Vec<WindowEvent>,
    /// winit 窗口引用（可选，在事件循环创建后设置）
    window: Option<Arc<winit::window::Window>>,
}

impl DesktopWindow {
    /// 创建新的桌面窗口实例
    pub fn new(config: WindowConfig) -> Self {
        Self { config, should_close: false, event_buffer: Vec::new(), window: None }
    }

    /// 使用 winit 窗口创建桌面窗口实例
    pub fn from_winit_window(window: Arc<winit::window::Window>, config: WindowConfig) -> Self {
        let inner_size = window.inner_size();
        Self {
            config: WindowConfig { width: inner_size.width, height: inner_size.height, ..config },
            should_close: false,
            event_buffer: Vec::new(),
            window: Some(window),
        }
    }

    /// 设置 winit 窗口引用
    pub fn set_window(&mut self, window: Arc<winit::window::Window>) {
        let inner_size = window.inner_size();
        self.config.width = inner_size.width;
        self.config.height = inner_size.height;
        self.window = Some(window);
    }

    /// 获取 winit 窗口引用
    pub fn winit_window(&self) -> Option<&Arc<winit::window::Window>> {
        self.window.as_ref()
    }

    /// 推送一个 winit 窗口事件到缓冲区
    ///
    /// 将 winit 的 `WindowEvent` 转换为引擎的 `WindowEvent` 并存入缓冲区。
    pub fn push_winit_event(&mut self, event: &winit::event::WindowEvent) {
        match event {
            winit::event::WindowEvent::Resized(physical_size) => {
                self.config.width = physical_size.width;
                self.config.height = physical_size.height;
                self.event_buffer.push(WindowEvent::Resized {
                    width: physical_size.width,
                    height: physical_size.height,
                });
            }
            winit::event::WindowEvent::CloseRequested => {
                self.should_close = true;
                self.event_buffer.push(WindowEvent::CloseRequested);
            }
            winit::event::WindowEvent::Focused(true) => {
                self.event_buffer.push(WindowEvent::Focused);
            }
            winit::event::WindowEvent::Focused(false) => {
                self.event_buffer.push(WindowEvent::Unfocused);
            }
            _ => {}
        }
    }
}

impl Window for DesktopWindow {
    fn size(&self) -> (u32, u32) {
        if let Some(window) = &self.window {
            let inner_size = window.inner_size();
            (inner_size.width, inner_size.height)
        } else {
            (self.config.width, self.config.height)
        }
    }

    fn set_size(&mut self, width: u32, height: u32) {
        self.config.width = width;
        self.config.height = height;
        if let Some(window) = &self.window {
            let _ = window.request_inner_size(winit::dpi::LogicalSize::new(width, height));
        }
    }

    fn set_title(&mut self, title: &str) {
        self.config.title = title.to_string();
        if let Some(window) = &self.window {
            window.set_title(title);
        }
    }

    fn poll_events(&mut self) -> Vec<WindowEvent> {
        std::mem::take(&mut self.event_buffer)
    }

    fn should_close(&self) -> bool {
        self.should_close
    }
}
