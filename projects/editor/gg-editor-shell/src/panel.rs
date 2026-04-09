//! 编辑器面板 trait 和布局提示

use crate::context::EditorContext;
use gg_core::GResult;

/// 面板位置
pub enum PanelPosition {
    /// 左侧
    Left,
    /// 右侧
    Right,
    /// 中央
    Center,
    /// 底部
    Bottom,
    /// 浮动
    Floating,
}

/// 面板布局提示
///
/// 为布局系统提供面板的位置和尺寸偏好信息。
pub struct PanelLayoutHint {
    /// 面板位置
    pub position: PanelPosition,
    /// 首选尺寸 (宽, 高)
    pub preferred_size: Option<(f32, f32)>,
    /// 最小尺寸 (宽, 高)
    pub min_size: Option<(f32, f32)>,
}

impl Default for PanelLayoutHint {
    fn default() -> Self {
        Self {
            position: PanelPosition::Center,
            preferred_size: None,
            min_size: None,
        }
    }
}

/// 编辑器面板 trait
///
/// 所有编辑器面板都应实现此 trait，通过 `EditorContext` 访问编辑器核心子系统。
pub trait EditorPanel {
    /// 面板名称
    fn name(&self) -> &str;

    /// 是否可见
    fn is_visible(&self) -> bool;

    /// 设置可见性
    fn set_visible(&mut self, visible: bool);

    /// 面板注册时调用
    fn on_register(&mut self, _context: &mut EditorContext) {}

    /// 面板注销时调用
    fn on_unregister(&mut self, _context: &mut EditorContext) {}

    /// 渲染面板
    fn render(&mut self, context: &mut EditorContext) -> GResult<()>;

    /// 面板布局提示
    fn layout_hint(&self) -> PanelLayoutHint {
        PanelLayoutHint::default()
    }
}
