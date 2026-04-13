#![warn(missing_docs)]

//! STG 引擎组件定义
//! 定义游戏中使用的各种 ECS 组件

use gg_ecs::{Component, Entity};
use gg_render::DrawCommand;
use std::collections::HashMap;

/// 绘制命令缓冲区
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
        self.commands.clear()
    }

    /// 添加一条绘制命令
    pub fn push(&mut self, command: DrawCommand) {
        self.commands.push(command)
    }
}

/// 输入状态资源
#[derive(Debug, Default, Clone)]
pub struct InputState {
    /// 上方向键是否按下
    pub up_pressed: bool,
    /// 下方向键是否按下
    pub down_pressed: bool,
    /// 左方向键是否按下
    pub left_pressed: bool,
    /// 右方向键是否按下
    pub right_pressed: bool,
    /// 射击键是否按下
    pub shoot_pressed: bool,
    /// 特殊武器键是否按下
    pub special_pressed: bool,
}

/// 屏幕边界资源
#[derive(Debug, Clone, Copy)]
pub struct ScreenBounds {
    /// 屏幕宽度
    pub width: f32,
    /// 屏幕高度
    pub height: f32,
}

/// 游戏状态资源
#[derive(Debug, Clone)]
pub struct GameState {
    /// 当前分数
    pub score: u32,
    /// 剩余生命
    pub lives: u32,
    /// 当前关卡
    pub level: u32,
    /// 当前波次
    pub wave: u32,
    /// 游戏是否结束
    pub game_over: bool,
    /// 游戏是否暂停
    pub paused: bool,
}

impl Default for GameState {
    fn default() -> Self {
        Self { score: 0, lives: 3, level: 1, wave: 1, game_over: false, paused: false }
    }
}

/// 变换组件
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
#[derive(Debug, Clone, Component)]
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
        Self { collider_type: ColliderType::Circle { radius: 16.0 }, layer: 1, mask: 1 }
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

/// 生命值组件
#[derive(Debug, Default, Component)]
pub struct Health {
    /// 当前生命值
    pub current: u32,
    /// 最大生命值
    pub max: u32,
}

/// 玩家组件
#[derive(Debug, Component)]
pub struct Player {
    /// 玩家实体标识
    pub id: Entity,
    /// 是否无敌
    pub invulnerable: bool,
    /// 无敌剩余帧数
    pub invulnerable_frames: u32,
    /// 火力等级
    pub power_level: u32,
    /// 炸弹数量
    pub bomb_count: u32,
}

impl Default for Player {
    fn default() -> Self {
        Self { id: Entity::new(0, 0), invulnerable: false, invulnerable_frames: 0, power_level: 1, bomb_count: 3 }
    }
}

/// 敌人组件
#[derive(Debug, Component)]
pub struct Enemy {
    /// 敌人类型
    pub enemy_type: String,
    /// 敌人等级
    pub level: u32,
    /// 敌人分数
    pub score: u32,
}

impl Default for Enemy {
    fn default() -> Self {
        Self { enemy_type: "basic".to_string(), level: 1, score: 100 }
    }
}

/// 子弹组件
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
        Self { bullet_type: "default".to_string(), damage: 1, shooter_type: ShooterType::Player }
    }
}

/// 发射者类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShooterType {
    /// 玩家
    Player,
    /// 敌人
    Enemy,
}

/// 武器组件
#[derive(Debug, Component)]
pub struct Weapon {
    /// 武器类型
    pub weapon_type: String,
    /// 射速（发/秒）
    pub fire_rate: f32,
    /// 子弹速度
    pub bullet_speed: f32,
    /// 上次射击时间（帧计数）
    pub last_fire_frame: u32,
}

impl Default for Weapon {
    fn default() -> Self {
        Self { weapon_type: "default".to_string(), fire_rate: 5.0, bullet_speed: 5.0, last_fire_frame: 0 }
    }
}

/// AI 组件
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
        Self { behavior: AIBehavior::Patrol, patrol_path: vec![], current_path_index: 0, move_speed: 1.0 }
    }
}

/// AI 行为模式
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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

/// 行为树节点类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BehaviorNodeType {
    /// 序列节点：依次执行子节点，任一失败则整体失败
    Sequence,
    /// 选择节点：依次执行子节点，任一成功则整体成功
    Selector,
    /// 条件节点：评估条件是否满足
    Condition,
    /// 动作节点：执行具体行为
    Action,
}

/// 行为树节点
#[derive(Debug, Clone)]
pub struct BehaviorNode {
    /// 节点类型
    pub node_type: BehaviorNodeType,
    /// 条件函数名称（仅 Condition 节点使用）
    pub condition: Option<String>,
    /// 动作函数名称（仅 Action 节点使用）
    pub action: Option<String>,
    /// 子节点索引列表
    pub children: Vec<usize>,
}

