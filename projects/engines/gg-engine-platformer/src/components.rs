//! Platformer 引擎组件定义
//! 定义游戏中使用的各种 ECS 组件

use gg_ecs::{Component, Entity};
use gg_render::DrawCommand;
use std::collections::HashMap;

/// 绘制命令缓冲区
///
/// 作为全局资源存储在 World 中，RenderSystem 将绘制命令写入此缓冲区，
/// 引擎在 tick 之后读取缓冲区内容并提交给实际渲染器。
#[derive(Debug, Default)]
pub struct DrawCommandBuffer {
    /// 待执行的绘制命令列表
    pub commands: Vec<DrawCommand>,
}

impl DrawCommandBuffer {
    /// 创建空的绘制命令缓冲区
    pub fn new() -> Self {
        Self { commands: Vec::new() }
    }

    /// 清空所有绘制命令
    pub fn clear(&mut self) {
        self.commands.clear();
    }

    /// 添加一条绘制命令
    pub fn push(&mut self, command: DrawCommand) {
        self.commands.push(command);
    }
}

/// 帧间隔时间资源
///
/// 作为全局资源存储在 World 中，记录上一帧到当前帧的时间间隔（秒），
/// 所有物理和运动系统应基于此值进行时间步进计算。
#[derive(Debug)]
pub struct DeltaTime {
    /// 帧间隔时间（秒）
    pub seconds: f32,
}

impl Default for DeltaTime {
    fn default() -> Self {
        Self { seconds: 1.0 / 60.0 }
    }
}

/// 碰撞信息结构
///
/// 记录两个碰撞体之间的碰撞详细信息，包括穿透深度、碰撞法线和碰撞点，
/// 供碰撞响应系统进行精确的穿透修正和速度反射。
#[derive(Debug, Clone)]
pub struct CollisionInfo {
    /// 穿透深度
    pub penetration_depth: f32,
    /// 碰撞法线（从 entity1 指向 entity2 的方向）
    pub normal: (f32, f32),
    /// 碰撞点坐标
    pub point: (f32, f32),
}

/// 输入动作枚举
///
/// 将物理按键映射到逻辑动作，支持从配置文件读取映射关系，
/// 实现输入与逻辑的解耦。包含键盘和手柄输入动作。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum InputAction {
    /// 向左移动
    MoveLeft,
    /// 向右移动
    MoveRight,
    /// 跳跃
    Jump,
    /// 手柄移动（摇杆方向）
    GamepadMove,
    /// 手柄跳跃（A 按钮）
    GamepadJump,
}

/// 输入状态资源
///
/// 存储当前帧各输入动作的激活状态，由引擎事件循环写入，由输入系统读取。
#[derive(Debug, Default)]
pub struct InputState {
    /// 各动作的激活状态
    pub actions: HashMap<InputAction, bool>,
    /// 手柄左摇杆 X 轴值（-1.0 ~ 1.0）
    pub gamepad_left_stick_x: f32,
    /// 手柄左摇杆 Y 轴值（-1.0 ~ 1.0）
    pub gamepad_left_stick_y: f32,
    /// 手柄是否已连接
    pub gamepad_connected: bool,
}

impl InputState {
    /// 创建空的输入状态
    pub fn new() -> Self {
        let mut actions = HashMap::new();
        actions.insert(InputAction::MoveLeft, false);
        actions.insert(InputAction::MoveRight, false);
        actions.insert(InputAction::Jump, false);
        actions.insert(InputAction::GamepadMove, false);
        actions.insert(InputAction::GamepadJump, false);
        Self { actions, gamepad_left_stick_x: 0.0, gamepad_left_stick_y: 0.0, gamepad_connected: false }
    }

    /// 检查指定动作是否激活
    pub fn is_action_pressed(&self, action: InputAction) -> bool {
        self.actions.get(&action).copied().unwrap_or(false)
    }

    /// 设置指定动作的激活状态
    pub fn set_action(&mut self, action: InputAction, pressed: bool) {
        self.actions.insert(action, pressed);
    }
}

/// 变换组件
/// 包含实体的位置和旋转信息
#[derive(Debug, Default, Component)]
pub struct Transform {
    /// X 坐标
    pub x: f32,
    /// Y 坐标
    pub y: f32,
    /// 旋转角度（弧度）
    pub rotation: f32,
    /// 缩放
    pub scale: f32,
}

/// 速度组件
/// 包含实体的速度和加速度信息
#[derive(Debug, Default, Component)]
pub struct Velocity {
    /// X 方向速度
    pub dx: f32,
    /// Y 方向速度
    pub dy: f32,
    /// X 方向加速度
    pub ax: f32,
    /// Y 方向加速度
    pub ay: f32,
}

/// 精灵组件
/// 包含实体的渲染信息
#[derive(Debug, Component)]
pub struct Sprite {
    /// 精灵路径
    pub path: String,
    /// 宽度
    pub width: f32,
    /// 高度
    pub height: f32,
    /// 是否可见
    pub visible: bool,
}

