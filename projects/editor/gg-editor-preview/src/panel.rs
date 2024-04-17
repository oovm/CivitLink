//! 实时预览面板实现
//! 提供游戏运行时预览的启动、停止、暂停和 HMR 热更新功能

use crate::{
    hmr::{HmrManager, HmrReloadResult},
    session::{PreviewSession, PreviewState},
    viewport::ViewportRenderer,
};
use gg_core::GResult;
use gg_editor_shell::{EditorContext, EditorPanel, PanelLayoutHint, PanelPosition, event::EditorEvent};
use gg_render::{Color, TextureId};
use gg_ui::{FlexAlign, FlexDirection, FontStyle, LayoutStyle, Overflow, SizeValue, Style, UiNodeData, UiNodeId, UiTree};

/// HMR 热更新状态
///
/// 描述 HMR 文件监视和热更新的当前状态。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HmrStatus {
    /// 空闲，未检测变更
    Idle,
    /// 正在检测文件变更
    Detecting,
    /// 正在重新加载资源
    Reloading,
    /// 热更新成功完成
    Success,
    /// 热更新失败
    Failed,
}

impl HmrStatus {
    /// 获取状态的显示文本
    pub fn display_text(&self) -> &str {
        match self {
            HmrStatus::Idle => "Idle",
            HmrStatus::Detecting => "Detecting...",
            HmrStatus::Reloading => "Reloading...",
            HmrStatus::Success => "Success",
            HmrStatus::Failed => "Failed",
        }
    }

    /// 获取状态对应的显示颜色
    pub fn display_color(&self) -> Color {
        match self {
            HmrStatus::Idle => Color::new(0.5, 0.5, 0.5, 1.0),
            HmrStatus::Detecting => Color::new(0.8, 0.8, 0.0, 1.0),
            HmrStatus::Reloading => Color::new(0.0, 0.6, 1.0, 1.0),
            HmrStatus::Success => Color::new(0.5, 0.8, 0.5, 1.0),
            HmrStatus::Failed => Color::new(0.9, 0.3, 0.3, 1.0),
        }
    }
}

/// 实时预览面板
///
/// 提供游戏运行时预览功能，支持启动、暂停、恢复和停止预览，
/// 以及从指定节点重新开始预览和 HMR 热更新。
pub struct PreviewPanel {
    /// 面板是否可见
    visible: bool,
    /// 预览状态
    state: PreviewState,
    /// 从指定节点开始预览的节点 ID
    start_node_id: Option<String>,
    /// 是否启用 HMR 热更新
    hmr_enabled: bool,
    /// HMR 热更新状态
    hmr_status: HmrStatus,
    /// 预览会话
    session: Option<PreviewSession>,
    /// 视口渲染纹理标识
    viewport_texture_id: Option<TextureId>,
    /// FPS 计数
    fps_counter: f32,
    /// 累计帧数
    frame_count: u64,
    /// HMR 热更新管理器
    hmr_manager: HmrManager,
    /// 当前场景名称
    scene_name: String,
    /// 当前场景对象数量
    object_count: usize,
    /// 视口渲染器
    viewport_renderer: ViewportRenderer,
}

impl PreviewPanel {
    /// 创建新的实时预览面板
    pub fn new() -> Self {
        Self {
            visible: true,
            state: PreviewState::Idle,
            start_node_id: None,
            hmr_enabled: true,
            hmr_status: HmrStatus::Idle,
            session: None,
            viewport_texture_id: None,
            fps_counter: 0.0,
            frame_count: 0,
            hmr_manager: HmrManager::new(),
            scene_name: "Main".to_string(),
            object_count: 0,
        }
    }

    /// 启动预览
    ///
    /// 从当前配置的起始节点开始运行游戏预览。
    pub fn start_preview(&mut self, context: &mut EditorContext) -> GResult<()> {
        if self.state != PreviewState::Idle {
            return Ok(());
        }

        if self.session.is_none() {
            let session = PreviewSession::new(None)?;
            self.session = Some(session);
        }

        if let Some(ref mut session) = self.session {
            session.start();
        }

        self.state = PreviewState::Running;
        context.events_mut().publish(EditorEvent::PreviewStarted);
        Ok(())
    }

