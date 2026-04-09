//! GG Galgame Schema 引擎清单模块
//! 定义 Galgame 引擎清单相关类型

use serde::{Deserialize, Serialize};

/// Galgame 引擎清单
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GalgameManifest {
    /// 引擎名称
    pub name: String,
    /// 引擎版本
    pub version: String,
    /// 插件列表
    pub plugins: Vec<String>,
    /// 默认窗口宽度
    #[serde(default = "default_window_width")]
    pub default_window_width: u32,
    /// 默认窗口高度
    #[serde(default = "default_window_height")]
    pub default_window_height: u32,
    /// 默认字体路径
    pub default_font: String,
    /// 初始场景
    pub initial_scene: String,
}

fn default_window_width() -> u32 {
    1280
}

fn default_window_height() -> u32 {
    720
}