impl Default for Sprite {
    fn default() -> Self {
        Self { path: "".to_string(), width: 32.0, height: 32.0, visible: true }
    }
}

/// 碰撞体组件
/// 包含碰撞检测信息
#[derive(Debug, Component)]
pub struct Collider {
    /// 碰撞体类型
    pub collider_type: ColliderType,
    /// 碰撞层
    pub layer: u32,
    /// 碰撞掩码
    pub mask: u32,
}

impl Default for Collider {
    fn default() -> Self {
        Self { collider_type: ColliderType::Rectangle { width: 32.0, height: 32.0 }, layer: 1, mask: 1 }
    }
}

/// 碰撞体类型
#[derive(Debug, Clone)]
pub enum ColliderType {
    /// 圆形碰撞体
    Circle {
        /// 半径
        radius: f32,
    },
    /// 矩形碰撞体
    Rectangle {
        /// 宽度
        width: f32,
        /// 高度
        height: f32,
    },
}

/// 物理材质组件
///
/// 定义碰撞体的物理材质属性，包括弹性系数和摩擦系数。
/// 不同平台可配置不同的物理材质，影响碰撞响应行为。
#[derive(Debug, Clone, Component)]
pub struct PhysicsMaterial {
    /// 弹性系数（0.0 = 完全非弹性，1.0 = 完全弹性）
    pub restitution: f32,
    /// 摩擦系数（0.0 = 无摩擦/冰面，1.0 = 最大摩擦）
    pub friction: f32,
}

impl Default for PhysicsMaterial {
    fn default() -> Self {
        Self { restitution: 0.0, friction: 0.8 }
    }
}

impl PhysicsMaterial {
    /// 创建冰面材质
    pub fn ice() -> Self {
        Self { restitution: 0.0, friction: 0.05 }
    }

    /// 创建橡胶材质（高弹性）
    pub fn rubber() -> Self {
        Self { restitution: 0.9, friction: 0.6 }
    }

    /// 创建弹簧材质
    pub fn spring() -> Self {
        Self { restitution: 1.0, friction: 0.8 }
    }
}

/// 物理体组件
/// 包含物理属性信息
#[derive(Debug, Component)]
pub struct PhysicsBody {
    /// 质量
    pub mass: f32,
    /// 重力缩放
    pub gravity_scale: f32,
    /// 是否受重力影响
    pub affected_by_gravity: bool,
    /// 是否是静态物体
    pub is_static: bool,
    /// 是否在地面上
    pub on_ground: bool,
    /// 地面法线
    pub ground_normal: (f32, f32),
    /// 是否启用连续碰撞检测（CCD）
    pub ccd_enabled: bool,
}

impl Default for PhysicsBody {
    fn default() -> Self {
        Self {
            mass: 1.0,
            gravity_scale: 1.0,
            affected_by_gravity: true,
            is_static: false,
            on_ground: false,
            ground_normal: (0.0, 1.0),
            ccd_enabled: false,
        }
    }
}

/// 生命值组件
/// 包含实体的生命值信息
#[derive(Debug, Default, Component)]
pub struct Health {
    /// 当前生命值
    pub current: u32,
    /// 最大生命值
    pub max: u32,
}

/// 玩家组件
/// 标记实体为玩家
#[derive(Debug, Component)]
pub struct Player {
    /// 玩家实体标识
    pub id: Entity,
    /// 是否可以跳跃
    pub can_jump: bool,
    /// 跳跃次数
    pub jump_count: u32,
    /// 最大跳跃次数
    pub max_jump_count: u32,
    /// 是否无敌
    pub invulnerable: bool,
    /// 无敌剩余时间（秒）
    pub invulnerable_timer: f32,
}

impl Default for Player {
    fn default() -> Self {
        Self {
            id: Entity::new(0, 0),
            can_jump: false,
            jump_count: 0,
            max_jump_count: 2,
            invulnerable: false,
            invulnerable_timer: 0.0,
        }
    }
}

/// 敌人组件
/// 标记实体为敌人，与 AI 组件配合使用
#[derive(Debug, Component)]
pub struct Enemy {
    /// 敌人类型名称
    pub enemy_type: String,
    /// 敌人碰撞伤害值
    pub damage: u32,
    /// 踩踏消灭时给予玩家的弹跳力度
    pub stomp_bounce_force: f32,
}

impl Default for Enemy {
    fn default() -> Self {
        Self { enemy_type: "basic".to_string(), damage: 1, stomp_bounce_force: 8.0 }
    }
}

