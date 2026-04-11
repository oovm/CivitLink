//! Platformer 引擎组件定义
//! 定义游戏中使用的各种 ECS 组件

use gg_ecs::{Component, Entity};
use gg_render::DrawCommand;

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
        Self {
            path: "".to_string(),
            width: 32.0,
            height: 32.0,
            visible: true,
        }
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
        Self {
            collider_type: ColliderType::Rectangle { width: 32.0, height: 32.0 },
            layer: 1,
            mask: 1,
        }
    }
}

/// 碰撞体类型
#[derive(Debug)]
pub enum ColliderType {
    /// 圆形碰撞体
    Circle { radius: f32 },
    /// 矩形碰撞体
    Rectangle { width: f32, height: f32 },
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
}

impl Default for Player {
    fn default() -> Self {
        Self {
            id: Entity::new(0, 0),
            can_jump: false,
            jump_count: 0,
            max_jump_count: 0,
        }
    }
}

/// 平台组件
/// 标记实体为平台
#[derive(Debug, Component)]
pub struct Platform {
    /// 平台类型
    pub platform_type: PlatformType,
    /// 是否可破坏
    pub breakable: bool,
    /// 耐久度
    pub durability: u32,
}

impl Default for Platform {
    fn default() -> Self {
        Self {
            platform_type: PlatformType::Static,
            breakable: false,
            durability: 1,
        }
    }
}

/// 平台类型
#[derive(Debug)]
pub enum PlatformType {
    /// 静态平台
    Static,
    /// 移动平台
    Moving,
    /// 易碎平台
    Breakable,
    /// 消失平台
    Disappearing,
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
        Self {
            item_type: "coin".to_string(),
            value: 1,
            collected: false,
        }
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
}

impl Default for AI {
    fn default() -> Self {
        Self {
            behavior: AIBehavior::Patrol,
            patrol_path: vec![],
            current_path_index: 0,
            move_speed: 1.0,
        }
    }
}

/// AI 行为模式
#[derive(Debug, Clone, Copy)]
pub enum AIBehavior {
    /// 巡逻
    Patrol,
    /// 追踪
    Chase,
    /// 攻击
    Attack,
    /// 躲避
    Evade,
}
