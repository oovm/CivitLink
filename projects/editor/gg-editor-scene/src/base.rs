//! 基础场景视图实现

use std::{cell::RefCell, rc::Rc};

use gg_core::GResult;
use gg_ecs::Entity;
use gg_editor_shell::{EditorContext, EditorEvent, EditorPanel, PanelLayoutHint, PanelPosition};
use gg_render::{Color, DrawCommand, Rect, RenderContext};
use gg_ui::{Style, UiNodeData, UiTree};
use gg_world::GameWorld;

use crate::{components::Transform2D, view::SceneView, viewport::ViewportState};
use crate::components::{RectRenderer, SpriteRenderer};

/// 缩放步进因子
const ZOOM_STEP: f32 = 1.1;

/// 基础网格间距（世界坐标单位）
const BASE_GRID_SPACING: f32 = 50.0;

/// 将 ECS Entity 转换为 u64 标识符
///
/// 使用代数高 32 位、索引低 32 位的编码方式，
/// 与 `gg_world::SceneSerializer` 保持一致。
fn entity_to_u64(entity: Entity) -> u64 {
    ((entity.generation() as u64) << 32) | (entity.index() as u64)
}

/// 将 u64 标识符还原为 ECS Entity
///
/// 与 `entity_to_u64` 互逆，从编码中提取索引和代数。
fn u64_to_entity(id: u64) -> Entity {
    let index = (id & 0xFFFFFFFF) as u32;
    let generation = (id >> 32) as u32;
    Entity::new(index, generation)
}

/// 场景实体类型
///
/// 标识场景实体的渲染类型，用于决定绘制命令的生成方式。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SceneEntityKind {
    /// 精灵类型
    ///
    /// 使用纹理渲染的实体，当前以填充矩形作为占位渲染。
    Sprite,
    /// 矩形类型
    ///
    /// 使用纯色矩形渲染的实体。
    Rect,
}

