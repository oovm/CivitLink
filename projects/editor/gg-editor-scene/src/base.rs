//! 基础场景视图实现

use std::{cell::RefCell, rc::Rc};

use gg_core::{GError, GErrorKind, GResult};
use gg_ecs::Entity;
use gg_editor_shell::{Command, DragData, EditorContext, EditorEvent, EditorPanel, Key, PanelLayoutHint, PanelPosition};
use gg_render::{Color, DrawCommand, Rect, RenderContext};
use gg_ui::{FlexDirection, FontStyle, LayoutStyle, Style, UiNodeData, UiTree};
use gg_world::GameWorld;

use crate::{
    components::{RectRenderer, SpriteRenderer, Transform2D},
    view::SceneView,
    viewport::ViewportState,
};

/// 缩放步进因子
const ZOOM_STEP: f32 = 1.1;

/// 基础网格间距（世界坐标单位）
const BASE_GRID_SPACING: f32 = 50.0;

/// 复制粘贴偏移量（世界坐标单位）
const CLIPBOARD_OFFSET: f32 = 20.0;

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

/// 变换类型
///
/// 标识变换操作的具体类型，用于 `TransformCommand` 中区分移动、旋转和缩放。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransformKind {
    /// 移动变换
    Translate,
    /// 旋转变换
    Rotate,
    /// 缩放变换
    Scale,
}

/// 变换值
///
/// 存储变换操作的具体数值，与 `TransformKind` 一一对应：
/// - `Translate` 对应 `Position((f32, f32))`
/// - `Rotate` 对应 `Rotation(f32)`
/// - `Scale` 对应 `Scale((f32, f32))`
#[derive(Debug, Clone, Copy)]
pub enum TransformValue {
    /// 位置值（x, y）
    Position((f32, f32)),
    /// 旋转值（弧度）
    Rotation(f32),
    /// 缩放值（scale_x, scale_y）
    Scale((f32, f32)),
}

/// 变换命令
///
/// 可撤销/重做的变换操作命令，记录实体的变换类型、旧值和新值。
/// 执行时将新值应用到实体的 `Transform2D` 组件，撤销时恢复旧值。
pub struct TransformCommand {
    /// 被变换的实体 ID
    pub entity_id: u64,
    /// 变换类型
    pub transform_kind: TransformKind,
    /// 变换前的值
    pub old_value: TransformValue,
    /// 变换后的值
    pub new_value: TransformValue,
    /// 命令描述
    pub description: String,
}

impl Command for TransformCommand {
    fn execute(&mut self, context: &mut EditorContext) -> GResult<()> {
        let entity = u64_to_entity(self.entity_id);
        let world = context.world_mut();
        if let Some(transform) = world.get_component_mut::<Transform2D>(entity) {
            match self.new_value {
                TransformValue::Position((x, y)) => {
                    transform.x = x;
                    transform.y = y;
                }
                TransformValue::Rotation(r) => {
                    transform.rotation = r;
                }
                TransformValue::Scale((sx, sy)) => {
                    transform.scale_x = sx;
                    transform.scale_y = sy;
                }
            }
            Ok(())
        }
        else {
            Err(GError {
                kind: GErrorKind::Ecs, message: format!("实体 {} 不存在或缺少 Transform2D 组件", self.entity_id)
            })
        }
    }

    fn undo(&mut self, context: &mut EditorContext) -> GResult<()> {
        let entity = u64_to_entity(self.entity_id);
        let world = context.world_mut();
        if let Some(transform) = world.get_component_mut::<Transform2D>(entity) {
            match self.old_value {
                TransformValue::Position((x, y)) => {
                    transform.x = x;
                    transform.y = y;
                }
                TransformValue::Rotation(r) => {
                    transform.rotation = r;
                }
                TransformValue::Scale((sx, sy)) => {
                    transform.scale_x = sx;
                    transform.scale_y = sy;
                }
            }
            Ok(())
        }
        else {
            Err(GError {
                kind: GErrorKind::Ecs, message: format!("实体 {} 不存在或缺少 Transform2D 组件", self.entity_id)
            })
        }
    }

    fn description(&self) -> &str {
        &self.description
    }
}

/// 实体快照
///
/// 存储实体的组件数据，用于删除实体后恢复。
/// 包含 `Transform2D` 和可选的渲染器组件数据。
#[derive(Debug, Clone)]
pub struct EntitySnapshot {
    /// 变换组件数据
    pub transform: Transform2D,
    /// 矩形渲染器组件数据（如果存在）
    pub rect_renderer: Option<RectRenderer>,
    /// 精灵渲染器组件数据（如果存在）
    pub sprite_renderer: Option<SpriteRenderer>,
}

/// 删除实体命令
///
/// 可撤销/重做的删除实体操作命令。
/// 执行时从世界中移除实体，撤销时重新生成实体并恢复所有组件。
pub struct DeleteEntityCommand {
    /// 被删除的实体 ID
    pub entity_id: u64,
    /// 实体组件快照，用于撤销时恢复
    pub snapshot: EntitySnapshot,
    /// 命令描述
    pub description: String,
}

impl Command for DeleteEntityCommand {
    fn execute(&mut self, context: &mut EditorContext) -> GResult<()> {
        let entity = u64_to_entity(self.entity_id);
        context.world_mut().despawn(entity)
    }

    fn undo(&mut self, context: &mut EditorContext) -> GResult<()> {
        let entity = u64_to_entity(self.entity_id);
        let world = context.world_mut();
        world.spawn_with_entity(entity);
        world.add_component(entity, self.snapshot.transform.clone())?;
        if let Some(ref rect) = self.snapshot.rect_renderer {
            world.add_component(entity, rect.clone())?;
        }
        if let Some(ref sprite) = self.snapshot.sprite_renderer {
            world.add_component(entity, sprite.clone())?;
        }
        Ok(())
    }

    fn description(&self) -> &str {
        &self.description
    }
}

