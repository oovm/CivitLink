//! 基础场景视图核心实现

use std::cell::RefCell;
use std::rc::Rc;

use gg_core::GResult;
use gg_ecs::Entity;
use gg_editor_shell::{EditorContext, EditorEvent, Key};
use gg_render::Color;

use crate::components::{RectRenderer, SpriteRenderer, Transform2D};
use crate::viewport::ViewportState;
use crate::world::GameWorld;

use super::clipboard::{ClipboardEntity, EntitySnapshot};
use super::context_menu::SceneContextMenu;
use super::types::{
    BASE_GRID_SPACING, CLIPBOARD_OFFSET, GizmoState, SceneEntity, SceneEntityKind, SelectionBox, TransformGizmo, TransformKind,
    TransformValue, ZOOM_STEP, entity_to_u64, u64_to_entity,
};

/// 基础场景视图
///
/// 提供场景视图的默认实现，包括视口管理、实体选中、网格显示、
/// 变换撤销/重做、实体删除/创建、复制粘贴和右键上下文菜单功能。
/// 可作为具体场景视图的基类使用。
pub struct BaseSceneView {
    /// 是否可见
    pub(crate) visible: bool,
    /// 视口状态
    pub(crate) viewport: ViewportState,
    /// 选中的实体列表
    pub(crate) selected_entities: Vec<Entity>,
    /// 网格是否可见
    pub(crate) grid_visible: bool,
    /// 场景实体渲染数据列表
    pub(crate) scene_entities: Vec<SceneEntity>,
    /// 事件缓冲：通过事件接收的选中实体
    pub(crate) incoming_selected: Rc<RefCell<Vec<u64>>>,
    /// 事件缓冲：是否收到取消选中事件
    pub(crate) incoming_deselected: Rc<RefCell<bool>>,
    /// 待发布的选中事件
    pub(crate) pending_select_events: Vec<u64>,
    /// 待发布的取消选中事件
    pub(crate) pending_deselect_event: bool,
    /// 是否正在中键平移
    pub(crate) is_panning: bool,
    /// 平移拖拽起始位置
    pub(crate) pan_start: Option<(f32, f32)>,
    /// 最后已知的鼠标位置，用于缩放中心
    pub(crate) last_mouse_pos: Option<(f32, f32)>,
    /// 是否正在拖拽实体
    pub(crate) is_dragging: bool,
    /// 拖拽起始鼠标屏幕坐标
    pub(crate) drag_start_screen: (f32, f32),
    /// 拖拽起始时实体的世界坐标
    pub(crate) drag_entity_start_pos: (f32, f32),
    /// 当前被拖拽的实体
    pub(crate) dragged_entity: Option<Entity>,
    /// 场景渲染目标名称
    pub(crate) render_target_name: String,
    /// 待提交的场景绘制命令
    pub(crate) render_commands: Vec<gg_render::DrawCommand>,
    /// 变换工具模式
    pub(crate) transform_gizmo: TransformGizmo,
    /// 变换工具状态
    pub(crate) gizmo_state: GizmoState,
    /// 框选状态
    pub(crate) selection_box: SelectionBox,
    /// 是否正在使用变换工具拖拽
    pub(crate) is_gizmo_dragging: bool,
    /// Ctrl 键是否按下
    pub(crate) ctrl_pressed: bool,
    /// Shift 键是否按下
    pub(crate) shift_pressed: bool,
    /// Alt 键是否按下
    pub(crate) alt_pressed: bool,
    /// 剪贴板中的实体数据列表
    pub(crate) clipboard: Vec<ClipboardEntity>,
    /// 右键上下文菜单
    pub(crate) context_menu: SceneContextMenu,
    /// Gizmo 拖拽开始时实体的变换旧值
    pub(crate) gizmo_drag_old_value: Option<TransformValue>,
}

