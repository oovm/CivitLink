//! STG 引擎配置模块
//! 定义引擎的配置结构和默认值

use serde::{Deserialize, Serialize};

/// STG 引擎配置
#[derive(Debug, Serialize, Deserialize)]
pub struct StgConfig {
    /// 游戏配置
    pub game: GameConfig,
    /// 显示配置
    pub display: DisplayConfig,
    /// 输入配置
    pub input: InputConfig,
    /// 游戏设置
    pub gameplay: GameplayConfig,
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
    /// 射击键
    pub shoot_key: String,
    /// 特殊武器键
    pub special_key: String,
}

/// 游戏玩法配置
#[derive(Debug, Serialize, Deserialize)]
pub struct GameplayConfig {
    /// 玩家初始生命值
    pub player_health: u32,
    /// 敌人生成间隔（毫秒）
    pub enemy_spawn_interval: u32,
    /// 子弹速度
    pub bullet_speed: f32,
    /// 敌人速度
    pub enemy_speed: f32,
}

impl Default for StgConfig {
    fn default() -> Self {
        Self {
            game: GameConfig {
                name: "STG Game".to_string(),
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
                shoot_key: "Space".to_string(),
                special_key: "Shift".to_string(),
            },
            gameplay: GameplayConfig {
                player_health: 3,
                enemy_spawn_interval: 1000,
                bullet_speed: 5.0,
                enemy_speed: 1.0,
            },
        }
    }
}
