#![warn(missing_docs)]

//! 桌面平台多窗口管理器实现

use std::collections::HashMap;

use gg_core::platform::{Window, WindowConfig, WindowId, WindowManager, WindowManagerEvent};
use gg_core::GResult;

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
        Self {
            windows: HashMap::new(),
            next_id: 1,
        }
    }

    /// 使用主窗口创建桌面窗口管理器
    ///
    /// 将给定的主窗口注册为 `WindowId(0)`。
    pub fn with_main_window(main_window: DesktopWindow) -> Self {
        let mut windows = HashMap::new();
        windows.insert(WindowId(0), main_window);
        Self {
            windows,
            next_id: 1,
        }
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
                all_events.push(WindowManagerEvent {
                    window_id: *window_id,
                    event,
                });
            }
        }
        all_events
    }

    fn window_count(&self) -> usize {
        self.windows.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gg_core::platform::WindowManager;

    #[test]
    fn test_create_window_manager() {
        let manager = DesktopWindowManager::new();
        assert_eq!(manager.window_count(), 0);
    }

    #[test]
    fn test_create_and_destroy_window() {
        let mut manager = DesktopWindowManager::new();
        let id = manager.create_window(WindowConfig::default());
        assert_eq!(manager.window_count(), 1);
        assert!(manager.get_window(id).is_some());
        manager.destroy_window(id).unwrap();
        assert_eq!(manager.window_count(), 0);
        assert!(manager.get_window(id).is_none());
    }

    #[test]
    fn test_destroy_nonexistent_window() {
        let mut manager = DesktopWindowManager::new();
        let result = manager.destroy_window(WindowId(999));
        assert!(result.is_err());
    }

    #[test]
    fn test_with_main_window() {
        let main_window = DesktopWindow::new(WindowConfig::default());
        let manager = DesktopWindowManager::with_main_window(main_window);
        assert_eq!(manager.window_count(), 1);
    }

    #[test]
    fn test_multiple_windows() {
        let mut manager = DesktopWindowManager::new();
        let id1 = manager.create_window(WindowConfig::new("Window 1", 800, 600));
        let id2 = manager.create_window(WindowConfig::new("Window 2", 1024, 768));
        assert_eq!(manager.window_count(), 2);
        assert_ne!(id1, id2);
        assert!(manager.get_window(id1).is_some());
        assert!(manager.get_window(id2).is_some());
    }

    #[test]
    fn test_poll_events_empty() {
        let mut manager = DesktopWindowManager::new();
        let events = manager.poll_events();
        assert!(events.is_empty());
    }

    #[test]
    fn test_window_event_routing() {
        let mut manager = DesktopWindowManager::new();
        let id1 = manager.create_window(WindowConfig::new("Window 1", 800, 600));
        let id2 = manager.create_window(WindowConfig::new("Window 2", 1024, 768));

        let events = manager.poll_events();
        assert!(events.is_empty());

        assert!(manager.get_window(id1).is_some());
        assert!(manager.get_window(id2).is_some());

        {
            let w1 = manager.get_window(id1).unwrap();
            assert_eq!(w1.size(), (800, 600));
        }
        {
            let w2 = manager.get_window(id2).unwrap();
            assert_eq!(w2.size(), (1024, 768));
        }
    }
}
