//! 编辑器面板 trait 和布局提示

use crate::{context::EditorContext, event::EditorEvent};
use gg_core::GResult;
use gg_ui::{UiNodeId, UiTree};

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
        Self { position: PanelPosition::Center, preferred_size: None, min_size: None }
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

    /// 处理编辑器事件
    ///
    /// 面板可在此方法中响应输入事件（鼠标/键盘）和其他编辑器事件。
    fn on_event(&mut self, event: &EditorEvent, context: &mut EditorContext) {
        let _ = (event, context);
    }

    /// 构建面板 UI 节点树
    ///
    /// 返回面板 UI 子树的根节点 ID，若面板不创建任何节点则返回 `None`。
    ///
    /// # 参数
    ///
    /// - `context` - 编辑器上下文
    /// - `ui_tree` - UI 节点树
    fn build_ui(&mut self, context: &mut EditorContext, ui_tree: &mut UiTree) -> Option<UiNodeId>;

    /// 渲染面板（已弃用，保留向后兼容）
    #[deprecated(note = "使用 build_ui 替代")]
    fn render(&mut self, _context: &mut EditorContext) -> GResult<()> {
        Ok(())
    }

    /// 面板布局提示
    fn layout_hint(&self) -> PanelLayoutHint {
        PanelLayoutHint::default()
    }

    /// 面板初始化优先级
    ///
    /// 数值越小越优先初始化。默认为 100。
    fn priority(&self) -> u32 {
        100
    }
}
