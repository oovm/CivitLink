//! STG 引擎 Boss AI 定义
//! 定义 Boss 组件、阶段和阶段转换条件

use crate::bullet_pattern::BulletPattern;
use gg_ecs::Component;

/// Boss 阶段转换条件
#[derive(Debug, Clone)]
pub enum PhaseTransition {
    /// 生命值低于百分比时切换
    HealthBelow(f32),
    /// 经过指定秒数后切换
    TimeElapsed(f32),
    /// 手动触发切换
    Manual,
}

/// Boss 移动模式
#[derive(Debug, Clone)]
pub enum BossMovePattern {
    /// 静止不动
    Stationary,
    /// 水平往返
    Horizontal {
        /// 移动范围
        range: f32,
        /// 移动速度
        speed: f32,
    },
    /// 圆形移动
    Circular {
        /// 半径
        radius: f32,
        /// 角速度
        angular_speed: f32,
    },
    /// 追踪玩家
    Chase {
        /// 追踪速度
        speed: f32,
    },
}

/// Boss 阶段定义
#[derive(Debug, Clone)]
pub struct BossPhase {
    /// 阶段名称
    pub name: String,
    /// 该阶段的弹幕模式列表
    pub bullet_patterns: Vec<BulletPattern>,
    /// 该阶段的移动模式
    pub move_pattern: BossMovePattern,
    /// 阶段切换条件
    pub transition: PhaseTransition,
    /// 阶段持续时间（帧，仅 TimeElapsed 模式使用）
    pub duration_frames: u32,
}

/// Boss 组件
#[derive(Debug, Component)]
pub struct Boss {
    /// Boss 阶段列表
    pub phases: Vec<BossPhase>,
    /// 当前阶段索引
    pub current_phase_index: usize,
    /// 阶段切换后的无敌帧数
    pub invulnerable_frames: u32,
    /// 阶段切换后的剩余无敌帧数
    pub remaining_invulnerable_frames: u32,
    /// 当前阶段已持续时间（帧）
    pub phase_elapsed_frames: u32,
    /// 水平移动偏移（BossMovePattern::Horizontal 使用）
    pub horizontal_offset: f32,
    /// 水平移动方向（1.0 或 -1.0）
    pub horizontal_direction: f32,
    /// 圆形移动角度（BossMovePattern::Circular 使用）
    pub circular_angle: f32,
}

impl Default for Boss {
    fn default() -> Self {
        Self {
            phases: vec![],
            current_phase_index: 0,
            invulnerable_frames: 60,
            remaining_invulnerable_frames: 0,
            phase_elapsed_frames: 0,
            horizontal_offset: 0.0,
            horizontal_direction: 1.0,
            circular_angle: 0.0,
        }
    }
}
