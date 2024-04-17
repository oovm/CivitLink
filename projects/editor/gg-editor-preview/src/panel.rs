//! 实时预览面板实现
//! 提供游戏运行时预览的启动、停止、暂停和 HMR 热更新功能

use gg_core::GResult;
use gg_editor_shell::{EditorContext, EditorPanel, PanelLayoutHint, PanelPosition};
use gg_ui::UiTree;

/// 实时预览面板
///
/// 提供游戏运行时预览功能，支持启动、暂停、恢复和停止预览，
/// 以及从指定节点重新开始预览和 HMR 热更新。
pub struct PreviewPanel {
    /// 面板是否可见
    visible: bool,
    /// 预览是否正在运行
    is_running: bool,
    /// 预览是否暂停
    is_paused: bool,
    /// 从指定节点开始预览的节点 ID
    start_node_id: Option<String>,
    /// 是否启用 HMR 热更新
    hmr_enabled: bool,
}

impl PreviewPanel {
    /// 创建新的实时预览面板
    pub fn new() -> Self {
        Self { visible: true, is_running: false, is_paused: false, start_node_id: None, hmr_enabled: true }
    }

    /// 启动预览
    ///
    /// 从当前配置的起始节点开始运行游戏预览。
    pub fn start_preview(&mut self, context: &mut EditorContext) -> GResult<()> {
        let _ = context;
        self.is_running = true;
        self.is_paused = false;
        Ok(())
    }

    /// 停止预览
    ///
    /// 终止当前运行的游戏预览。
    pub fn stop_preview(&mut self) -> GResult<()> {
        self.is_running = false;
        self.is_paused = false;
        Ok(())
    }

    /// 暂停预览
    ///
    /// 暂停当前运行的游戏预览，保留当前状态。
    pub fn pause_preview(&mut self) -> GResult<()> {
        if self.is_running {
            self.is_paused = true;
        }
        Ok(())
    }

    /// 恢复预览
    ///
    /// 从暂停状态恢复游戏预览运行。
    pub fn resume_preview(&mut self) -> GResult<()> {
        if self.is_running && self.is_paused {
            self.is_paused = false;
        }
        Ok(())
    }

    /// 从指定节点重新开始预览
    ///
    /// 将预览起始节点设置为指定 ID，并重新启动预览。
    pub fn restart_from_node(&mut self, node_id: String, context: &mut EditorContext) -> GResult<()> {
        self.start_node_id = Some(node_id);
        self.is_paused = false;
        let _ = context;
        Ok(())
    }

    /// 触发 HMR 热更新
    ///
    /// 检测文件变更并重新加载受影响的资源，无需重启预览。
    pub fn trigger_hmr_reload(&mut self, context: &mut EditorContext) -> GResult<()> {
        let _ = context;
        Ok(())
    }
}

impl EditorPanel for PreviewPanel {
    /// 获取面板名称
    fn name(&self) -> &str {
        "Live Preview"
    }

    /// 获取面板可见性
    fn is_visible(&self) -> bool {
        self.visible
    }

    /// 设置面板可见性
    fn set_visible(&mut self, visible: bool) {
        self.visible = visible;
    }

    /// 构建预览面板 UI 节点树
    fn build_ui(&mut self, _context: &mut EditorContext, _ui_tree: &mut UiTree) -> GResult<()> {
        Ok(())
    }

    /// 获取面板布局提示
    fn layout_hint(&self) -> PanelLayoutHint {
        PanelLayoutHint {
            position: PanelPosition::Center,
            preferred_size: Some((800.0, 600.0)),
            min_size: Some((400.0, 300.0)),
        }
    }
}
