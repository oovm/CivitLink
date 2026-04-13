//! 场景视图类型定义

use gg_ecs::Entity;
use gg_render::Color;

/// 缩放步进因子
pub const ZOOM_STEP: f32 = 1.1;

/// 基础网格间距（世界坐标单位）
pub const BASE_GRID_SPACING: f32 = 50.0;

/// 复制粘贴偏移量（世界坐标单位）
pub const CLIPBOARD_OFFSET: f32 = 20.0;

/// 将 ECS Entity 转换为 u64 标识符
///
/// 使用代数高 32 位、索引低 32 位的编码方式，
/// 与 `gg_world::SceneSerializer` 保持一致。
pub fn entity_to_u64(entity: Entity) -> u64 {
    ((entity.generation() as u64) << 32) | (entity.index() as u64)
}

/// 将 u64 标识符还原为 ECS Entity
///
/// 与 `entity_to_u64` 互逆，从编码中提取索引和代数。
pub fn u64_to_entity(id: u64) -> Entity {
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