/// 创建实体命令
///
/// 可撤销/重做的创建实体操作命令。
/// 执行时在世界中生成新实体并添加 `Transform2D` 和对应渲染器组件，
/// 撤销时从世界中移除该实体。
pub struct CreateEntityCommand {
    /// 创建的实体 ID，执行后设置
    pub entity_id: Option<u64>,
    /// 实体的世界坐标位置
    pub position: (f32, f32),
    /// 实体类型
    pub entity_kind: SceneEntityKind,
    /// 命令描述
    pub description: String,
}

impl Command for CreateEntityCommand {
    fn execute(&mut self, context: &mut EditorContext) -> GResult<()> {
        let world = context.world_mut();
        let entity =
            world.spawn().insert(Transform2D { x: self.position.0, y: self.position.1, ..Transform2D::default() }).id();
        match self.entity_kind {
            SceneEntityKind::Rect => {
                world.add_component(entity, RectRenderer::default())?;
            }
            SceneEntityKind::Sprite => {
                world.add_component(entity, SpriteRenderer::default())?;
            }
        }
        self.entity_id = Some(entity_to_u64(entity));
        Ok(())
    }

    fn undo(&mut self, context: &mut EditorContext) -> GResult<()> {
        if let Some(id) = self.entity_id {
            let entity = u64_to_entity(id);
            context.world_mut().despawn(entity)
        }
        else {
            Err(GError { kind: GErrorKind::Other, message: "实体尚未创建，无法撤销".to_string() })
        }
    }

    fn description(&self) -> &str {
        &self.description
    }
}

/// 剪贴板实体数据
///
/// 存储复制到剪贴板的实体组件信息，用于粘贴时创建新实体。
#[derive(Debug, Clone)]
pub struct ClipboardEntity {
    /// 实体的变换组件数据
    pub transform: Transform2D,
    /// 矩形渲染器组件数据（如果存在）
    pub rect_renderer: Option<RectRenderer>,
    /// 精灵渲染器组件数据（如果存在）
    pub sprite_renderer: Option<SpriteRenderer>,
    /// 实体类型
    pub kind: SceneEntityKind,
}

/// 场景右键菜单
///
/// 管理场景视图右键上下文菜单的显示状态和位置信息。
pub struct SceneContextMenu {
    /// 菜单是否可见
    pub visible: bool,
    /// 菜单屏幕坐标位置
    pub position: (f32, f32),
    /// 菜单对应的世界坐标位置，用于在该位置创建实体
    pub world_position: (f32, f32),
}

impl SceneContextMenu {
    /// 创建默认的隐藏右键菜单
    pub fn new() -> Self {
        Self { visible: false, position: (0.0, 0.0), world_position: (0.0, 0.0) }
    }

    /// 在指定位置显示右键菜单
    ///
    /// `position` 为屏幕坐标，`world_position` 为对应的世界坐标。
    pub fn show(&mut self, position: (f32, f32), world_position: (f32, f32)) {
        self.visible = true;
        self.position = position;
        self.world_position = world_position;
    }

    /// 隐藏右键菜单
    pub fn hide(&mut self) {
        self.visible = false;
    }
}

impl Default for SceneContextMenu {
    fn default() -> Self {
        Self::new()
    }
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
/// 存储场景中单个实体的渲染信息，包括位置、尺寸、旋转、缩放、颜色和类型。
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
    /// 旋转角度（弧度）
    pub rotation: f32,
    /// X 缩放因子
    pub scale_x: f32,
    /// Y 缩放因子
    pub scale_y: f32,
    /// 填充颜色
    pub color: Color,
    /// 实体渲染类型
    pub kind: SceneEntityKind,
}

/// 变换工具模式
///
/// 定义场景编辑器中变换工具的三种操作模式。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransformGizmo {
    /// 移动模式
    Translate,
    /// 旋转模式
    Rotate,
    /// 缩放模式
    Scale,
}

impl Default for TransformGizmo {
    fn default() -> Self {
        TransformGizmo::Translate
    }
}

/// 变换工具状态
///
/// 管理变换工具的当前交互状态，包括活跃轴和拖拽偏移。
#[derive(Debug, Clone)]
pub struct GizmoState {
    /// 当前变换模式
    pub mode: TransformGizmo,
    /// 活跃轴（"x"、"y" 或 "xy"）
    pub active_axis: Option<String>,
    /// 拖拽起始世界坐标
    pub drag_start_world: Option<(f32, f32)>,
    /// 拖拽起始时实体的世界坐标
    pub drag_entity_start_pos: Option<(f32, f32)>,
    /// 拖拽起始时实体的旋转角度
    pub drag_entity_start_rotation: Option<f32>,
    /// 拖拽起始时实体的缩放
    pub drag_entity_start_scale: Option<(f32, f32)>,
}

impl GizmoState {
    /// 创建默认变换工具状态
    pub fn new() -> Self {
        Self {
            mode: TransformGizmo::Translate,
            active_axis: None,
            drag_start_world: None,
            drag_entity_start_pos: None,
            drag_entity_start_rotation: None,
            drag_entity_start_scale: None,
        }
    }
}

impl Default for GizmoState {
    fn default() -> Self {
        Self::new()
    }
}

/// 框选状态
///
/// 管理场景编辑器中框选操作的状态信息。
#[derive(Debug, Clone)]
pub struct SelectionBox {
    /// 框选起始屏幕坐标
    pub start_pos: (f32, f32),
    /// 框选当前屏幕坐标
    pub current_pos: (f32, f32),
    /// 框选是否激活
    pub is_active: bool,
}

impl SelectionBox {
    /// 创建默认框选状态
    pub fn new() -> Self {
        Self { start_pos: (0.0, 0.0), current_pos: (0.0, 0.0), is_active: false }
    }
}

impl Default for SelectionBox {
    fn default() -> Self {
        Self::new()
    }
}

