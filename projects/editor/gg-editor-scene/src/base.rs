//! 基础场景视图实现

use std::{cell::RefCell, rc::Rc};

use gg_core::GResult;
use gg_editor_shell::{EditorContext, EditorEvent, EditorPanel, PanelLayoutHint, PanelPosition};
use gg_ui::{Style, UiNodeData, UiTree};

use crate::{view::SceneView, viewport::ViewportState};

/// 缩放步进因子
const ZOOM_STEP: f32 = 1.1;

/// 基础场景视图
///
/// 提供场景视图的默认实现，包括视口管理、实体选中和网格显示功能。
/// 可作为具体场景视图的基类使用。
pub struct BaseSceneView {
    /// 是否可见
    visible: bool,
    /// 视口状态
    viewport: ViewportState,
    /// 选中的实体列表
    selected_entities: Vec<u64>,
    /// 网格是否可见
    grid_visible: bool,
    /// 事件缓冲：通过事件接收的选中实体
    incoming_selected: Rc<RefCell<Vec<u64>>>,
    /// 事件缓冲：是否收到取消选中事件
    incoming_deselected: Rc<RefCell<bool>>,
    /// 待发布的选中事件
    pending_select_events: Vec<u64>,
    /// 待发布的取消选中事件
    pending_deselect_event: bool,
    /// 是否正在中键平移
    is_panning: bool,
    /// 平移拖拽起始位置
    pan_start: Option<(f32, f32)>,
    /// 最后已知的鼠标位置，用于缩放中心
    last_mouse_pos: Option<(f32, f32)>,
}

impl BaseSceneView {
    /// 创建新的基础场景视图
    pub fn new() -> Self {
        Self {
            visible: true,
            viewport: ViewportState::new(),
            selected_entities: Vec::new(),
            grid_visible: true,
            incoming_selected: Rc::new(RefCell::new(Vec::new())),
            incoming_deselected: Rc::new(RefCell::new(false)),
            pending_select_events: Vec::new(),
            pending_deselect_event: false,
            is_panning: false,
            pan_start: None,
            last_mouse_pos: None,
        }
    }

    /// 获取视口状态引用
    pub fn viewport(&self) -> &ViewportState {
        &self.viewport
    }

    /// 获取视口状态可变引用
    pub fn viewport_mut(&mut self) -> &mut ViewportState {
        &mut self.viewport
    }

    /// 获取选中的实体列表
    pub fn selected_entities(&self) -> &[u64] {
        &self.selected_entities
    }

    /// 选中实体
    ///
    /// 将实体添加到选中列表，并在下次渲染时发布 `EntitySelected` 事件。
    pub fn select_entity(&mut self, entity: u64) {
        if !self.selected_entities.contains(&entity) {
            self.selected_entities.push(entity);
            self.pending_select_events.push(entity);
        }
    }

    /// 取消所有选中
    ///
    /// 清空选中列表，并在下次渲染时发布 `EntityDeselected` 事件。
    pub fn deselect_all(&mut self) {
        self.selected_entities.clear();
        self.pending_deselect_event = true;
    }

    /// 网格是否可见
    pub fn grid_visible(&self) -> bool {
        self.grid_visible
    }

    /// 设置网格可见性
    pub fn set_grid_visible(&mut self, visible: bool) {
        self.grid_visible = visible;
    }

    /// 平移视口
    ///
    /// 将指定增量传递给视口的 `pan` 方法。
    pub fn pan_viewport(&mut self, delta: (f32, f32)) {
        self.viewport.pan(delta);
    }

    /// 缩放视口
    ///
    /// 将指定缩放因子和中心点传递给视口的 `zoom_to` 方法。
    pub fn zoom_viewport(&mut self, factor: f32, center: (f32, f32)) {
        self.viewport.zoom_to(factor, center);
    }

    /// 处理鼠标中键按下
    ///
    /// 开始视口平移操作，记录起始位置。
    pub fn handle_middle_button_down(&mut self, pos: (f32, f32)) {
        self.is_panning = true;
        self.pan_start = Some(pos);
    }

    /// 处理鼠标中键释放
    ///
    /// 停止视口平移操作。
    pub fn handle_middle_button_up(&mut self) {
        self.is_panning = false;
        self.pan_start = None;
    }

    /// 处理鼠标移动
    ///
    /// 如果正在平移，根据鼠标位移更新视口偏移。
    pub fn handle_mouse_move(&mut self, pos: (f32, f32)) {
        if self.is_panning {
            if let Some(start) = self.pan_start {
                let dx = pos.0 - start.0;
                let dy = pos.1 - start.1;
                self.viewport.pan((-dx / self.viewport.zoom, -dy / self.viewport.zoom));
                self.pan_start = Some(pos);
            }
        }
        self.last_mouse_pos = Some(pos);
    }

