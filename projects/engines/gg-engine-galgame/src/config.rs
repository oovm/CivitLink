//! Galgame 引擎配置模块
//! 定义游戏配置文件的结构

use serde::{Deserialize, Serialize};

/// 游戏信息
#[derive(Debug, Clone, Deserialize)]
pub struct GameSection {
    /// 游戏名称
    #[serde(default = "default_game_name")]
    pub name: String,
    /// 版本
    #[serde(default = "default_game_version")]
    pub version: String,
    /// 初始场景
    #[serde(default = "default_initial_scene")]
    pub initial_scene: String,
}

fn default_game_name() -> String {
    "Untitled".to_string()
}

fn default_game_version() -> String {
    "0.1.0".to_string()
}

fn default_initial_scene() -> String {
    "start".to_string()
}

impl Default for GameSection {
    fn default() -> Self {
        Self { name: default_game_name(), version: default_game_version(), initial_scene: default_initial_scene() }
    }
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

/// UI 主题颜色配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiThemeColors {
    /// 对话面板背景色（RGBA）
    #[serde(default = "default_panel_bg")]
    pub panel_background: [f32; 4],
    /// 默认说话者名称颜色（RGBA）
    #[serde(default = "default_speaker_color")]
    pub speaker_color: [f32; 4],
    /// 默认对话文本颜色（RGBA）
    #[serde(default = "default_text_color")]
    pub text_color: [f32; 4],
    /// 选项按钮背景色（RGBA）
    #[serde(default = "default_choice_bg")]
    pub choice_background: [f32; 4],
    /// 选项按钮文本颜色（RGBA）
    #[serde(default = "default_choice_text_color")]
    pub choice_text_color: [f32; 4],
}

fn default_panel_bg() -> [f32; 4] {
    [0.0, 0.0, 0.0, 0.7]
}

fn default_speaker_color() -> [f32; 4] {
    [1.0, 0.9, 0.3, 1.0]
}

fn default_text_color() -> [f32; 4] {
    [1.0, 1.0, 1.0, 1.0]
}

fn default_choice_bg() -> [f32; 4] {
    [0.2, 0.2, 0.4, 0.9]
}

fn default_choice_text_color() -> [f32; 4] {
    [1.0, 1.0, 1.0, 1.0]
}

impl Default for UiThemeColors {
    fn default() -> Self {
        Self {
            panel_background: default_panel_bg(),
            speaker_color: default_speaker_color(),
            text_color: default_text_color(),
            choice_background: default_choice_bg(),
            choice_text_color: default_choice_text_color(),
        }
    }
}

/// UI 主题字体配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiThemeFonts {
    /// 说话者名称字体大小
    #[serde(default = "default_speaker_size")]
    pub speaker_size: f32,
    /// 对话文本字体大小
    #[serde(default = "default_text_size")]
    pub text_size: f32,
    /// 选项按钮字体大小
    #[serde(default = "default_choice_size")]
    pub choice_size: f32,
}

fn default_speaker_size() -> f32 {
    20.0
}

fn default_text_size() -> f32 {
    18.0
}

fn default_choice_size() -> f32 {
    16.0
}

impl Default for UiThemeFonts {
    fn default() -> Self {
        Self { speaker_size: default_speaker_size(), text_size: default_text_size(), choice_size: default_choice_size() }
    }
}

/// UI 主题配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiTheme {
    /// 颜色配置
    #[serde(default)]
    pub colors: UiThemeColors,
    /// 字体配置
    #[serde(default)]
    pub fonts: UiThemeFonts,
    /// 对话面板高度（像素）
    #[serde(default = "default_panel_height")]
    pub panel_height: f32,
}

fn default_panel_height() -> f32 {
    200.0
}

impl Default for UiTheme {
    fn default() -> Self {
        Self { colors: UiThemeColors::default(), fonts: UiThemeFonts::default(), panel_height: default_panel_height() }
    }
}

/// Galgame 游戏配置
#[derive(Debug, Clone, Deserialize)]
pub struct GalgameConfig {
    /// 游戏信息
    #[serde(default)]
    pub game: GameSection,
    /// 显示配置
    #[serde(default)]
    pub display: DisplaySection,
    /// 音频配置
    #[serde(default)]
    pub audio: AudioSection,
    /// UI 主题配置
    #[serde(default)]
    pub ui: UiTheme,
}

impl Default for GalgameConfig {
    fn default() -> Self {
        Self {
            game: GameSection::default(),
            display: DisplaySection::default(),
            audio: AudioSection::default(),
            ui: UiTheme::default(),
        }
    }
}
