//! Platformer 引擎配置模块
//! 定义引擎的配置结构和默认值

use serde::{Deserialize, Serialize};

/// Platformer 引擎配置
#[derive(Debug, Serialize, Deserialize)]
pub struct PlatformerConfig {
    /// 游戏配置
    pub game: GameConfig,
    /// 显示配置
    pub display: DisplayConfig,
    /// 输入配置
    pub input: InputConfig,
    /// 游戏设置
    pub gameplay: GameplayConfig,
    /// 物理设置
    pub physics: PhysicsConfig,
}

/// 游戏配置
#[derive(Debug, Serialize, Deserialize)]
pub struct GameConfig {
    /// 游戏名称
    pub name: String,
    /// 游戏版本
    pub version: String,
    /// 作者
    pub author: String,
}

/// 显示配置
#[derive(Debug, Serialize, Deserialize)]
pub struct DisplayConfig {
    /// 窗口宽度
    pub width: u32,
    /// 窗口高度
    pub height: u32,
    /// 是否全屏
    pub fullscreen: bool,
    /// 帧率限制
    pub fps_limit: u32,
}

/// 输入配置
#[derive(Debug, Serialize, Deserialize)]
pub struct InputConfig {
    /// 移动速度
    pub move_speed: f32,
    /// 跳跃力度
    pub jump_force: f32,
    /// 跳跃键
    pub jump_key: String,
    /// 左移键
    pub left_key: String,
    /// 右移键
    pub right_key: String,
}

/// 游戏玩法配置
#[derive(Debug, Serialize, Deserialize)]
pub struct GameplayConfig {
    /// 玩家初始生命值
    pub player_health: u32,
    /// 重力加速度
    pub gravity: f32,
    /// 最大下落速度
    pub max_fall_speed: f32,
    /// 地面摩擦系数
    pub ground_friction: f32,
    /// 空中摩擦力
    pub air_friction: f32,
}

/// 物理配置
#[derive(Debug, Serialize, Deserialize)]
pub struct PhysicsConfig {
    /// 碰撞检测精度
    pub collision_precision: f32,
    /// 物理更新频率
    pub physics_update_rate: u32,
    /// 平台碰撞层
    pub platform_layer: u32,
    /// 玩家碰撞层
    pub player_layer: u32,
    /// 物品碰撞层
    pub collectible_layer: u32,
}

impl Default for PlatformerConfig {
    fn default() -> Self {
        Self {
            game: GameConfig {
                name: "Platformer Game".to_string(),
                version: "0.1.0".to_string(),
                author: "GG Engine".to_string(),
            },
            display: DisplayConfig {
                width: 800,
                height: 600,
                fullscreen: false,
                fps_limit: 60,
            },
            input: InputConfig {
                move_speed: 3.0,
                jump_force: 8.0,
                jump_key: "Space".to_string(),
                left_key: "Left".to_string(),
                right_key: "Right".to_string(),
            },
            gameplay: GameplayConfig {
                player_health: 3,
                gravity: 0.5,
                max_fall_speed: 10.0,
                ground_friction: 0.8,
                air_friction: 0.1,
            },
            physics: PhysicsConfig {
                collision_precision: 0.01,
                physics_update_rate: 60,
                platform_layer: 1,
                player_layer: 2,
                collectible_layer: 4,
            },
        }
    }
}
