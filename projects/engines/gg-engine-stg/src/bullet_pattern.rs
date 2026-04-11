//! STG 引擎弹幕模式定义
//! 定义弹幕模式类型、发射器和生命周期组件

use gg_ecs::{Component, Resource};

/// 弹幕模式类型
#[derive(Debug, Clone)]
pub enum PatternType {
    /// 单发直射
    Single,
    /// 扇形散射
    Spread {
        /// 散射角度范围（弧度）
        spread_angle: f32,
    },
    /// 环形弹幕
    Ring,
    /// 螺旋弹幕
    Spiral {
        /// 螺旋角速度（弧度/帧）
        angular_speed: f32,
    },
    /// 瞄准玩家方向
    Aimed,
    /// 自定义角度序列
    Custom {
        /// 角度列表（弧度）
        angles: Vec<f32>,
    },
}

/// 弹幕模式组件
#[derive(Debug, Clone, Component)]
pub struct BulletPattern {
    /// 弹幕模式类型
    pub pattern_type: PatternType,
    /// 子弹速度
    pub speed: f32,
    /// 每次发射的子弹数量
    pub count: u32,
    /// 发射间隔（帧）
    pub interval: u32,
    /// 初始角度偏移（弧度）
    pub angle_offset: f32,
}

impl Default for BulletPattern {
    fn default() -> Self {
        Self { pattern_type: PatternType::Single, speed: 3.0, count: 1, interval: 60, angle_offset: 0.0 }
    }
}

/// 弹幕发射器组件
#[derive(Debug, Component)]
pub struct BulletEmitter {
    /// 弹幕模式列表（支持组合弹幕）
    pub patterns: Vec<BulletPattern>,
    /// 当前模式索引
    pub current_pattern_index: usize,
    /// 当前模式已发射次数
    pub current_pattern_fire_count: u32,
    /// 当前模式发射次数上限（0 表示无限）
    pub pattern_fire_limit: u32,
    /// 发射计时器（帧）
    pub fire_timer: u32,
    /// 是否激活
    pub active: bool,
    /// 当前螺旋角度（仅 Spiral 模式使用）
    pub spiral_angle: f32,
}

impl Default for BulletEmitter {
    fn default() -> Self {
        Self {
            patterns: vec![BulletPattern::default()],
            current_pattern_index: 0,
            current_pattern_fire_count: 0,
            pattern_fire_limit: 0,
            fire_timer: 0,
            active: true,
            spiral_angle: 0.0,
        }
    }
}

/// 子弹生命周期组件
#[derive(Debug, Component)]
pub struct BulletLifetime {
    /// 剩余帧数
    pub remaining_frames: u32,
    /// 最大帧数
    pub max_frames: u32,
}

impl Default for BulletLifetime {
    fn default() -> Self {
        Self { remaining_frames: 300, max_frames: 300 }
    }
}

/// 子弹对象池资源
#[derive(Debug, Default, Resource)]
pub struct BulletPool {
    /// 可复用的实体 ID 列表
    pub available: Vec<u32>,
    /// 已创建的子弹总数
    pub total_created: u32,
}