/// 平台组件
/// 标记实体为平台，支持多种平台类型
#[derive(Debug, Component)]
pub struct Platform {
    /// 平台类型
    pub platform_type: PlatformType,
    /// 是否可破坏
    pub breakable: bool,
    /// 耐久度
    pub durability: u32,
    /// 移动平台起点坐标
    pub move_start: (f32, f32),
    /// 移动平台终点坐标
    pub move_end: (f32, f32),
    /// 移动平台速度
    pub move_speed: f32,
    /// 移动平台当前方向（1 或 -1）
    pub move_direction: f32,
    /// 易碎平台销毁延迟（秒）
    pub break_delay: f32,
    /// 易碎平台销毁计时器（秒）
    pub break_timer: f32,
    /// 消失平台淡出时间（秒）
    pub fade_time: f32,
    /// 消失平台淡出计时器（秒）
    pub fade_timer: f32,
    /// 消失平台重生时间（秒）
    pub respawn_time: f32,
    /// 消失平台重生计时器（秒）
    pub respawn_timer: f32,
    /// 是否为单向平台（仅从上方可站立）
    pub is_one_way: bool,
    /// 平台是否活跃（消失平台淡出期间为 false）
    pub is_active: bool,
    /// 弹簧平台弹跳力度
    pub spring_force: f32,
    /// 弹簧平台是否被压缩
    pub spring_compressed: bool,
    /// 传送带平台推动方向（-1.0 = 向左，1.0 = 向右）
    pub conveyor_direction: f32,
    /// 传送带平台推动速度
    pub conveyor_speed: f32,
}

impl Default for Platform {
    fn default() -> Self {
        Self {
            platform_type: PlatformType::Static,
            breakable: false,
            durability: 1,
            move_start: (0.0, 0.0),
            move_end: (0.0, 0.0),
            move_speed: 2.0,
            move_direction: 1.0,
            break_delay: 0.5,
            break_timer: 0.0,
            fade_time: 0.5,
            fade_timer: 0.0,
            respawn_time: 3.0,
            respawn_timer: 0.0,
            is_one_way: false,
            is_active: true,
            spring_force: 15.0,
            spring_compressed: false,
            conveyor_direction: 1.0,
            conveyor_speed: 3.0,
        }
    }
}

/// 平台类型
#[derive(Debug, Clone, Copy)]
pub enum PlatformType {
    /// 静态平台
    Static,
    /// 移动平台
    Moving,
    /// 易碎平台
    Breakable,
    /// 消失平台
    Disappearing,
    /// 弹簧平台（落地时施加额外向上力）
    Spring,
    /// 传送带平台（站在上面的实体被自动推动）
    Conveyor,
    /// 冰面平台（降低摩擦系数产生滑行效果）
    Ice,
}

/// 可收集物品组件
/// 标记实体为可收集物品
#[derive(Debug, Component)]
pub struct Collectible {
    /// 物品类型
    pub item_type: String,
    /// 价值
    pub value: u32,
    /// 是否已被收集
    pub collected: bool,
}

impl Default for Collectible {
    fn default() -> Self {
        Self { item_type: "coin".to_string(), value: 1, collected: false }
    }
}

/// AI 组件
/// 包含敌人 AI 信息
#[derive(Debug, Component)]
pub struct AI {
    /// AI 行为模式
    pub behavior: AIBehavior,
    /// 巡逻路径点坐标列表
    pub patrol_path: Vec<(f32, f32)>,
    /// 当前路径点索引
    pub current_path_index: usize,
    /// 移动速度
    pub move_speed: f32,
    /// 检测范围（像素）
    pub detection_range: f32,
    /// 攻击范围（像素）
    pub attack_range: f32,
    /// 触发躲避的生命值比例阈值（0.0~1.0）
    pub evade_health_threshold: f32,
    /// 跳跃攻击冷却计时器（秒）
    pub jump_attack_timer: f32,
    /// 跳跃攻击冷却时间（秒）
    pub jump_attack_cooldown: f32,
    /// 远程攻击冷却计时器（秒）
    pub ranged_attack_timer: f32,
    /// 远程攻击冷却时间（秒）
    pub ranged_attack_cooldown: f32,
    /// 导航路径点列表（用于路径寻找）
    pub nav_waypoints: Vec<(f32, f32)>,
    /// 当前导航路径点索引
    pub nav_waypoint_index: usize,
}

impl Default for AI {
    fn default() -> Self {
        Self {
            behavior: AIBehavior::Patrol,
            patrol_path: vec![],
            current_path_index: 0,
            move_speed: 1.0,
            detection_range: 200.0,
            attack_range: 50.0,
            evade_health_threshold: 0.3,
            jump_attack_timer: 0.0,
            jump_attack_cooldown: 2.0,
            ranged_attack_timer: 0.0,
            ranged_attack_cooldown: 3.0,
            nav_waypoints: vec![],
            nav_waypoint_index: 0,
        }
    }
}

/// AI 行为模式
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AIBehavior {
    /// 巡逻
    Patrol,
    /// 追踪
    Chase,
    /// 攻击
    Attack,
    /// 躲避
    Evade,
    /// 跳跃攻击（向玩家跳跃并发起攻击）
    JumpAttack,
    /// 远程攻击（向玩家发射投射物）
    RangedAttack,
}