/// 基础场景视图
///
/// 提供场景视图的默认实现，包括视口管理、实体选中、网格显示、
/// 变换撤销/重做、实体删除/创建、复制粘贴和右键上下文菜单功能。
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
    /// 场景渲染目标名称
    render_target_name: String,
    /// 待提交的场景绘制命令
    render_commands: Vec<DrawCommand>,
    /// 变换工具模式
    transform_gizmo: TransformGizmo,
    /// 变换工具状态
    gizmo_state: GizmoState,
    /// 框选状态
    selection_box: SelectionBox,
    /// 是否正在使用变换工具拖拽
    is_gizmo_dragging: bool,
    /// Ctrl 键是否按下
    ctrl_pressed: bool,
    /// Shift 键是否按下
    shift_pressed: bool,
    /// Alt 键是否按下
    alt_pressed: bool,
    /// 剪贴板中的实体数据列表
    clipboard: Vec<ClipboardEntity>,
    /// 右键上下文菜单
    context_menu: SceneContextMenu,
    /// Gizmo 拖拽开始时实体的变换旧值
    gizmo_drag_old_value: Option<TransformValue>,
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

    /// 重置视口到默认状态
    pub fn reset_viewport(&mut self) {
        self.viewport = ViewportState::new();
    }

    /// 放大视口
    ///
    /// 以视口中心为缩放中心，按 ZOOM_STEP 因子放大。
    pub fn zoom_in(&mut self) {
        let center = (self.viewport.size.0 / 2.0, self.viewport.size.1 / 2.0);
        self.viewport.zoom_to(ZOOM_STEP, center);
    }

    /// 缩小视口
    ///
    /// 以视口中心为缩放中心，按 1/ZOOM_STEP 因子缩小。
    pub fn zoom_out(&mut self) {
        let center = (self.viewport.size.0 / 2.0, self.viewport.size.1 / 2.0);
        self.viewport.zoom_to(1.0 / ZOOM_STEP, center);
    }

    /// 切换网格可见性
    pub fn toggle_grid(&mut self) {
        self.grid_visible = !self.grid_visible;
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
        }
        else {
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

    /// 框选命中测试
    ///
    /// 查找屏幕坐标框选区域内的所有实体。
    /// 将框选矩形的两个角点转换为世界坐标，然后遍历所有拥有 Transform2D 的实体，
    /// 检查实体包围盒是否与框选区域相交。
    fn hit_test_box(
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
    ///
    /// 以矩形中心为旋转中心，应用缩放和旋转变换，
    /// 返回变换后的四个角点坐标（屏幕坐标）。
    fn compute_transformed_corners(
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
    ///
    /// 优先从 `SpriteRenderer` 获取尺寸，其次从 `RectRenderer` 获取，
    /// 均不存在时返回默认尺寸 (50.0, 50.0)。
    fn get_entity_size(world: &GameWorld, entity: Entity) -> (f32, f32) {
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
    ///
    /// 将实体添加到选中列表，不取消其他已选中实体。
    pub fn select_entity_multi(&mut self, entity: Entity) {
        if !self.selected_entities.contains(&entity) {
            self.selected_entities.push(entity);
            self.pending_select_events.push(entity_to_u64(entity));
        }
    }

    /// 聚焦选中实体
    ///
    /// 计算选中实体的包围盒，调整视口使其居中显示。
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

    /// 渲染变换工具
    ///
    /// 根据当前变换模式在选中实体位置渲染变换工具。
    /// 移动模式渲染四向箭头，旋转模式渲染圆环，缩放模式渲染方块手柄。
    pub fn render_gizmo(&self, context: &mut RenderContext) {
        if self.selected_entities.is_empty() {
            return;
        }

        let entity = self.selected_entities[0];
        if let Some(se) = self.scene_entities.iter().find(|e| e.entity == entity) {
            let (screen_x, screen_y) = self.viewport.world_to_screen((se.world_x, se.world_y));
            let screen_w = se.width * self.viewport.zoom;
            let screen_h = se.height * self.viewport.zoom;
            let center_x = screen_x + screen_w / 2.0;
            let center_y = screen_y + screen_h / 2.0;

            let gizmo_size = 60.0;

            match self.transform_gizmo {
                TransformGizmo::Translate => {
                    let x_color = Color::new(1.0, 0.3, 0.3, 0.9);
                    let y_color = Color::new(0.3, 1.0, 0.3, 0.9);
                    context.draw(DrawCommand::Line {
                        start: [center_x, center_y],
                        end: [center_x + gizmo_size, center_y],
                        color: x_color,
                        width: 2.0,
                    });
                    context.draw(DrawCommand::Line {
                        start: [center_x, center_y],
                        end: [center_x, center_y + gizmo_size],
                        color: y_color,
                        width: 2.0,
                    });
                    context.draw(DrawCommand::Rect {
                        rect: Rect::new(center_x + gizmo_size - 4.0, center_y - 4.0, 8.0, 8.0),
                        color: x_color,
                        corner_radius: 0.0,
                    });
                    context.draw(DrawCommand::Rect {
                        rect: Rect::new(center_x - 4.0, center_y + gizmo_size - 4.0, 8.0, 8.0),
                        color: y_color,
                        corner_radius: 0.0,
                    });
                }
                TransformGizmo::Rotate => {
                    let rotate_color = Color::new(0.3, 0.6, 1.0, 0.9);
                    let segments = 32;
                    for i in 0..segments {
                        let angle1 = 2.0 * std::f32::consts::PI * i as f32 / segments as f32;
                        let angle2 = 2.0 * std::f32::consts::PI * (i + 1) as f32 / segments as f32;
                        let x1 = center_x + gizmo_size * angle1.cos();
                        let y1 = center_y + gizmo_size * angle1.sin();
                        let x2 = center_x + gizmo_size * angle2.cos();
                        let y2 = center_y + gizmo_size * angle2.sin();
                        context.draw(DrawCommand::Line { start: [x1, y1], end: [x2, y2], color: rotate_color, width: 2.0 });
                    }
                }
                TransformGizmo::Scale => {
                    let x_color = Color::new(1.0, 0.3, 0.3, 0.9);
                    let y_color = Color::new(0.3, 1.0, 0.3, 0.9);
                    let handle_size = 8.0;
                    context.draw(DrawCommand::Line {
                        start: [center_x, center_y],
                        end: [center_x + gizmo_size, center_y],
                        color: x_color,
                        width: 2.0,
                    });
                    context.draw(DrawCommand::Line {
                        start: [center_x, center_y],
                        end: [center_x, center_y + gizmo_size],
                        color: y_color,
                        width: 2.0,
                    });
                    context.draw(DrawCommand::Rect {
                        rect: Rect::new(
                            center_x + gizmo_size - handle_size / 2.0,
                            center_y - handle_size / 2.0,
                            handle_size,
                            handle_size,
                        ),
                        color: x_color,
                        corner_radius: 2.0,
                    });
                    context.draw(DrawCommand::Rect {
                        rect: Rect::new(
                            center_x - handle_size / 2.0,
                            center_y + gizmo_size - handle_size / 2.0,
                            handle_size,
                            handle_size,
                        ),
                        color: y_color,
                        corner_radius: 2.0,
                    });
                }
            }

            let center_color = Color::new(1.0, 1.0, 1.0, 0.9);
            context.draw(DrawCommand::Rect {
                rect: Rect::new(center_x - 3.0, center_y - 3.0, 6.0, 6.0),
                color: center_color,
                corner_radius: 3.0,
            });
        }
    }

    /// 渲染框选矩形
    ///
    /// 当框选操作激活时，渲染半透明蓝色选择框。
    pub fn render_selection_box(&self, context: &mut RenderContext) {
        if !self.selection_box.is_active {
            return;
        }

        let x = self.selection_box.start_pos.0.min(self.selection_box.current_pos.0);
        let y = self.selection_box.start_pos.1.min(self.selection_box.current_pos.1);
        let w = (self.selection_box.current_pos.0 - self.selection_box.start_pos.0).abs();
        let h = (self.selection_box.current_pos.1 - self.selection_box.start_pos.1).abs();

        let fill_color = Color::new(0.2, 0.5, 1.0, 0.15);
        let border_color = Color::new(0.2, 0.5, 1.0, 0.6);

        context.draw(DrawCommand::Rect { rect: Rect::new(x, y, w, h), color: fill_color, corner_radius: 0.0 });
        context.draw(DrawCommand::Line { start: [x, y], end: [x + w, y], color: border_color, width: 1.0 });
        context.draw(DrawCommand::Line { start: [x + w, y], end: [x + w, y + h], color: border_color, width: 1.0 });
        context.draw(DrawCommand::Line { start: [x + w, y + h], end: [x, y + h], color: border_color, width: 1.0 });
        context.draw(DrawCommand::Line { start: [x, y + h], end: [x, y], color: border_color, width: 1.0 });
    }

    /// 处理变换工具交互
    ///
    /// 检测鼠标是否点击在变换工具的轴手柄上，并记录拖拽起始时的变换值。
    fn handle_gizmo_interaction(&mut self, screen_pos: (f32, f32), context: &mut EditorContext) -> bool {
        if self.selected_entities.is_empty() {
            return false;
        }

        let entity = self.selected_entities[0];
        let entity_id = entity_to_u64(entity);
        if let Some(se) = self.scene_entities.iter().find(|e| e.entity == entity) {
            let (screen_x, screen_y) = self.viewport.world_to_screen((se.world_x, se.world_y));
            let screen_w = se.width * self.viewport.zoom;
            let screen_h = se.height * self.viewport.zoom;
            let center_x = screen_x + screen_w / 2.0;
            let center_y = screen_y + screen_h / 2.0;

            let gizmo_size = 60.0;
            let hit_threshold = 10.0;

            match self.transform_gizmo {
                TransformGizmo::Translate | TransformGizmo::Scale => {
                    let x_axis_start = center_x;
                    let x_axis_end = center_x + gizmo_size;
                    let y_axis_start = center_y;
                    let y_axis_end = center_y + gizmo_size;

                    let on_x_axis = screen_pos.1 >= center_y - hit_threshold
                        && screen_pos.1 <= center_y + hit_threshold
                        && screen_pos.0 >= x_axis_start - hit_threshold
                        && screen_pos.0 <= x_axis_end + hit_threshold;

                    let on_y_axis = screen_pos.0 >= center_x - hit_threshold
                        && screen_pos.0 <= center_x + hit_threshold
                        && screen_pos.1 >= y_axis_start - hit_threshold
                        && screen_pos.1 <= y_axis_end + hit_threshold;

                    if on_x_axis || on_y_axis {
                        self.is_gizmo_dragging = true;
                        self.gizmo_state.active_axis = if on_x_axis && on_y_axis {
                            Some("xy".to_string())
                        }
                        else if on_x_axis {
                            Some("x".to_string())
                        }
                        else {
                            Some("y".to_string())
                        };
                        self.gizmo_state.drag_start_world = Some(self.viewport.screen_to_world(screen_pos));
                        self.gizmo_state.drag_entity_start_pos = Some((se.world_x, se.world_y));
                        self.gizmo_state.drag_entity_start_scale = Some((se.scale_x, se.scale_y));

                        let world = context.world();
                        if let Some(transform) = world.get_component::<Transform2D>(entity) {
                            self.gizmo_drag_old_value = match self.transform_gizmo {
                                TransformGizmo::Translate => Some(TransformValue::Position((transform.x, transform.y))),
                                TransformGizmo::Scale => Some(TransformValue::Scale((transform.scale_x, transform.scale_y))),
                                TransformGizmo::Rotate => None,
                            };
                        }

                        return true;
                    }
                }
                TransformGizmo::Rotate => {
                    let dx = screen_pos.0 - center_x;
                    let dy = screen_pos.1 - center_y;
                    let dist = (dx * dx + dy * dy).sqrt();
                    if dist >= gizmo_size - hit_threshold && dist <= gizmo_size + hit_threshold {
                        self.is_gizmo_dragging = true;
                        self.gizmo_state.active_axis = Some("rotation".to_string());
                        self.gizmo_state.drag_start_world = Some(self.viewport.screen_to_world(screen_pos));
                        self.gizmo_state.drag_entity_start_rotation = Some(se.rotation);

                        let world = context.world();
                        if let Some(transform) = world.get_component::<Transform2D>(entity) {
                            self.gizmo_drag_old_value = Some(TransformValue::Rotation(transform.rotation));
                        }

                        return true;
                    }
                }
            }
        }
        false
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
                    rotation: transform.rotation,
                    scale_x: transform.scale_x,
                    scale_y: transform.scale_y,
                    color: sprite.color,
                    kind: SceneEntityKind::Sprite,
                });
            }
            else if let Some(rect) = world.get_component::<RectRenderer>(entity) {
                entities.push(SceneEntity {
                    entity,
                    world_x: transform.x,
                    world_y: transform.y,
                    width: rect.width,
                    height: rect.height,
                    rotation: transform.rotation,
                    scale_x: transform.scale_x,
                    scale_y: transform.scale_y,
                    color: rect.color,
                    kind: SceneEntityKind::Rect,
                });
            }
            else {
                entities.push(SceneEntity {
                    entity,
                    world_x: transform.x,
                    world_y: transform.y,
                    width: 50.0,
                    height: 50.0,
                    rotation: transform.rotation,
                    scale_x: transform.scale_x,
                    scale_y: transform.scale_y,
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
            let (screen_x, screen_y) = self.viewport.world_to_screen((entity.world_x, entity.world_y));
            let screen_w = entity.width * zoom;
            let screen_h = entity.height * zoom;

            let has_transform = entity.rotation != 0.0 || entity.scale_x != 1.0 || entity.scale_y != 1.0;

            if has_transform {
                let corners = Self::compute_transformed_corners(
                    screen_x,
                    screen_y,
                    screen_w,
                    screen_h,
                    entity.rotation,
                    entity.scale_x,
                    entity.scale_y,
                );

                context.draw(DrawCommand::Line {
                    start: [corners[0].0, corners[0].1],
                    end: [corners[1].0, corners[1].1],
                    color: entity.color,
                    width: 2.0,
                });
                context.draw(DrawCommand::Line {
                    start: [corners[1].0, corners[1].1],
                    end: [corners[2].0, corners[2].1],
                    color: entity.color,
                    width: 2.0,
                });
                context.draw(DrawCommand::Line {
                    start: [corners[2].0, corners[2].1],
                    end: [corners[3].0, corners[3].1],
                    color: entity.color,
                    width: 2.0,
                });
                context.draw(DrawCommand::Line {
                    start: [corners[3].0, corners[3].1],
                    end: [corners[0].0, corners[0].1],
                    color: entity.color,
                    width: 2.0,
                });
            }
            else {
                context.draw(DrawCommand::Rect {
                    rect: Rect::new(screen_x, screen_y, screen_w, screen_h),
                    color: entity.color,
                    corner_radius: 0.0,
                });
            }
        }

        self.render_grid(context);
        self.render_selection_overlay(context);
        self.render_gizmo(context);
        self.render_selection_box(context);
    }

    /// 收集场景绘制命令
    ///
    /// 创建临时 RenderContext，调用 render_scene 渲染场景，
    /// 然后提取绘制命令存储到内部缓冲区，供 EditorShell 在 tick 中使用。
    pub fn collect_render_commands(&mut self, world: &GameWorld) {
        let mut context = RenderContext::new(self.viewport.size.0 as u32, self.viewport.size.1 as u32);
        self.render_scene(&mut context, world);
        self.render_commands = context.commands().to_vec();
    }

    /// 取出待提交的场景绘制命令
    ///
    /// 返回内部缓冲区中的所有绘制命令并清空缓冲区。
    /// EditorShell 可在 tick 中调用此方法获取命令后提交给 WgpuRenderer。
    pub fn drain_render_commands(&mut self) -> Vec<DrawCommand> {
        std::mem::take(&mut self.render_commands)
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
        }
        else if screen_spacing > 200.0 {
            BASE_GRID_SPACING * 0.5
        }
        else {
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
                let (screen_x, screen_y) = self.viewport.world_to_screen((scene_entity.world_x, scene_entity.world_y));
                let screen_w = scene_entity.width * self.viewport.zoom;
                let screen_h = scene_entity.height * self.viewport.zoom;

                let has_transform = scene_entity.rotation != 0.0 || scene_entity.scale_x != 1.0 || scene_entity.scale_y != 1.0;

                if has_transform {
                    let corners = Self::compute_transformed_corners(
                        screen_x - 2.0,
                        screen_y - 2.0,
                        screen_w + 4.0,
                        screen_h + 4.0,
                        scene_entity.rotation,
                        scene_entity.scale_x,
                        scene_entity.scale_y,
                    );

                    context.draw(DrawCommand::Line {
                        start: [corners[0].0, corners[0].1],
                        end: [corners[1].0, corners[1].1],
                        color: highlight_color,
                        width: 2.0,
                    });
                    context.draw(DrawCommand::Line {
                        start: [corners[1].0, corners[1].1],
                        end: [corners[2].0, corners[2].1],
                        color: highlight_color,
                        width: 2.0,
                    });
                    context.draw(DrawCommand::Line {
                        start: [corners[2].0, corners[2].1],
                        end: [corners[3].0, corners[3].1],
                        color: highlight_color,
                        width: 2.0,
                    });
                    context.draw(DrawCommand::Line {
                        start: [corners[3].0, corners[3].1],
                        end: [corners[0].0, corners[0].1],
                        color: highlight_color,
                        width: 2.0,
                    });
                }
                else {
                    context.draw(DrawCommand::Rect {
                        rect: Rect::new(screen_x - 2.0, screen_y - 2.0, screen_w + 4.0, screen_h + 4.0),
                        color: highlight_color,
                        corner_radius: 0.0,
                    });
                }
            }
        }
    }

    /// 构建缩放指示器文本
    fn zoom_indicator_text(&self) -> String {
        format!("{:.0}%", self.viewport.zoom * 100.0)
    }

    /// 从 GameWorld 中获取实体的快照数据
    ///
    /// 读取实体的 `Transform2D`、`RectRenderer` 和 `SpriteRenderer` 组件数据，
    /// 构建用于撤销恢复的 `EntitySnapshot`。
    fn take_entity_snapshot(world: &GameWorld, entity: Entity) -> Option<EntitySnapshot> {
        let transform = world.get_component::<Transform2D>(entity)?.clone();
        let rect_renderer = world.get_component::<RectRenderer>(entity).cloned();
        let sprite_renderer = world.get_component::<SpriteRenderer>(entity).cloned();
        Some(EntitySnapshot { transform, rect_renderer, sprite_renderer })
    }

    /// 从 GameWorld 中获取实体的剪贴板数据
    ///
    /// 读取实体的 `Transform2D`、`RectRenderer` 和 `SpriteRenderer` 组件数据，
    /// 根据渲染器类型确定 `SceneEntityKind`，构建用于粘贴的 `ClipboardEntity`。
    fn take_clipboard_entity(world: &GameWorld, entity: Entity) -> Option<ClipboardEntity> {
        let transform = world.get_component::<Transform2D>(entity)?.clone();
        let rect_renderer = world.get_component::<RectRenderer>(entity).cloned();
        let sprite_renderer = world.get_component::<SpriteRenderer>(entity).cloned();
        let kind = if sprite_renderer.is_some() { SceneEntityKind::Sprite } else { SceneEntityKind::Rect };
        Some(ClipboardEntity { transform, rect_renderer, sprite_renderer, kind })
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
    /// 响应鼠标和键盘事件实现实体选择、拖拽移动、变换撤销/重做、
    /// 实体删除/创建、复制粘贴和右键上下文菜单等功能：
    /// - 左键按下：命中测试选中实体或取消选中
    /// - 鼠标移动：拖拽选中的实体更新其世界坐标
    /// - 左键释放：结束拖拽，创建 TransformCommand 记录变更
    /// - 中键按下/释放/移动：视口平移
    /// - 滚轮：视口缩放
    /// - Ctrl+Z：撤销
    /// - Ctrl+Y：重做
    /// - Delete：删除选中实体
    /// - Ctrl+C：复制选中实体到剪贴板
    /// - Ctrl+V：粘贴剪贴板中的实体
    /// - 右键：显示上下文菜单
    fn on_event(&mut self, event: &EditorEvent, context: &mut EditorContext) {
        match event {
            EditorEvent::KeyDown { key } => match key {
                Key::W => {
                    if !self.ctrl_pressed && !self.alt_pressed {
                        self.set_transform_mode(TransformGizmo::Translate);
                    }
                }
                Key::E => {
                    if !self.ctrl_pressed && !self.alt_pressed {
                        self.set_transform_mode(TransformGizmo::Rotate);
                    }
                }
                Key::R => {
                    if !self.ctrl_pressed && !self.alt_pressed {
                        self.set_transform_mode(TransformGizmo::Scale);
                    }
                }
                Key::F => self.frame_selection(),
                Key::Escape => {
                    if self.is_gizmo_dragging {
                        self.is_gizmo_dragging = false;
                        self.gizmo_state.active_axis = None;
                    }
                    if self.selection_box.is_active {
                        self.selection_box.is_active = false;
                    }
                    self.context_menu.hide();
                }
                Key::Z => {
                    if self.ctrl_pressed {
                        let _ = context.undo_command();
                    }
                }
                Key::Y => {
                    if self.ctrl_pressed {
                        let _ = context.redo_command();
                    }
                }
                Key::Delete => {
                    if !self.selected_entities.is_empty() {
                        let entities_to_delete: Vec<Entity> = self.selected_entities.clone();
                        for entity in entities_to_delete {
                            let entity_id = entity_to_u64(entity);
                            if let Some(snapshot) = Self::take_entity_snapshot(context.world(), entity) {
                                let command = Box::new(DeleteEntityCommand {
                                    entity_id,
                                    snapshot,
                                    description: format!("删除实体 {}", entity_id),
                                });
                                context.execute_command(command);
                            }
                        }
                        self.selected_entities.clear();
                        context.events_mut().publish(EditorEvent::EntityDeselected);
                    }
                }
                Key::C => {
                    if self.ctrl_pressed && !self.selected_entities.is_empty() {
                        self.clipboard.clear();
                        for &entity in &self.selected_entities {
                            if let Some(clip) = Self::take_clipboard_entity(context.world(), entity) {
                                self.clipboard.push(clip);
                            }
                        }
                    }
                }
                Key::V => {
                    if self.ctrl_pressed && !self.clipboard.is_empty() {
                        for clip in &self.clipboard {
                            let position = (clip.transform.x + CLIPBOARD_OFFSET, clip.transform.y + CLIPBOARD_OFFSET);
                            let command = Box::new(CreateEntityCommand {
                                entity_id: None,
                                position,
                                entity_kind: clip.kind,
                                description: format!("粘贴实体"),
                            });
                            context.execute_command(command);
                        }
                    }
                }
                Key::Control => {
                    self.ctrl_pressed = true;
                }
                Key::Shift => {
                    self.shift_pressed = true;
                }
                Key::Alt => {
                    self.alt_pressed = true;
                }
                _ => {}
            },
            EditorEvent::KeyUp { key } => match key {
                Key::Control => {
                    self.ctrl_pressed = false;
                }
                Key::Shift => {
                    self.shift_pressed = false;
                }
                Key::Alt => {
                    self.alt_pressed = false;
                }
                _ => {}
            },
            EditorEvent::MouseDown { button, position } => match button {
                gg_editor_shell::MouseButton::Left => {
                    self.context_menu.hide();
                    if self.handle_gizmo_interaction(*position, context) {
                    }
                    else {
                        let world_pos = self.viewport.screen_to_world(*position);
                        let world = context.world();
                        let hit = Self::hit_test_world(world, world_pos);

                        if let Some((entity, wx, wy)) = hit {
                            self.select_entity(entity);
                            self.is_dragging = true;
                            self.drag_start_screen = *position;
                            self.drag_entity_start_pos = (wx, wy);
                            self.dragged_entity = Some(entity);
                            context.events_mut().publish(EditorEvent::EntitySelected { entity: entity_to_u64(entity) });
                        }
                        else {
                            self.deselect_all();
                            self.is_dragging = false;
                            self.dragged_entity = None;
                            self.selection_box.start_pos = *position;
                            self.selection_box.current_pos = *position;
                            self.selection_box.is_active = true;
                            context.events_mut().publish(EditorEvent::EntityDeselected);
                        }
                    }
                }
                gg_editor_shell::MouseButton::Middle => {
                    self.handle_middle_button_down(*position);
                }
                gg_editor_shell::MouseButton::Right => {
                    let world_pos = self.viewport.screen_to_world(*position);
                    let world = context.world();
                    let hit = Self::hit_test_world(world, world_pos);
                    if hit.is_some() {
                        self.context_menu.show(*position, world_pos);
                    }
                    else {
                        self.context_menu.show(*position, world_pos);
                    }
                }
            },
            EditorEvent::MouseUp { button, .. } => match button {
                gg_editor_shell::MouseButton::Left => {
                    if self.is_gizmo_dragging {
                        if let Some(entity) = self.selected_entities.first() {
                            let entity_id = entity_to_u64(*entity);
                            if let Some(old_value) = self.gizmo_drag_old_value.take() {
                                let world = context.world();
                                if let Some(transform) = world.get_component::<Transform2D>(*entity) {
                                    let new_value = match self.transform_gizmo {
                                        TransformGizmo::Translate => TransformValue::Position((transform.x, transform.y)),
                                        TransformGizmo::Rotate => TransformValue::Rotation(transform.rotation),
                                        TransformGizmo::Scale => TransformValue::Scale((transform.scale_x, transform.scale_y)),
                                    };
                                    let transform_kind = match self.transform_gizmo {
                                        TransformGizmo::Translate => TransformKind::Translate,
                                        TransformGizmo::Rotate => TransformKind::Rotate,
                                        TransformGizmo::Scale => TransformKind::Scale,
                                    };
                                    let desc = match transform_kind {
                                        TransformKind::Translate => "移动实体",
                                        TransformKind::Rotate => "旋转实体",
                                        TransformKind::Scale => "缩放实体",
                                    };
                                    let command = Box::new(TransformCommand {
                                        entity_id,
                                        transform_kind,
                                        old_value,
                                        new_value,
                                        description: format!("{} {}", desc, entity_id),
                                    });
                                    context.execute_command(command);
                                }
                            }
                        }
                    }
                    self.is_dragging = false;
                    self.dragged_entity = None;
                    self.is_gizmo_dragging = false;
                    self.gizmo_state.active_axis = None;
                    self.gizmo_drag_old_value = None;
                    if self.selection_box.is_active {
                        self.selection_box.is_active = false;
                    }
                }
                gg_editor_shell::MouseButton::Middle => {
                    self.handle_middle_button_up();
                }
                gg_editor_shell::MouseButton::Right => {}
            },
            EditorEvent::MouseMove { position } => {
                if self.is_gizmo_dragging {
                    if let Some(entity) = self.selected_entities.first() {
                        let current_world = self.viewport.screen_to_world(*position);
                        if let Some(start_world) = self.gizmo_state.drag_start_world {
                            if let Some(start_pos) = self.gizmo_state.drag_entity_start_pos {
                                let dx = current_world.0 - start_world.0;
                                let dy = current_world.1 - start_world.1;

                                let world = context.world_mut();
                                if let Some(transform) = world.get_component_mut::<Transform2D>(*entity) {
                                    match self.transform_gizmo {
                                        TransformGizmo::Translate => match self.gizmo_state.active_axis.as_deref() {
                                            Some("x") => transform.x = start_pos.0 + dx,
                                            Some("y") => transform.y = start_pos.1 + dy,
                                            Some("xy") => {
                                                transform.x = start_pos.0 + dx;
                                                transform.y = start_pos.1 + dy;
                                            }
                                            _ => {}
                                        },
                                        TransformGizmo::Rotate => {
                                            if let Some(se) = self.scene_entities.iter().find(|e| e.entity == *entity) {
                                                let (cx, cy) = self.viewport.world_to_screen((se.world_x, se.world_y));
                                                let start_angle = (self.gizmo_state.drag_start_world.unwrap().1 - cy)
                                                    .atan2(self.gizmo_state.drag_start_world.unwrap().0 - cx);
                                                let current_angle = (position.1 - cy).atan2(position.0 - cx);
                                                transform.rotation += current_angle - start_angle;
                                            }
                                        }
                                        TransformGizmo::Scale => {
                                            if let Some(start_scale) = self.gizmo_state.drag_entity_start_scale {
                                                match self.gizmo_state.active_axis.as_deref() {
                                                    Some("x") => {
                                                        let delta = 1.0 + dx * 0.01;
                                                        transform.scale_x = (start_scale.0 * delta).max(0.01);
                                                    }
                                                    Some("y") => {
                                                        let delta = 1.0 + dy * 0.01;
                                                        transform.scale_y = (start_scale.1 * delta).max(0.01);
                                                    }
                                                    Some("xy") => {
                                                        let delta = 1.0 + (dx + dy) * 0.005;
                                                        transform.scale_x = (start_scale.0 * delta).max(0.01);
                                                        transform.scale_y = (start_scale.1 * delta).max(0.01);
                                                    }
                                                    _ => {}
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                else if self.is_dragging {
                    if let Some(entity) = self.dragged_entity {
                        let (start_wx, start_wy) = self.viewport.screen_to_world(self.drag_start_screen);
                        let (current_wx, current_wy) = self.viewport.screen_to_world(*position);
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
                }
                else if self.selection_box.is_active {
                    self.selection_box.current_pos = *position;
                }
                else if self.is_panning {
                    self.handle_mouse_move(*position);
                }
                self.last_mouse_pos = Some(*position);
            }
            EditorEvent::MouseWheel { delta, position } => {
                self.handle_scroll(delta.1, *position);
            }
            EditorEvent::DragEnd { position, data } => match data {
                DragData::AssetPath(path) => {
                    let world_pos = self.viewport.screen_to_world(*position);
                    let world = context.world_mut();
                    let entity = world
                        .spawn()
                        .insert(Transform2D { x: world_pos.0, y: world_pos.1, ..Transform2D::default() })
                        .insert(SpriteRenderer { texture_path: path.clone(), ..SpriteRenderer::default() })
                        .id();
                    context.events_mut().publish(EditorEvent::EntitySelected { entity: entity_to_u64(entity) });
                }
                DragData::MultiAsset(paths) => {
                    let world_pos = self.viewport.screen_to_world(*position);
                    for (i, path) in paths.iter().enumerate() {
                        let offset_x = (i as f32 % 5.0) * 60.0;
                        let offset_y = (i as f32 / 5.0).floor() * 60.0;
                        let world = context.world_mut();
                        let entity = world
                            .spawn()
                            .insert(Transform2D {
                                x: world_pos.0 + offset_x,
                                y: world_pos.1 + offset_y,
                                ..Transform2D::default()
                            })
                            .insert(SpriteRenderer { texture_path: path.clone(), ..SpriteRenderer::default() })
                            .id();
                        context.events_mut().publish(EditorEvent::EntitySelected { entity: entity_to_u64(entity) });
                    }
                }
                DragData::Entity(_) | DragData::Custom { .. } => {}
            },
            EditorEvent::Custom { name, .. } => match name.as_str() {
                "reset_view" => self.reset_viewport(),
                "zoom_in" => self.zoom_in(),
                "zoom_out" => self.zoom_out(),
                "toggle_grid" => self.toggle_grid(),
                _ => {}
            },
            _ => {}
        }
    }

    /// 构建场景视图面板 UI 节点树
    fn build_ui(&mut self, context: &mut EditorContext, ui_tree: &mut UiTree) -> Option<gg_ui::UiNodeId> {
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

        let viewport_style = Style::new()
            .with_background_color(Color::new(0.1, 0.1, 0.1, 1.0))
            .with_layout(LayoutStyle::new().with_direction(FlexDirection::Column));
        let viewport_id = ui_tree.create_node(
            "scene_viewport",
            viewport_style,
            UiNodeData::Custom { kind: format!("scene_render_target:{}", self.render_target_name) },
        );

        let zoom_text = self.zoom_indicator_text();
        let zoom_indicator_id = ui_tree.create_node(
            "zoom_indicator",
            Style::new().with_font(FontStyle::new().with_size(12.0).with_color(Color::new(0.9, 0.9, 0.9, 1.0))),
            UiNodeData::Text { content: zoom_text },
        );

        let toolbar_id = ui_tree.create_node(
            "scene_toolbar",
            Style::new().with_layout(LayoutStyle::new().with_direction(FlexDirection::Row).with_gap(4.0)),
            UiNodeData::Container,
        );

        let toggle_grid_id = ui_tree.create_node(
            "btn_toggle_grid",
            Style::new().with_background_color(Color::new(0.2, 0.2, 0.2, 1.0)),
            UiNodeData::Custom { kind: "button".to_string() },
        );

        let reset_view_id = ui_tree.create_node(
            "btn_reset_view",
            Style::new().with_background_color(Color::new(0.2, 0.2, 0.2, 1.0)),
            UiNodeData::Custom { kind: "button".to_string() },
        );

        let zoom_in_id = ui_tree.create_node(
            "btn_zoom_in",
            Style::new().with_background_color(Color::new(0.2, 0.2, 0.2, 1.0)),
            UiNodeData::Custom { kind: "button".to_string() },
        );

        let zoom_out_id = ui_tree.create_node(
            "btn_zoom_out",
            Style::new().with_background_color(Color::new(0.2, 0.2, 0.2, 1.0)),
            UiNodeData::Custom { kind: "button".to_string() },
        );

        ui_tree.add_child(toolbar_id, toggle_grid_id);
        ui_tree.add_child(toolbar_id, reset_view_id);
        ui_tree.add_child(toolbar_id, zoom_in_id);
        ui_tree.add_child(toolbar_id, zoom_out_id);

        ui_tree.add_child(viewport_id, zoom_indicator_id);
        ui_tree.add_child(viewport_id, toolbar_id);

        if self.context_menu.visible {
            let menu_x = self.context_menu.position.0;
            let menu_y = self.context_menu.position.1;
            let menu_style = Style::new()
                .with_background_color(Color::new(0.18, 0.18, 0.18, 0.95))
                .with_layout(LayoutStyle::new().with_direction(FlexDirection::Column).with_gap(2.0));
            let context_menu_id =
                ui_tree.create_node("context_menu", menu_style, UiNodeData::Custom { kind: "context_menu".to_string() });

            let create_empty_style = Style::new()
                .with_background_color(Color::new(0.25, 0.25, 0.25, 1.0))
                .with_layout(LayoutStyle::new().with_gap(4.0));
            let create_empty_id = ui_tree.create_node(
                "menu_create_empty",
                create_empty_style,
                UiNodeData::Text { content: "创建空实体".to_string() },
            );

            let create_rect_style = Style::new()
                .with_background_color(Color::new(0.25, 0.25, 0.25, 1.0))
                .with_layout(LayoutStyle::new().with_gap(4.0));
            let create_rect_id = ui_tree.create_node(
                "menu_create_rect",
                create_rect_style,
                UiNodeData::Text { content: "创建矩形".to_string() },
            );

            let create_sprite_style = Style::new()
                .with_background_color(Color::new(0.25, 0.25, 0.25, 1.0))
                .with_layout(LayoutStyle::new().with_gap(4.0));
            let create_sprite_id = ui_tree.create_node(
                "menu_create_sprite",
                create_sprite_style,
                UiNodeData::Text { content: "创建精灵".to_string() },
            );

            ui_tree.add_child(context_menu_id, create_empty_id);
            ui_tree.add_child(context_menu_id, create_rect_id);
            ui_tree.add_child(context_menu_id, create_sprite_id);
            ui_tree.add_child(viewport_id, context_menu_id);
        }

        Some(viewport_id)
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
