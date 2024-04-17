#![warn(missing_docs)]

//! 桌面平台多窗口管理器实现

use std::collections::HashMap;

use gg_core::{
    GResult,
    platform::{Window, WindowConfig, WindowId, WindowManager, WindowManagerEvent},
};

use crate::DesktopWindow;

/// 桌面平台窗口管理器
///
/// 管理多个 `DesktopWindow` 实例，支持创建、销毁和查询窗口。
/// 基于 winit 的多窗口事件循环，为编辑器多面板布局提供基础设施。
pub struct DesktopWindowManager {
    /// 管理的窗口集合
    windows: HashMap<WindowId, DesktopWindow>,
    /// 下一个窗口 ID
    next_id: u64,
}

impl DesktopWindowManager {
    /// 创建新的桌面窗口管理器
    pub fn new() -> Self {
        Self { windows: HashMap::new(), next_id: 1 }
    }

    /// 使用主窗口创建桌面窗口管理器
    ///
    /// 将给定的主窗口注册为 `WindowId(0)`。
    pub fn with_main_window(main_window: DesktopWindow) -> Self {
        let mut windows = HashMap::new();
        windows.insert(WindowId(0), main_window);
        Self { windows, next_id: 1 }
    }

    /// 分配下一个窗口 ID
    fn allocate_id(&mut self) -> WindowId {
        let id = WindowId(self.next_id);
        self.next_id += 1;
        id
    }
}

impl WindowManager for DesktopWindowManager {
    fn create_window(&mut self, config: WindowConfig) -> WindowId {
        let id = self.allocate_id();
        let window = DesktopWindow::new(config);
        self.windows.insert(id, window);
        id
    }

    fn destroy_window(&mut self, id: WindowId) -> GResult<()> {
        self.windows.remove(&id).ok_or_else(|| gg_core::GError {
            kind: gg_core::GErrorKind::Runtime,
            message: format!("Window not found: {:?}", id),
        })?;
        Ok(())
    }

    fn get_window(&mut self, id: WindowId) -> Option<&mut dyn gg_core::platform::Window> {
        self.windows.get_mut(&id).map(|w| w as &mut dyn gg_core::platform::Window)
    }

    fn poll_events(&mut self) -> Vec<WindowManagerEvent> {
        let mut all_events = Vec::new();
        for (window_id, window) in &mut self.windows {
            let events = window.poll_events();
            for event in events {
                all_events.push(WindowManagerEvent { window_id: *window_id, event });
            }
        }
        all_events
    }

    fn window_count(&self) -> usize {
        self.windows.len()
    }
}
