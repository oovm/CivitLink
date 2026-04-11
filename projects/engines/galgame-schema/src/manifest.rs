//! GG Galgame Schema 引擎清单模块
//!
//! 定义 Galgame 引擎清单相关类型，支持从通用引擎清单转换。

use gg_manifest::EngineManifest;
use serde::{Deserialize, Serialize};

/// Galgame 引擎清单
///
/// Galgame 特化的清单视图，可从通用 EngineManifest 转换而来。
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
    #[serde(default)]
    pub default_font: String,
    /// 初始场景
    #[serde(default = "default_initial_scene")]
    pub initial_scene: String,
}

fn default_window_width() -> u32 {
    1280
}

fn default_window_height() -> u32 {
    720
}

fn default_initial_scene() -> String {
    "start".to_string()
}

impl From<&EngineManifest> for GalgameManifest {
    fn from(manifest: &EngineManifest) -> Self {
        Self {
            name: manifest.engine.name.clone(),
            version: manifest.engine.version.clone(),
            plugins: manifest.modules.plugins.clone(),
            default_window_width: manifest.display.width,
            default_window_height: manifest.display.height,
            default_font: String::new(),
            initial_scene: "start".to_string(),
        }
    }
}
