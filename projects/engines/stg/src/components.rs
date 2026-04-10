//! STG 引擎组件定义
//! 定义游戏中使用的各种 ECS 组件

use gg_ecs::Component;

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
            collider_type: ColliderType::Circle { radius: 16.0 },
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
#[derive(Debug, Default, Component)]
pub struct Player {
    /// 玩家 ID
    pub id: u32,
    /// 是否无敌
    pub invulnerable: bool,
    /// 无敌时间（毫秒）
    pub invulnerable_time: u32,
}

/// 敌人组件
/// 标记实体为敌人
#[derive(Debug, Default, Component)]
pub struct Enemy {
    /// 敌人类型
    pub enemy_type: String,
    /// 敌人等级
    pub level: u32,
    /// 敌人分数
    pub score: u32,
}

/// 子弹组件
/// 标记实体为子弹
#[derive(Debug, Component)]
pub struct Bullet {
    /// 子弹类型
    pub bullet_type: String,
    /// 伤害值
    pub damage: u32,
    /// 发射者类型
    pub shooter_type: ShooterType,
}

impl Default for Bullet {
    fn default() -> Self {
        Self {
            bullet_type: "default".to_string(),
            damage: 1,
            shooter_type: ShooterType::Player,
        }
    }
}

/// 发射者类型
#[derive(Debug)]
pub enum ShooterType {
    /// 玩家
    Player,
    /// 敌人
    Enemy,
}

/// 武器组件
/// 包含武器信息
#[derive(Debug, Component)]
pub struct Weapon {
    /// 武器类型
    pub weapon_type: String,
    /// 射速（发/秒）
    pub fire_rate: f32,
    /// 子弹速度
    pub bullet_speed: f32,
    /// 上次射击时间（毫秒）
    pub last_fire_time: u64,
}

impl Default for Weapon {
    fn default() -> Self {
        Self {
            weapon_type: "default".to_string(),
            fire_rate: 5.0,
            bullet_speed: 5.0,
            last_fire_time: 0,
        }
    }
}

/// AI 组件
/// 包含敌人 AI 信息
#[derive(Debug, Component)]
pub struct AI {
    /// AI 行为模式
    pub behavior: AIBehavior,
    /// 巡逻路径
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
#[derive(Debug)]
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