    /// 停止预览
    ///
    /// 终止当前运行的游戏预览。
    pub fn stop_preview(&mut self) -> GResult<()> {
        if self.state == PreviewState::Idle {
            return Ok(());
        }

        if let Some(ref mut session) = self.session {
            session.stop();
        }

        self.state = PreviewState::Idle;
        self.frame_count = 0;
        self.fps_counter = 0.0;
        self.hmr_status = HmrStatus::Idle;
        Ok(())
    }

    /// 暂停预览
    ///
    /// 暂停当前运行的游戏预览，保留当前状态。
    pub fn pause_preview(&mut self) -> GResult<()> {
        if self.state != PreviewState::Running {
            return Ok(());
        }

        if let Some(ref mut session) = self.session {
            session.pause();
        }

        self.state = PreviewState::Paused;
        Ok(())
    }

    /// 恢复预览
    ///
    /// 从暂停状态恢复游戏预览运行。
    pub fn resume_preview(&mut self) -> GResult<()> {
        if self.state != PreviewState::Paused {
            return Ok(());
        }

        if let Some(ref mut session) = self.session {
            session.resume();
        }

        self.state = PreviewState::Running;
        Ok(())
    }

    /// 从指定节点重新开始预览
    ///
    /// 停止当前预览会话，创建从指定节点开始的新会话，并启动预览。
    pub fn restart_from_node(&mut self, node_id: String, context: &mut EditorContext) -> GResult<()> {
        self.start_node_id = Some(node_id.clone());
        if let Some(ref mut session) = self.session {
            session.stop();
        }
        let new_session = PreviewSession::with_start_node(node_id, None)?;
        self.session = Some(new_session);
        if let Some(ref mut session) = self.session {
            session.start();
        }
        self.state = PreviewState::Running;
        context.events_mut().publish(EditorEvent::PreviewStarted);
        Ok(())
    }

    /// 触发 HMR 热更新
    ///
    /// 通过 HmrManager 检测文件变更并执行热重载流程。
    /// 脚本变更触发重编译和状态迁移，资源变更触发重加载和缓存更新。
    /// 热重载失败时自动回滚到上一次稳定模块。
    pub fn trigger_hmr_reload(&mut self, context: &mut EditorContext) -> GResult<()> {
        if !self.hmr_enabled {
            return Ok(());
        }

        self.hmr_status = HmrStatus::Detecting;

        let changes = self.hmr_manager.detect_changes()?;
        if changes.is_empty() {
            self.hmr_status = HmrStatus::Idle;
            return Ok(());
        }

        self.hmr_status = HmrStatus::Reloading;

        let changed_paths: Vec<String> = changes.iter().map(|c| c.path.clone()).collect();
        context.events_mut().publish(EditorEvent::HmrReloadTriggered { changed_files: changed_paths });

        let results = self.hmr_manager.process_changes()?;

        let has_failure = results.iter().any(|r| matches!(r, HmrReloadResult::Failed { .. }));
        if has_failure {
            self.hmr_status = HmrStatus::Failed;
            context.events_mut().publish(EditorEvent::HmrReloadCompleted { success: false });
        }
        else {
            self.hmr_status = HmrStatus::Success;
            context.events_mut().publish(EditorEvent::HmrReloadCompleted { success: true });
        }

        Ok(())
    }

    /// 切换 HMR 启用状态
    ///
    /// 切换 HMR 热更新的启用/禁用状态。
    /// 启用时自动启动文件监视器，禁用时停止监视器。
    pub fn toggle_hmr(&mut self) {
        self.hmr_enabled = !self.hmr_enabled;
        if self.hmr_enabled {
            self.hmr_manager.set_enabled(true);
            let _ = self.hmr_manager.start_watching();
        }
        else {
            self.hmr_manager.set_enabled(false);
            let _ = self.hmr_manager.stop_watching();
            self.hmr_status = HmrStatus::Idle;
        }
    }

