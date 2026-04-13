//! STG 引擎配置模块
//! 定义引擎的配置结构和默认值

use crate::components::AudioEvent;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

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
    /// 碰撞配置
    pub collision: CollisionConfig,
    /// 道具配置
    pub item: ItemConfig,
    /// 音频配置
    pub audio: AudioConfig,
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
    /// Boss 基础生命值
    pub boss_base_health: u32,
}

/// 碰撞配置
#[derive(Debug, Serialize, Deserialize)]
pub struct CollisionConfig {
    /// 空间哈希单元格大小
    pub cell_size: f32,
    /// 玩家碰撞层
    pub player_layer: u32,
    /// 敌人碰撞层
    pub enemy_layer: u32,
    /// 玩家子弹碰撞层
    pub player_bullet_layer: u32,
    /// 敌人子弹碰撞层
    pub enemy_bullet_layer: u32,
    /// 道具碰撞层
    pub item_layer: u32,
}

/// 道具配置
#[derive(Debug, Serialize, Deserialize)]
pub struct ItemConfig {
    /// 道具掉落概率
    pub drop_rate: f32,
    /// PowerUp 权重
    pub powerup_weight: f32,
    /// ScoreBonus 权重
    pub score_weight: f32,
    /// Bomb 权重
    pub bomb_weight: f32,
    /// Life 权重
    pub life_weight: f32,
    /// Shield 权重
    pub shield_weight: f32,
}

/// 音频配置
#[derive(Debug, Serialize, Deserialize)]
pub struct AudioConfig {
    /// 主音量（0.0 ~ 1.0）
    pub master_volume: f32,
    /// 音效音量（0.0 ~ 1.0）
    pub sfx_volume: f32,
    /// 音频事件到片段路径的映射
    pub event_clip_map: HashMap<AudioEvent, String>,
}

/// 关卡配置项
#[derive(Debug, Serialize, Deserialize)]
pub struct LevelConfigItem {
    /// 初始波次敌人数
    pub base_enemy_count: u32,
    /// 每波递增敌人数
    pub enemy_increment: u32,
    /// 生成间隔（帧）
    pub spawn_interval: u32,
    /// Boss 出现波次间隔
    pub boss_interval: u32,
}

impl Default for StgConfig {
    fn default() -> Self {
        Self {
            game: GameConfig { name: "STG Game".to_string(), version: "0.1.0".to_string(), author: "GG Engine".to_string() },
            display: DisplayConfig { width: 800, height: 600, fullscreen: false, fps_limit: 60 },
            input: InputConfig { move_speed: 3.0, shoot_key: "Space".to_string(), special_key: "Shift".to_string() },
            gameplay: GameplayConfig {
                player_health: 3,
                enemy_spawn_interval: 1000,
                bullet_speed: 5.0,
                enemy_speed: 1.0,
                boss_base_health: 500,
            },
            collision: CollisionConfig {
                cell_size: 64.0,
                player_layer: 1,
                enemy_layer: 4,
                player_bullet_layer: 2,
                enemy_bullet_layer: 4,
                item_layer: 8,
            },
            item: ItemConfig {
                drop_rate: 0.3,
                powerup_weight: 40.0,
                score_weight: 30.0,
                bomb_weight: 15.0,
                life_weight: 5.0,
                shield_weight: 10.0,
            },
            audio: AudioConfig { master_volume: 1.0, sfx_volume: 1.0, event_clip_map: HashMap::new() },
            level: LevelConfigItem { base_enemy_count: 5, enemy_increment: 2, spawn_interval: 60, boss_interval: 5 },
        }
    }
}
