#![warn(missing_docs)]

use gg_core::platform::{Window, WindowConfig, WindowEvent};

use std::sync::{Arc, Mutex};

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::JsCast;

/// Web 平台窗口实现
///
/// 基于 HTML Canvas 元素提供 Web 环境的窗口管理能力。
/// 通过内部事件缓冲区接收来自 DOM 的窗口事件，
/// 并在 `poll_events()` 时返回给引擎。
pub struct WebWindow {
    /// 窗口配置
    config: WindowConfig,
    /// 是否应该关闭
    should_close: bool,
    /// 窗口事件缓冲区
    event_buffer: Vec<WindowEvent>,
    /// DOM 事件监听器推送的事件队列
    dom_event_queue: Arc<Mutex<Vec<WindowEvent>>>,
}

impl WebWindow {
    /// 创建新的 Web 窗口实例
    pub fn new(config: WindowConfig) -> Self {
        Self { config, should_close: false, event_buffer: Vec::new(), dom_event_queue: Arc::new(Mutex::new(Vec::new())) }
    }

    /// 推送窗口大小变更事件到缓冲区
    ///
    /// 由 DOM resize 事件监听器调用，将新的窗口尺寸存入缓冲区。
    pub fn push_resize_event(&mut self, width: u32, height: u32) {
        self.config.width = width;
        self.config.height = height;
        self.event_buffer.push(WindowEvent::Resized { width, height });
    }

    /// 推送窗口焦点变更事件到缓冲区
    ///
    /// 由 DOM focus/blur 事件监听器调用。
    pub fn push_focus_event(&mut self, focused: bool) {
        if focused {
            self.event_buffer.push(WindowEvent::Focused);
        }
        else {
            self.event_buffer.push(WindowEvent::Unfocused);
        }
    }

    /// 推送窗口关闭请求事件到缓冲区
    pub fn push_close_requested(&mut self) {
        self.should_close = true;
        self.event_buffer.push(WindowEvent::CloseRequested);
    }

    /// 在 wasm32 环境下注册 DOM 事件监听器
    ///
    /// 注册 `resize` 和 `focus`/`blur` 事件监听器，
    /// 将 DOM 事件转换为引擎 `WindowEvent` 并推送到内部队列。
    /// 在非 wasm32 环境下此方法为空操作。
    pub fn setup_dom_listeners(&self) {
        #[cfg(target_arch = "wasm32")]
        {
            let queue = self.dom_event_queue.clone();

            if let Some(window) = web_sys::window() {
                let queue_clone = queue.clone();
                let resize_closure = wasm_bindgen::closure::Closure::wrap(Box::new(move || {
                    if let Some(w) = web_sys::window() {
                        let width = w.inner_width().ok().and_then(|v| v.as_f64()).unwrap_or(0.0) as u32;
                        let height = w.inner_height().ok().and_then(|v| v.as_f64()).unwrap_or(0.0) as u32;
                        if let Ok(mut q) = queue_clone.lock() {
                            q.push(WindowEvent::Resized { width, height });
                        }
                    }
                }) as Box<dyn Fn()>);

                let _ = window.add_event_listener_with_callback("resize", resize_closure.as_ref().unchecked_ref());
                resize_closure.forget();

                let queue_focus = queue.clone();
                let focus_closure = wasm_bindgen::closure::Closure::wrap(Box::new(move || {
                    if let Ok(mut q) = queue_focus.lock() {
                        q.push(WindowEvent::Focused);
                    }
                }) as Box<dyn Fn()>);

                let _ = window.add_event_listener_with_callback("focus", focus_closure.as_ref().unchecked_ref());
                focus_closure.forget();

                let queue_blur = queue.clone();
                let blur_closure = wasm_bindgen::closure::Closure::wrap(Box::new(move || {
                    if let Ok(mut q) = queue_blur.lock() {
                        q.push(WindowEvent::Unfocused);
                    }
                }) as Box<dyn Fn()>);

                let _ = window.add_event_listener_with_callback("blur", blur_closure.as_ref().unchecked_ref());
                blur_closure.forget();
            }
        }
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
        if let Ok(mut dom_events) = self.dom_event_queue.lock() {
            self.event_buffer.extend(dom_events.drain(..));
        }
        std::mem::take(&mut self.event_buffer)
    }

    fn should_close(&self) -> bool {
        self.should_close
    }
}