    /// 设置当前场景名称
    ///
    /// 更新状态栏中显示的场景名称。
    pub fn set_scene_name(&mut self, name: String) {
        self.scene_name = name;
    }

    /// 设置当前场景对象数量
    ///
    /// 更新状态栏中显示的对象数量。
    pub fn set_object_count(&mut self, count: usize) {
        self.object_count = count;
    }

    /// 更新 FPS 计数
    ///
    /// 根据帧间隔时间更新 FPS 值。
    pub fn update_fps(&mut self, delta_seconds: f32) {
        if delta_seconds > 0.0 {
            let current_fps = 1.0 / delta_seconds;
            self.fps_counter = self.fps_counter * 0.9 + current_fps * 0.1;
        }
    }

    /// 推进预览一帧
    ///
    /// 如果预览正在运行，推进游戏逻辑、渲染到纹理并更新帧计数。
    pub fn tick(&mut self) -> GResult<()> {
        if self.state == PreviewState::Running {
            if let Some(ref mut session) = self.session {
                let rendered = session.tick_and_render()?;
                if rendered {
                    if let Some(texture_id) = session.render_texture_id() {
                        self.viewport_texture_id = Some(texture_id);
                    }
                }
            }
            self.frame_count += 1;
        }
        Ok(())
    }

    /// 调整视口大小
    ///
    /// 通知视口渲染器更新渲染纹理尺寸。
    pub fn resize_viewport(&mut self, width: u32, height: u32) {
        self.viewport_renderer.resize(width, height);
    }

    /// 获取视口渲染器的引用
    pub fn viewport_renderer(&self) -> &ViewportRenderer {
        &self.viewport_renderer
    }

    /// 获取视口渲染器的可变引用
    pub fn viewport_renderer_mut(&mut self) -> &mut ViewportRenderer {
        &mut self.viewport_renderer
    }

    /// 获取 HMR 管理器的引用
    pub fn hmr_manager(&self) -> &HmrManager {
        &self.hmr_manager
    }

    /// 获取 HMR 管理器的可变引用
    pub fn hmr_manager_mut(&mut self) -> &mut HmrManager {
        &mut self.hmr_manager
    }

    /// 添加 HMR 监视路径
    ///
    /// 将项目目录添加到 HMR 文件监视列表中。
    pub fn add_hmr_watch_path(&mut self, path: String) {
        self.hmr_manager.add_watch_path(path);
    }

    /// 获取 HMR 状态
    ///
    /// 返回当前 HMR 热更新的状态。
    pub fn hmr_status(&self) -> HmrStatus {
        self.hmr_status
    }