impl BaseSceneView {
    /// 创建新的基础场景视图
    pub fn new() -> Self {
        Self {
            visible: true,
            viewport: ViewportState::new(),
            selected_entities: Vec::new(),
            grid_visible: true,
            scene_entities: Vec::new(),
            incoming_selected: Rc::new(RefCell::new(Vec::new())),
            incoming_deselected: Rc::new(RefCell::new(false)),
            pending_select_events: Vec::new(),
            pending_deselect_event: false,
            is_panning: false,
            pan_start: None,
            last_mouse_pos: None,
            is_dragging: false,
            drag_start_screen: (0.0, 0.0),
            drag_entity_start_pos: (0.0, 0.0),
            dragged_entity: None,
            render_target_name: "scene_view".to_string(),
            render_commands: Vec::new(),
            transform_gizmo: TransformGizmo::Translate,
            gizmo_state: GizmoState::new(),
            selection_box: SelectionBox::new(),
            is_gizmo_dragging: false,
            ctrl_pressed: false,
            shift_pressed: false,
            alt_pressed: false,
            clipboard: Vec::new(),
            context_menu: SceneContextMenu::new(),
            gizmo_drag_old_value: None,
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

    /// 获取渲染目标名称
    pub fn render_target_name(&self) -> &str {
        &self.render_target_name
    }

    /// 设置渲染目标名称
    pub fn set_render_target_name(&mut self, name: String) {
        self.render_target_name = name;
    }

    /// 获取选中的实体列表
    pub fn selected_entities(&self) -> &[Entity] {
        &self.selected_entities
    }

    /// 获取场景实体渲染数据列表
    pub fn scene_entities(&self) -> &[SceneEntity] {
        &self.scene_entities
    }

    /// 设置场景实体渲染数据
    pub fn set_scene_entities(&mut self, entities: Vec<SceneEntity>) {
        self.scene_entities = entities;
    }

    /// 选中实体
    pub fn select_entity(&mut self, entity: Entity) {
        if !self.selected_entities.contains(&entity) {
            self.selected_entities.push(entity);
            self.pending_select_events.push(entity_to_u64(entity));
        }
    }

    /// 取消所有选中
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
    pub fn pan_viewport(&mut self, delta: (f32, f32)) {
        self.viewport.pan(delta);
    }

    /// 缩放视口
    pub fn zoom_viewport(&mut self, factor: f32, center: (f32, f32)) {
        self.viewport.zoom_to(factor, center);
    }

    /// 重置视口到默认状态
    pub fn reset_viewport(&mut self) {
        self.viewport = ViewportState::new();
    }

    /// 放大视口
    pub fn zoom_in(&mut self) {
        let center = (self.viewport.size.0 / 2.0, self.viewport.size.1 / 2.0);
        self.viewport.zoom_to(ZOOM_STEP, center);
    }

    /// 缩小视口
    pub fn zoom_out(&mut self) {
        let center = (self.viewport.size.0 / 2.0, self.viewport.size.1 / 2.0);
        self.viewport.zoom_to(1.0 / ZOOM_STEP, center);
    }

    /// 切换网格可见性
    pub fn toggle_grid(&mut self) {
        self.grid_visible = !self.grid_visible;
    }

    /// 处理鼠标中键按下
    pub fn handle_middle_button_down(&mut self, pos: (f32, f32)) {
        self.is_panning = true;
        self.pan_start = Some(pos);
    }

    /// 处理鼠标中键释放
    pub fn handle_middle_button_up(&mut self) {
        self.is_panning = false;
        self.pan_start = None;
    }

    /// 处理鼠标移动
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
    pub fn handle_scroll(&mut self, delta: f32, mouse_pos: (f32, f32)) {
        let factor = if delta > 0.0 { ZOOM_STEP } else { 1.0 / ZOOM_STEP };
        self.viewport.zoom_to(factor, mouse_pos);
        self.last_mouse_pos = Some(mouse_pos);
    }

    /// 处理点击选中实体
    pub fn handle_click(&mut self, screen_pos: (f32, f32), context: &mut EditorContext) {
        let world_pos = self.viewport.screen_to_world(screen_pos);
        let world = context.world();
        let hit = Self::hit_test_world(world, world_pos);

        if let Some((entity, _, _)) = hit {
            self.select_entity(entity);
            context.events_mut().publish(EditorEvent::EntitySelected { entity: entity_to_u64(entity) });
        }
        else {
            self.deselect_all();
            context.events_mut().publish(EditorEvent::EntityDeselected);
        }
    }

    /// 在 GameWorld 中进行点击命中测试
    pub(crate) fn hit_test_world(world: &GameWorld, world_pos: (f32, f32)) -> Option<(Entity, f32, f32)> {
        let query = world.query::<Transform2D>();
        for (entity, transform) in query {
            let (width, height) = Self::get_entity_size(world, entity);
            if world_pos.0 >= transform.x
                && world_pos.0 <= transform.x + width
                && world_pos.1 >= transform.y
                && world_pos.1 <= transform.y + height
            {
                return Some((entity, transform.x, transform.y));
            }
        }
        None
    }

    /// 框选命中测试
    pub(crate) fn hit_test_box(
        world: &GameWorld,
        screen_start: (f32, f32),
        screen_end: (f32, f32),
        viewport: &ViewportState,
    ) -> Vec<Entity> {
        let world_start = viewport.screen_to_world(screen_start);
        let world_end = viewport.screen_to_world(screen_end);

        let min_x = world_start.0.min(world_end.0);
        let min_y = world_start.1.min(world_end.1);
        let max_x = world_start.0.max(world_end.0);
        let max_y = world_start.1.max(world_end.1);

        let mut result = Vec::new();
        let query = world.query::<Transform2D>();
        for (entity, transform) in query {
            let (width, height) = Self::get_entity_size(world, entity);
            let entity_min_x = transform.x;
            let entity_min_y = transform.y;
            let entity_max_x = transform.x + width;
            let entity_max_y = transform.y + height;

            if entity_min_x <= max_x && entity_max_x >= min_x && entity_min_y <= max_y && entity_max_y >= min_y {
                result.push(entity);
            }
        }
        result
    }

    /// 计算旋转缩放后的矩形四个角点
    pub(crate) fn compute_transformed_corners(
        screen_x: f32,
        screen_y: f32,
        screen_w: f32,
        screen_h: f32,
        rotation: f32,
        scale_x: f32,
        scale_y: f32,
    ) -> [(f32, f32); 4] {
        let cx = screen_x + screen_w / 2.0;
        let cy = screen_y + screen_h / 2.0;
        let half_w = screen_w * scale_x / 2.0;
        let half_h = screen_h * scale_y / 2.0;

        let corners = [(-half_w, -half_h), (half_w, -half_h), (half_w, half_h), (-half_w, half_h)];

        let cos_r = rotation.cos();
        let sin_r = rotation.sin();

        corners.map(|(x, y)| {
            let rx = x * cos_r - y * sin_r;
            let ry = x * sin_r + y * cos_r;
            (cx + rx, cy + ry)
        })
    }

    /// 获取实体在场景中的渲染尺寸
    pub(crate) fn get_entity_size(world: &GameWorld, entity: Entity) -> (f32, f32) {
        if let Some(sprite) = world.get_component::<SpriteRenderer>(entity) {
            (sprite.width, sprite.height)
        }
        else if let Some(rect) = world.get_component::<RectRenderer>(entity) {
            (rect.width, rect.height)
        }
        else {
            (50.0, 50.0)
        }
    }

    /// 设置变换工具模式
    pub fn set_transform_mode(&mut self, mode: TransformGizmo) {
        self.transform_gizmo = mode;
        self.gizmo_state.mode = mode;
    }

    /// 获取当前变换工具模式
    pub fn transform_mode(&self) -> TransformGizmo {
        self.transform_gizmo
    }

    /// 追加选中实体
    pub fn select_entity_multi(&mut self, entity: Entity) {
        if !self.selected_entities.contains(&entity) {
            self.selected_entities.push(entity);
            self.pending_select_events.push(entity_to_u64(entity));
        }
    }

    /// 聚焦选中实体
    pub fn frame_selection(&mut self) {
        if self.selected_entities.is_empty() {
            return;
        }

        let mut min_x = f32::MAX;
        let mut min_y = f32::MAX;
        let mut max_x = f32::MIN;
        let mut max_y = f32::MIN;

        for &entity in &self.selected_entities {
            if let Some(se) = self.scene_entities.iter().find(|e| e.entity == entity) {
                min_x = min_x.min(se.world_x);
                min_y = min_y.min(se.world_y);
                max_x = max_x.max(se.world_x + se.width);
                max_y = max_y.max(se.world_y + se.height);
            }
        }

        if min_x == f32::MAX {
            return;
        }

        let center_x = (min_x + max_x) / 2.0;
        let center_y = (min_y + max_y) / 2.0;
        let bbox_w = max_x - min_x;
        let bbox_h = max_y - min_y;

        let zoom = if bbox_w > 0.0 && bbox_h > 0.0 {
            let zoom_x = self.viewport.size.0 / (bbox_w * 1.5);
            let zoom_y = self.viewport.size.1 / (bbox_h * 1.5);
            zoom_x.min(zoom_y).clamp(0.1, 10.0)
        }
        else {
            1.0
        };

        self.viewport.offset = (center_x - self.viewport.size.0 / (2.0 * zoom), center_y - self.viewport.size.1 / (2.0 * zoom));
        self.viewport.zoom = zoom;
    }
}

impl Default for BaseSceneView {
    fn default() -> Self {
        Self::new()
    }
}