    /// 处理滚轮缩放
    ///
    /// 以鼠标位置为中心进行缩放。`delta` 为正表示放大，为负表示缩小。
    pub fn handle_scroll(&mut self, delta: f32, mouse_pos: (f32, f32)) {
        let factor = if delta > 0.0 { ZOOM_STEP } else { 1.0 / ZOOM_STEP };
        self.viewport.zoom_to(factor, mouse_pos);
        self.last_mouse_pos = Some(mouse_pos);
    }

    /// 处理点击选中实体
    ///
    /// 将屏幕坐标转换为世界坐标，查找实体并发布 `EntitySelected` 事件。
    /// 当前使用占位实体 ID (0)，待 ECS 空间查询可用后替换为实际查找逻辑。
    pub fn handle_click(&mut self, screen_pos: (f32, f32), context: &mut EditorContext) {
        let _world_pos = self.viewport.screen_to_world(screen_pos);
        let placeholder_entity: u64 = 0;
        self.select_entity(placeholder_entity);
        context.events_mut().publish(EditorEvent::EntitySelected { entity: placeholder_entity });
    }

    /// 渲染选中实体的叠加高亮
    ///
    /// 占位实现：实际渲染将使用 `viewport.world_to_screen()` 将实体包围盒
    /// 转换为屏幕坐标并绘制高亮边框。
    pub fn render_selection_overlay(&self, _context: &mut EditorContext) -> GResult<()> {
        Ok(())
    }

    /// 构建缩放指示器文本
    fn zoom_indicator_text(&self) -> String {
        format!("{:.0}%", self.viewport.zoom * 100.0)
    }
}

impl Default for BaseSceneView {
    fn default() -> Self {
        Self::new()
    }
}

impl EditorPanel for BaseSceneView {
    fn name(&self) -> &str {
        "Scene View"
    }

    fn is_visible(&self) -> bool {
        self.visible
    }

    fn set_visible(&mut self, visible: bool) {
        self.visible = visible;
    }

    fn on_register(&mut self, context: &mut EditorContext) {
        let incoming_selected = Rc::clone(&self.incoming_selected);
        context.events_mut().subscribe(Box::new(move |event| {
            if let EditorEvent::EntitySelected { entity } = event {
                incoming_selected.borrow_mut().push(*entity);
            }
        }));

        let incoming_deselected = Rc::clone(&self.incoming_deselected);
        context.events_mut().subscribe(Box::new(move |event| {
            if matches!(event, EditorEvent::EntityDeselected) {
                *incoming_deselected.borrow_mut() = true;
            }
        }));
    }

    /// 构建场景视图面板 UI 节点树
    fn build_ui(&mut self, context: &mut EditorContext, ui_tree: &mut UiTree) -> GResult<()> {
        for entity in self.pending_select_events.drain(..) {
            context.events_mut().publish(EditorEvent::EntitySelected { entity });
        }
        if self.pending_deselect_event {
            context.events_mut().publish(EditorEvent::EntityDeselected);
            self.pending_deselect_event = false;
        }

        if *self.incoming_deselected.borrow() {
            self.selected_entities.clear();
            *self.incoming_deselected.borrow_mut() = false;
        }
        let mut incoming = self.incoming_selected.borrow_mut();
        for entity in incoming.drain(..) {
            if !self.selected_entities.contains(&entity) {
                self.selected_entities.push(entity);
            }
        }
        drop(incoming);

        let viewport_id = ui_tree.create_node("scene_viewport", Style::new(), UiNodeData::Container);

        let zoom_text = self.zoom_indicator_text();
        let zoom_indicator_id = ui_tree.create_node("zoom_indicator", Style::new(), UiNodeData::Text { content: zoom_text });

        let toolbar_id = ui_tree.create_node("scene_toolbar", Style::new(), UiNodeData::Container);

        let toggle_grid_id =
            ui_tree.create_node("btn_toggle_grid", Style::new(), UiNodeData::Custom { kind: "button".to_string() });

        let reset_view_id =
            ui_tree.create_node("btn_reset_view", Style::new(), UiNodeData::Custom { kind: "button".to_string() });

        let zoom_in_id = ui_tree.create_node("btn_zoom_in", Style::new(), UiNodeData::Custom { kind: "button".to_string() });

        let zoom_out_id = ui_tree.create_node("btn_zoom_out", Style::new(), UiNodeData::Custom { kind: "button".to_string() });

        ui_tree.add_child(toolbar_id, toggle_grid_id);
        ui_tree.add_child(toolbar_id, reset_view_id);
        ui_tree.add_child(toolbar_id, zoom_in_id);
        ui_tree.add_child(toolbar_id, zoom_out_id);

        ui_tree.add_child(viewport_id, zoom_indicator_id);
        ui_tree.add_child(viewport_id, toolbar_id);

        ui_tree.set_root(viewport_id);

        self.render_selection_overlay(context)?;

        Ok(())
    }

    fn layout_hint(&self) -> PanelLayoutHint {
        PanelLayoutHint {
            position: PanelPosition::Center,
            preferred_size: Some((800.0, 600.0)),
            min_size: Some((400.0, 300.0)),
        }
    }
}

impl SceneView for BaseSceneView {}