impl Default for BehaviorNode {
    fn default() -> Self {
        Self { node_type: BehaviorNodeType::Action, condition: None, action: None, children: Vec::new() }
    }
}

/// 行为树组件
/// 用于定义实体的复杂 AI 行为逻辑
#[derive(Debug, Default, Component)]
pub struct BehaviorTree {
    /// 节点列表
    pub nodes: Vec<BehaviorNode>,
    /// 当前活跃节点索引
    pub active_node: Option<usize>,
}

/// 波次配置
///
/// 定义单个敌人波次的生成参数。
#[derive(Debug, Clone)]
pub struct WaveConfig {
    /// 敌人数量
    pub enemy_count: u32,
    /// 敌人类型列表
    pub enemy_types: Vec<String>,
    /// 生成间隔（帧）
    pub spawn_interval: u32,
    /// 是否为 Boss 波次
    pub boss_wave: bool,
}

impl Default for WaveConfig {
    fn default() -> Self {
        Self { enemy_count: 5, enemy_types: vec!["basic".to_string()], spawn_interval: 60, boss_wave: false }
    }
}

/// 关卡配置资源
///
/// 管理当前关卡的所有波次配置和推进状态。
#[derive(Debug, Clone)]
pub struct LevelConfig {
    /// 波次配置列表
    pub waves: Vec<WaveConfig>,
    /// 当前波次索引
    pub current_wave: usize,
    /// 当前关卡
    pub current_level: usize,
    /// 当前波次已生成的敌人数量
    pub spawned_count: u32,
    /// 生成计时器（帧）
    pub spawn_timer: u32,
    /// 波次是否已完成（所有敌人已消灭）
    pub wave_complete: bool,
}

impl Default for LevelConfig {
    fn default() -> Self {
        Self { waves: Vec::new(), current_wave: 0, current_level: 1, spawned_count: 0, spawn_timer: 0, wave_complete: false }
    }
}

/// 视觉特效类型
///
/// 定义 STG 游戏中可用的视觉特效类型。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VfxType {
    /// 爆炸特效
    Explosion,
    /// 命中火花特效
    HitSpark,
    /// 道具发光特效
    PowerUpGlow,
    /// Boss 光环特效
    BossAura,
}

/// 视觉特效发射器组件
///
/// 附加到实体上以播放视觉特效，管理特效的生命周期。
#[derive(Debug, Component)]
pub struct VfxEmitter {
    /// 特效类型
    pub effect_type: VfxType,
    /// 持续时间（帧）
    pub duration: u32,
    /// 已经过时间（帧）
    pub elapsed: u32,
    /// 是否激活
    pub active: bool,
}

impl Default for VfxEmitter {
    fn default() -> Self {
        Self { effect_type: VfxType::Explosion, duration: 30, elapsed: 0, active: true }
    }
}

/// 音频事件类型
///
/// 定义 STG 游戏中可触发的音频事件。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AudioEvent {
    /// 射击音效
    Shoot,
    /// 爆炸音效
    Explosion,
    /// 道具拾取音效
    ItemPickup,
    /// Boss 出现音效
    BossAppear,
    /// 炸弹使用音效
    Bomb,
    /// 游戏结束音效
    GameOver,
    /// 玩家受伤音效
    PlayerHit,
}

/// 音频源组件
///
/// 附加到实体上以播放音频，支持自动播放和循环控制。
#[derive(Debug, Component)]
pub struct AudioSource {
    /// 音频片段路径
    pub clip_path: String,
    /// 音量（0.0 ~ 1.0）
    pub volume: f32,
    /// 是否循环播放
    pub looping: bool,
    /// 是否在实体创建时自动播放
    pub play_on_start: bool,
}

impl Default for AudioSource {
    fn default() -> Self {
        Self { clip_path: String::new(), volume: 1.0, looping: false, play_on_start: false }
    }
}

/// 音频命令
///
/// 定义音频系统可执行的操作命令。
#[derive(Debug, Clone)]
pub enum AudioCommand {
    /// 播放指定路径的音频片段
    Play(String),
    /// 设置音频音量
    SetVolume(f32),
    /// 停止所有音频
    StopAll,
}

/// 音频总线资源
///
/// 管理音频事件队列和命令队列，连接游戏逻辑与音频后端。
#[derive(Debug, Default)]
pub struct AudioBus {
    /// 待处理的音频事件列表
    pub events: Vec<AudioEvent>,
    /// 待执行的音频命令列表
    commands: Vec<AudioCommand>,
}

impl AudioBus {
    /// 创建新的音频总线
    pub fn new() -> Self {
        Self { events: Vec::new(), commands: Vec::new() }
    }

    /// 发送音频命令
    pub fn send(&mut self, command: AudioCommand) {
        self.commands.push(command);
    }

    /// 获取并清空所有待执行命令
    pub fn drain_commands(&mut self) -> Vec<AudioCommand> {
        self.commands.drain(..).collect()
    }
}
