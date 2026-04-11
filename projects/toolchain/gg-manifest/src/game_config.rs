//! game.toml 配置定义
//!
//! 定义游戏配置文件格式，提供解析、序列化和默认值功能。

use serde::{Deserialize, Serialize};

fn default_game_name() -> String {
    "untitled".to_string()
}

fn default_version() -> String {
    "0.1.0".to_string()
}

fn default_initial_scene() -> String {
    "start".to_string()
}

fn default_width() -> u32 {
    1280
}

fn default_height() -> u32 {
    720
}

fn default_title() -> String {
    "GG Game".to_string()
}

fn default_master_volume() -> f32 {
    1.0
}

fn default_bgm_volume() -> f32 {
    0.8
}

fn default_se_volume() -> f32 {
    1.0
}

/// game.toml 的 game 节
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameSection {
    /// 游戏名称
    #[serde(default = "default_game_name")]
    pub name: String,
    /// 游戏版本
    #[serde(default = "default_version")]
    pub version: String,
    /// 初始场景标识
    #[serde(default = "default_initial_scene")]
    pub initial_scene: String,
}

/// game.toml 的 display 节
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DisplaySection {
    /// 窗口宽度
    #[serde(default = "default_width")]
    pub width: u32,
    /// 窗口高度
    #[serde(default = "default_height")]
    pub height: u32,
    /// 是否全屏
    #[serde(default)]
    pub fullscreen: bool,
    /// 窗口标题
    #[serde(default = "default_title")]
    pub title: String,
}

/// game.toml 的 audio 节
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioSection {
    /// 主音量
    #[serde(default = "default_master_volume")]
    pub master_volume: f32,
    /// 背景音乐音量
    #[serde(default = "default_bgm_volume")]
    pub bgm_volume: f32,
    /// 音效音量
    #[serde(default = "default_se_volume")]
    pub se_volume: f32,
}

/// 从 game.toml 解析的游戏配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameConfig {
    /// 游戏信息节
    #[serde(default)]
    pub game: GameSection,
    /// 显示配置节
    #[serde(default)]
    pub display: DisplaySection,
    /// 音频配置节
    #[serde(default)]
    pub audio: AudioSection,
}

impl Default for GameSection {
    fn default() -> Self {
        Self {
            name: default_game_name(),
            version: default_version(),
            initial_scene: default_initial_scene(),
        }
    }
}

impl Default for DisplaySection {
    fn default() -> Self {
        Self {
            width: default_width(),
            height: default_height(),
            fullscreen: false,
            title: default_title(),
        }
    }
}

impl Default for AudioSection {
    fn default() -> Self {
        Self {
            master_volume: default_master_volume(),
            bgm_volume: default_bgm_volume(),
            se_volume: default_se_volume(),
        }
    }
}

impl Default for GameConfig {
    fn default() -> Self {
        Self {
            game: GameSection::default(),
            display: DisplaySection::default(),
            audio: AudioSection::default(),
        }
    }
}

impl GameConfig {
    /// 从 TOML 文件加载游戏配置
    pub fn load_from_file(path: &std::path::Path) -> Result<GameConfig, Box<dyn std::error::Error>> {
        let content = std::fs::read_to_string(path)?;
        let config: GameConfig = toml::from_str(&content)?;
        Ok(config)
    }

    /// 将游戏配置保存为 TOML 文件
    pub fn save_to_file(&self, path: &std::path::Path) -> Result<(), Box<dyn std::error::Error>> {
        let content = toml::to_string_pretty(self)?;
        std::fs::write(path, content)?;
        Ok(())
    }
}