/// 场景实体渲染数据
///
/// 存储场景中单个实体的渲染信息，包括位置、尺寸、颜色和类型。
/// 世界坐标通过 `ViewportState::world_to_screen()` 转换为屏幕坐标后渲染。
#[derive(Debug, Clone)]
pub struct SceneEntity {
    /// ECS 实体标识符
    pub entity: Entity,
    /// 世界坐标 X
    pub world_x: f32,
    /// 世界坐标 Y
    pub world_y: f32,
    /// 宽度（世界坐标单位）
    pub width: f32,
    /// 高度（世界坐标单位）
    pub height: f32,
    /// 填充颜色
    pub color: Color,
    /// 实体渲染类型
    pub kind: SceneEntityKind,
}

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
    selected_entities: Vec<Entity>,
    /// 网格是否可见
    grid_visible: bool,
    /// 场景实体渲染数据列表
    scene_entities: Vec<SceneEntity>,
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
    /// 是否正在拖拽实体
    is_dragging: bool,
    /// 拖拽起始鼠标屏幕坐标
    drag_start_screen: (f32, f32),
    /// 拖拽起始时实体的世界坐标
    drag_entity_start_pos: (f32, f32),
    /// 当前被拖拽的实体
    dragged_entity: Option<Entity>,
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
    pub fn selected_entities(&self) -> &[Entity] {
        &self.selected_entities
    }

    /// 获取场景实体渲染数据列表
    pub fn scene_entities(&self) -> &[SceneEntity] {
        &self.scene_entities
    }

    /// 设置场景实体渲染数据
    ///
    /// 替换当前的场景实体列表，用于外部代码（如编辑器外壳）填充实体数据。
    pub fn set_scene_entities(&mut self, entities: Vec<SceneEntity>) {
        self.scene_entities = entities;
    }

    /// 选中实体
    ///
    /// 将实体添加到选中列表，并在下次渲染时发布 `EntitySelected` 事件。
    pub fn select_entity(&mut self, entity: Entity) {
        if !self.selected_entities.contains(&entity) {
            self.selected_entities.push(entity);
            self.pending_select_events.push(entity_to_u64(entity));
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
    pub fn handle_click(&mut self, screen_pos: (f32, f32), context: &mut EditorContext) {
        let world_pos = self.viewport.screen_to_world(screen_pos);
        let world = context.world();
        let hit = Self::hit_test_world(world, world_pos);

        if let Some((entity, _, _)) = hit {
            self.select_entity(entity);
            context.events_mut().publish(EditorEvent::EntitySelected { entity: entity_to_u64(entity) });
        } else {
            self.deselect_all();
            context.events_mut().publish(EditorEvent::EntityDeselected);
        }
    }

    /// 在 GameWorld 中进行点击命中测试
    ///
    /// 遍历拥有 `Transform2D` 组件的实体，检查世界坐标是否落在实体包围盒内。
    /// 返回命中的实体、其世界坐标和尺寸信息。
    fn hit_test_world(world: &GameWorld, world_pos: (f32, f32)) -> Option<(Entity, f32, f32)> {
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

    /// 获取实体在场景中的渲染尺寸
    ///
    /// 优先从 `SpriteRenderer` 获取尺寸，其次从 `RectRenderer` 获取，
    /// 均不存在时返回默认尺寸 (50.0, 50.0)。
    fn get_entity_size(world: &GameWorld, entity: Entity) -> (f32, f32) {
        if let Some(sprite) = world.get_component::<SpriteRenderer>(entity) {
            (sprite.width, sprite.height)
        } else if let Some(rect) = world.get_component::<RectRenderer>(entity) {
            (rect.width, rect.height)
        } else {
            (50.0, 50.0)
        }
    }

    /// 从 GameWorld 查询实体并构建场景实体列表
    ///
    /// 查询所有拥有 `Transform2D` 组件的实体，
    /// 根据是否拥有 `SpriteRenderer` 或 `RectRenderer` 确定渲染类型和尺寸。
    fn build_scene_entities(world: &GameWorld) -> Vec<SceneEntity> {
        let mut entities = Vec::new();
        let query = world.query::<Transform2D>();
        for (entity, transform) in query {
            if let Some(sprite) = world.get_component::<SpriteRenderer>(entity) {
                entities.push(SceneEntity {
                    entity,
                    world_x: transform.x,
                    world_y: transform.y,
                    width: sprite.width,
                    height: sprite.height,
                    color: sprite.color,
                    kind: SceneEntityKind::Sprite,
                });
            } else if let Some(rect) = world.get_component::<RectRenderer>(entity) {
                entities.push(SceneEntity {
                    entity,
                    world_x: transform.x,
                    world_y: transform.y,
                    width: rect.width,
                    height: rect.height,
                    color: rect.color,
                    kind: SceneEntityKind::Rect,
                });
            } else {
                entities.push(SceneEntity {
                    entity,
                    world_x: transform.x,
                    world_y: transform.y,
                    width: 50.0,
                    height: 50.0,
                    color: Color::new(0.5, 0.5, 0.5, 1.0),
                    kind: SceneEntityKind::Rect,
                });
            }
        }
        entities
    }

    /// 渲染场景实体
    ///
    /// 从 GameWorld 查询拥有 Transform2D 组件的实体，
    /// 通过 ViewportState 转换坐标后渲染，最后叠加网格和选中高亮。
    pub fn render_scene(&mut self, context: &mut RenderContext, world: &GameWorld) {
        self.scene_entities = Self::build_scene_entities(world);
        let zoom = self.viewport.zoom;

        for entity in &self.scene_entities {
            let (screen_x, screen_y) =
                self.viewport.world_to_screen((entity.world_x, entity.world_y));
            let screen_w = entity.width * zoom;
            let screen_h = entity.height * zoom;

            match entity.kind {
                SceneEntityKind::Sprite => {
                    context.draw(DrawCommand::Rect {
                        rect: Rect::new(screen_x, screen_y, screen_w, screen_h),
                        color: entity.color,
                        corner_radius: 0.0,
                    });
                }
                SceneEntityKind::Rect => {
                    context.draw(DrawCommand::Rect {
                        rect: Rect::new(screen_x, screen_y, screen_w, screen_h),
                        color: entity.color,
                        corner_radius: 0.0,
                    });
                }
            }
        }

        self.render_grid(context);
        self.render_selection_overlay(context);
    }

    /// 渲染背景网格
    ///
    /// 根据视口偏移和缩放级别绘制网格线。
    /// 网格间距随缩放级别自适应，避免过密或过疏。
    fn render_grid(&self, context: &mut RenderContext) {
        if !self.grid_visible {
            return;
        }

        let zoom = self.viewport.zoom;
        let offset = self.viewport.offset;
        let (vp_width, vp_height) = self.viewport.size;

        let screen_spacing = BASE_GRID_SPACING * zoom;
        let world_step = if screen_spacing < 20.0 {
            BASE_GRID_SPACING * 2.0
        } else if screen_spacing > 200.0 {
            BASE_GRID_SPACING * 0.5
        } else {
            BASE_GRID_SPACING
        };

        let grid_color = Color::new(0.3, 0.3, 0.3, 0.5);

        let start_x = offset.0;
        let start_y = offset.1;
        let end_x = offset.0 + vp_width / zoom;
        let end_y = offset.1 + vp_height / zoom;

        let grid_start_x = (start_x / world_step).floor() * world_step;
        let grid_start_y = (start_y / world_step).floor() * world_step;

        let mut x = grid_start_x;
        while x <= end_x {
            let (screen_x, _) = self.viewport.world_to_screen((x, 0.0));
            context.draw(DrawCommand::Line {
                start: [screen_x, 0.0],
                end: [screen_x, vp_height],
                color: grid_color,
                width: 1.0,
            });
            x += world_step;
        }

        let mut y = grid_start_y;
        while y <= end_y {
            let (_, screen_y) = self.viewport.world_to_screen((0.0, y));
            context.draw(DrawCommand::Line {
                start: [0.0, screen_y],
                end: [vp_width, screen_y],
                color: grid_color,
                width: 1.0,
            });
            y += world_step;
        }
    }

    /// 渲染选中实体高亮叠加层
    ///
    /// 为每个选中的实体绘制蓝色边框矩形，
    /// 边框比实体包围盒略大以形成视觉高亮效果。
    pub fn render_selection_overlay(&self, context: &mut RenderContext) {
        let highlight_color = Color::new(0.2, 0.5, 1.0, 0.8);

        for &entity in &self.selected_entities {
            if let Some(scene_entity) = self.scene_entities.iter().find(|e| e.entity == entity) {
                let (screen_x, screen_y) =
                    self.viewport.world_to_screen((scene_entity.world_x, scene_entity.world_y));
                let screen_w = scene_entity.width * self.viewport.zoom;
                let screen_h = scene_entity.height * self.viewport.zoom;

                context.draw(DrawCommand::Rect {
                    rect: Rect::new(
                        screen_x - 2.0,
                        screen_y - 2.0,
                        screen_w + 4.0,
                        screen_h + 4.0,
                    ),
                    color: highlight_color,
                    corner_radius: 0.0,
                });
            }
        }
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

    /// 处理编辑器事件
    ///
    /// 响应鼠标事件实现实体选择和拖拽移动：
    /// - 左键按下：命中测试选中实体或取消选中
    /// - 鼠标移动：拖拽选中的实体更新其世界坐标
    /// - 左键释放：结束拖拽
    /// - 中键按下/释放/移动：视口平移
    /// - 滚轮：视口缩放
    fn on_event(&mut self, event: &EditorEvent, context: &mut EditorContext) {
        match event {
            EditorEvent::MouseDown { button, position } => {
                match button {
                    gg_editor_shell::MouseButton::Left => {
                        let world_pos = self.viewport.screen_to_world(*position);
                        let world = context.world();
                        let hit = Self::hit_test_world(world, world_pos);

                        if let Some((entity, wx, wy)) = hit {
                            self.select_entity(entity);
                            self.is_dragging = true;
                            self.drag_start_screen = *position;
                            self.drag_entity_start_pos = (wx, wy);
                            self.dragged_entity = Some(entity);
                            context.events_mut().publish(EditorEvent::EntitySelected {
                                entity: entity_to_u64(entity),
                            });
                        } else {
                            self.deselect_all();
                            self.is_dragging = false;
                            self.dragged_entity = None;
                            context.events_mut().publish(EditorEvent::EntityDeselected);
                        }
                    }
                    gg_editor_shell::MouseButton::Middle => {
                        self.handle_middle_button_down(*position);
                    }
                    gg_editor_shell::MouseButton::Right => {}
                }
            }
            EditorEvent::MouseUp { button, .. } => {
                match button {
                    gg_editor_shell::MouseButton::Left => {
                        self.is_dragging = false;
                        self.dragged_entity = None;
                    }
                    gg_editor_shell::MouseButton::Middle => {
                        self.handle_middle_button_up();
                    }
                    gg_editor_shell::MouseButton::Right => {}
                }
            }
            EditorEvent::MouseMove { position } => {
                if self.is_dragging {
                    if let Some(entity) = self.dragged_entity {
                        let (start_wx, start_wy) =
                            self.viewport.screen_to_world(self.drag_start_screen);
                        let (current_wx, current_wy) =
                            self.viewport.screen_to_world(*position);
                        let dx = current_wx - start_wx;
                        let dy = current_wy - start_wy;

                        let new_x = self.drag_entity_start_pos.0 + dx;
                        let new_y = self.drag_entity_start_pos.1 + dy;

                        let world = context.world_mut();
                        if let Some(transform) = world.get_component_mut::<Transform2D>(entity) {
                            transform.x = new_x;
                            transform.y = new_y;
                        }
                    }
                } else if self.is_panning {
                    self.handle_mouse_move(*position);
                }
                self.last_mouse_pos = Some(*position);
            }
            EditorEvent::MouseWheel { delta, position } => {
                self.handle_scroll(delta.1, *position);
            }
            _ => {}
        }
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
        for entity_id in incoming.drain(..) {
            let entity = u64_to_entity(entity_id);
            if !self.selected_entities.contains(&entity) {
                self.selected_entities.push(entity);
            }
        }
        drop(incoming);

        let viewport_style = Style::new();
        let viewport_id =
            ui_tree.create_node("scene_viewport", viewport_style, UiNodeData::Container);

        let zoom_text = self.zoom_indicator_text();
        let zoom_indicator_id =
            ui_tree.create_node("zoom_indicator", Style::new(), UiNodeData::Text { content: zoom_text });

        let toolbar_id = ui_tree.create_node("scene_toolbar", Style::new(), UiNodeData::Container);

        let toggle_grid_id =
            ui_tree.create_node("btn_toggle_grid", Style::new(), UiNodeData::Custom { kind: "button".to_string() });

        let reset_view_id =
            ui_tree.create_node("btn_reset_view", Style::new(), UiNodeData::Custom { kind: "button".to_string() });

        let zoom_in_id =
            ui_tree.create_node("btn_zoom_in", Style::new(), UiNodeData::Custom { kind: "button".to_string() });

        let zoom_out_id =
            ui_tree.create_node("btn_zoom_out", Style::new(), UiNodeData::Custom { kind: "button".to_string() });

        ui_tree.add_child(toolbar_id, toggle_grid_id);
        ui_tree.add_child(toolbar_id, reset_view_id);
        ui_tree.add_child(toolbar_id, zoom_in_id);
        ui_tree.add_child(toolbar_id, zoom_out_id);

        ui_tree.add_child(viewport_id, zoom_indicator_id);
        ui_tree.add_child(viewport_id, toolbar_id);

        ui_tree.set_root(viewport_id);

        let mut render_context =
            RenderContext::new(self.viewport.size.0 as u32, self.viewport.size.1 as u32);

        let world = context.world();
        self.render_scene(&mut render_context, world);

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
