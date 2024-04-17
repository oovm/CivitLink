//! 编辑器 DPI 管理模块
//!
//! 提供编辑器级别的 DPI 缩放管理，支持系统 DPI 检测和缩放因子变更通知。

use gg_ui::{DpiScale, Signal};

/// 编辑器 DPI 管理器
pub struct EditorDpiManager {
    /// 当前缩放因子
    current_scale: DpiScale,
    /// 缩放因子变更信号
    change_signal: Signal<DpiScale>,
}

impl EditorDpiManager {
    /// 创建管理器，使用 identity 缩放
    pub fn new() -> Self {
        Self { current_scale: DpiScale::identity(), change_signal: Signal::new(DpiScale::identity()) }
    }

    /// 检测系统 DPI 缩放因子
    ///
    /// 简单实现：返回 DpiScale::identity()
    /// 实际系统检测需要平台 API，此处为占位
    pub fn detect_system_dpi() -> DpiScale {
        DpiScale::identity()
    }

    /// 设置缩放因子并通知变更
    pub fn set_scale(&mut self, scale: DpiScale) {
        self.current_scale = scale;
        self.change_signal.set(scale);
    }

    /// 获取缩放因子变更信号
    pub fn on_scale_change(&self) -> &Signal<DpiScale> {
        &self.change_signal
    }

    /// 获取当前缩放因子
    pub fn current_scale(&self) -> &DpiScale {
        &self.current_scale
    }
}

impl Default for EditorDpiManager {
    fn default() -> Self {
        Self::new()
    }
}
