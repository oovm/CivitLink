#![warn(missing_docs)]

//! 默认单窗口管理器实现
//!
//! 将单个 `Box<dyn Window>` 包装为 `WindowManager`，用于向后兼容。

use super::window::{Window, WindowConfig, WindowEvent, WindowId, WindowManager, WindowManagerEvent};
use crate::{GError, GErrorKind, GResult};

/// 单窗口管理器
///
/// 将单个窗口包装为 `WindowManager` 接口，用于不需要多窗口的场景。
/// 主窗口的 `WindowId` 固定为 `WindowId(0)`。
pub struct SingleWindowManager {
    /// 管理的单个窗口
    window: Box<dyn Window>,
    /// 窗口是否存在
    exists: bool,
}

impl SingleWindowManager {
    /// 创建新的单窗口管理器
    pub fn new(window: Box<dyn Window>) -> Self {
        Self { window, exists: true }
    }

    /// 获取主窗口 ID
    pub fn main_window_id() -> WindowId {
        WindowId(0)
    }
}

impl WindowManager for SingleWindowManager {
    fn create_window(&mut self, _config: WindowConfig) -> WindowId {
        WindowId(0)
    }

    fn destroy_window(&mut self, id: WindowId) -> GResult<()> {
        if id == WindowId(0) {
            self.exists = false;
            Ok(())
        } else {
            Err(GError {
                kind: GErrorKind::Runtime,
                message: format!("Window not found: {:?}", id),
            })
        }
    }

    fn get_window(&mut self, id: WindowId) -> Option<&mut dyn Window> {
        if id == WindowId(0) && self.exists {
            Some(self.window.as_mut())
        } else {
            None
        }
    }

    fn poll_events(&mut self) -> Vec<WindowManagerEvent> {
        if !self.exists {
            return Vec::new();
        }
        let events = self.window.poll_events();
        events.into_iter().map(|event| WindowManagerEvent { window_id: WindowId(0), event }).collect()
    }

    fn window_count(&self) -> usize {
        if self.exists { 1 } else { 0 }
    }
}