    /// 设置 HMR 状态
    ///
    /// 更新 HMR 热更新的状态。
    pub fn set_hmr_status(&mut self, status: HmrStatus) {
        self.hmr_status = status;
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
    fn build_ui(&mut self, _context: &mut EditorContext, ui_tree: &mut UiTree) -> Option<UiNodeId> {
        let root_id = ui_tree.create_node(
            "preview_root",
            Style::new()
                .with_layout(
                    LayoutStyle::new()
                        .with_direction(FlexDirection::Column)
                        .with_width(SizeValue::Percent(1.0))
                        .with_height(SizeValue::Percent(1.0)),
                )
                .with_background_color(Color::new(0.12, 0.12, 0.12, 1.0))
                .with_overflow(Overflow::Clip),
            UiNodeData::Container,
        );

        let toolbar_id = ui_tree.create_node(
            "preview_toolbar",
            Style::new()
                .with_layout(
                    LayoutStyle::new()
                        .with_direction(FlexDirection::Row)
                        .with_width(SizeValue::Percent(1.0))
                        .with_height(SizeValue::Px(36.0))
                        .with_padding(4.0)
                        .with_gap(4.0)
                        .with_align_items(FlexAlign::Center),
                )
                .with_background_color(Color::new(0.15, 0.15, 0.16, 1.0))
                .with_border_color(Color::new(0.24, 0.24, 0.24, 1.0))
                .with_border_width(1.0),
            UiNodeData::Container,
        );
        ui_tree.add_child(root_id, toolbar_id);

        let play_btn_id = ui_tree.create_node(
            "btn_play",
            Style::new()
                .with_layout(LayoutStyle::new().with_height(SizeValue::Px(28.0)).with_padding(4.0))
                .with_background_color(Color::new(0.54, 0.82, 0.52, 1.0))
                .with_corner_radius(2.0)
                .with_font(FontStyle::new().with_size(12.0).with_color(Color::WHITE)),
            UiNodeData::Custom { kind: "PreviewBtn:Play".to_string() },
        );
        ui_tree.add_child(toolbar_id, play_btn_id);
        if self.state != PreviewState::Idle {
            if let Some(node) = ui_tree.get_mut(play_btn_id) {
                node.visible = false;
            }
        }

        let pause_btn_id = ui_tree.create_node(
            "btn_pause",
            Style::new()
                .with_layout(LayoutStyle::new().with_height(SizeValue::Px(28.0)).with_padding(4.0))
                .with_background_color(Color::new(0.80, 0.65, 0.0, 1.0))
                .with_corner_radius(2.0)
                .with_font(FontStyle::new().with_size(12.0).with_color(Color::WHITE)),
            UiNodeData::Custom { kind: "PreviewBtn:Pause".to_string() },
        );
        ui_tree.add_child(toolbar_id, pause_btn_id);
        if self.state != PreviewState::Running {
            if let Some(node) = ui_tree.get_mut(pause_btn_id) {
                node.visible = false;
            }
        }

        let resume_btn_id = ui_tree.create_node(
            "btn_resume",
            Style::new()
                .with_layout(LayoutStyle::new().with_height(SizeValue::Px(28.0)).with_padding(4.0))
                .with_background_color(Color::new(0.54, 0.82, 0.52, 1.0))
                .with_corner_radius(2.0)
                .with_font(FontStyle::new().with_size(12.0).with_color(Color::WHITE)),
            UiNodeData::Custom { kind: "PreviewBtn:Resume".to_string() },
        );
        ui_tree.add_child(toolbar_id, resume_btn_id);
        if self.state != PreviewState::Paused {
            if let Some(node) = ui_tree.get_mut(resume_btn_id) {
                node.visible = false;
            }
        }

        let stop_btn_id = ui_tree.create_node(
            "btn_stop",
            Style::new()
                .with_layout(LayoutStyle::new().with_height(SizeValue::Px(28.0)).with_padding(4.0))
                .with_background_color(Color::new(0.96, 0.28, 0.28, 1.0))
                .with_corner_radius(2.0)
                .with_font(FontStyle::new().with_size(12.0).with_color(Color::WHITE)),
            UiNodeData::Custom { kind: "PreviewBtn:Stop".to_string() },
        );
        ui_tree.add_child(toolbar_id, stop_btn_id);
        if self.state == PreviewState::Idle {
            if let Some(node) = ui_tree.get_mut(stop_btn_id) {
                node.visible = false;
            }
        }

        let hmr_label = if self.hmr_enabled { "HMR: ON" } else { "HMR: OFF" };
        let hmr_id = ui_tree.create_node(
            "hmr_toggle",
            Style::new()
                .with_layout(LayoutStyle::new().with_height(SizeValue::Px(28.0)).with_padding(4.0))
                .with_corner_radius(2.0)
                .with_font(FontStyle::new().with_size(12.0).with_color(Color::new(0.52, 0.52, 0.52, 1.0))),
            UiNodeData::Custom { kind: "PreviewBtn:HmrToggle".to_string() },
        );
        ui_tree.add_child(toolbar_id, hmr_id);

        let fps_text = format!("FPS: {:.0}", self.fps_counter);
        let fps_id = ui_tree.create_node(
            "fps_display",
            Style::new()
                .with_layout(LayoutStyle::new().with_height(SizeValue::Px(28.0)).with_padding(4.0).with_margin_left(8.0))
                .with_font(FontStyle::new().with_size(12.0).with_color(Color::new(0.52, 0.52, 0.52, 1.0))),
            UiNodeData::Text { content: fps_text },
        );
        ui_tree.add_child(toolbar_id, fps_id);

        let viewport_style = Style::new()
            .with_layout(
                LayoutStyle::new()
                    .with_direction(FlexDirection::Column)
                    .with_width(SizeValue::Percent(1.0))
                    .with_height(SizeValue::Percent(1.0)),
            )
            .with_background_color(Color::new(0.1, 0.1, 0.1, 1.0));

        let viewport_data = if let Some(texture_id) = self.viewport_texture_id {
            UiNodeData::Image { texture_id: Some(texture_id), size: Some((800.0, 600.0)) }
        }
        else {
            UiNodeData::Custom { kind: "SceneView".to_string() }
        };
        let viewport_id = ui_tree.create_node("preview_viewport", viewport_style, viewport_data);
        ui_tree.add_child(root_id, viewport_id);

        let statusbar_id = ui_tree.create_node(
            "preview_statusbar",
            Style::new()
                .with_layout(
                    LayoutStyle::new()
                        .with_direction(FlexDirection::Row)
                        .with_width(SizeValue::Percent(1.0))
                        .with_height(SizeValue::Px(22.0))
                        .with_padding(4.0)
                        .with_gap(8.0)
                        .with_align_items(FlexAlign::Center),
                )
                .with_background_color(Color::new(0.0, 0.48, 0.8, 1.0)),
            UiNodeData::Container,
        );
        ui_tree.add_child(root_id, statusbar_id);

        let scene_label = format!("场景: {}", self.scene_name);
        let scene_id = ui_tree.create_node(
            "status_scene",
            Style::new().with_font(FontStyle::new().with_size(10.0).with_color(Color::WHITE)),
            UiNodeData::Text { content: scene_label },
        );
        ui_tree.add_child(statusbar_id, scene_id);

        let obj_label = format!("对象数: {}", self.object_count);
        let obj_id = ui_tree.create_node(
            "status_objects",
            Style::new().with_font(FontStyle::new().with_size(10.0).with_color(Color::WHITE)),
            UiNodeData::Text { content: obj_label },
        );
        ui_tree.add_child(statusbar_id, obj_id);

        let fps_label = format!("FPS: {:.0}", self.fps_counter);
        let fps_status_id = ui_tree.create_node(
            "status_fps",
            Style::new().with_font(FontStyle::new().with_size(10.0).with_color(Color::WHITE)),
            UiNodeData::Text { content: fps_label },
        );
        ui_tree.add_child(statusbar_id, fps_status_id);

        let hmr_status_label = format!("HMR: {}", self.hmr_status.display_text());
        let hmr_status_color = self.hmr_status.display_color();
        let hmr_status_id = ui_tree.create_node(
            "status_hmr",
            Style::new().with_font(FontStyle::new().with_size(10.0).with_color(hmr_status_color)),
            UiNodeData::Text { content: hmr_status_label },
        );
        ui_tree.add_child(statusbar_id, hmr_status_id);

        Some(root_id)
    }

    /// 处理编辑器事件
    fn on_event(&mut self, event: &EditorEvent, context: &mut EditorContext) {
        match event {
            EditorEvent::MouseDown { button, position } => {
                let _ = (button, position, context);
            }
            EditorEvent::PreviewStarted => {}
            EditorEvent::PreviewStopped => {
                self.state = PreviewState::Idle;
            }
            EditorEvent::PreviewPaused => {
                self.state = PreviewState::Paused;
            }
            EditorEvent::PreviewResumed => {
                self.state = PreviewState::Running;
            }
            EditorEvent::HmrReloadCompleted { success } => {
                self.hmr_status = if *success { HmrStatus::Success } else { HmrStatus::Failed };
            }
            _ => {}
        }
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
