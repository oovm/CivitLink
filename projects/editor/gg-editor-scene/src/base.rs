//! 基础场景视图实现

use std::{cell::RefCell, rc::Rc};

use gg_core::GResult;
use gg_editor_shell::{EditorContext, EditorEvent, EditorPanel, PanelLayoutHint, PanelPosition};
use gg_ui::UiTree;

use crate::{view::SceneView, viewport::ViewportState};

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
    fn build_ui(&mut self, context: &mut EditorContext, _ui_tree: &mut UiTree) -> GResult<()> {
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
