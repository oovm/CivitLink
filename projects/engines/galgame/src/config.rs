//! Galgame 引擎配置模块
//! 定义游戏配置文件的结构

use serde::Deserialize;

/// 游戏信息
#[derive(Debug, Clone, Deserialize)]
pub struct GameSection {
    /// 游戏名称
    pub name: String,
    /// 版本
    pub version: String,
    /// 初始场景
    pub initial_scene: String,
}

/// 显示配置
#[derive(Debug, Clone, Deserialize)]
pub struct DisplaySection {
    /// 宽度（默认 1280）
    #[serde(default = "default_width")]
    pub width: u32,
    /// 高度（默认 720）
    #[serde(default = "default_height")]
    pub height: u32,
    /// 全屏（默认 false）
    #[serde(default)]
    pub fullscreen: bool,
}

fn default_width() -> u32 {
    1280
}

fn default_height() -> u32 {
    720
}

impl Default for DisplaySection {
    fn default() -> Self {
        Self { width: 1280, height: 720, fullscreen: false }
    }
}

/// 音频配置
#[derive(Debug, Clone, Deserialize)]
pub struct AudioSection {
    /// 主音量（默认 1.0）
    #[serde(default = "default_master_volume")]
    pub master_volume: f32,
    /// BGM 音量（默认 0.8）
    #[serde(default = "default_bgm_volume")]
    pub bgm_volume: f32,
    /// SE 音量（默认 1.0）
    #[serde(default = "default_se_volume")]
    pub se_volume: f32,
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

impl Default for AudioSection {
    fn default() -> Self {
        Self { master_volume: 1.0, bgm_volume: 0.8, se_volume: 1.0 }
    }
}

/// Galgame 游戏配置
#[derive(Debug, Clone, Deserialize)]
pub struct GalgameConfig {
    /// 游戏信息
    pub game: GameSection,
    /// 显示配置
    #[serde(default)]
    pub display: DisplaySection,
    /// 音频配置
    #[serde(default)]
    pub audio: AudioSection,
}
