use gg_core::{GError, GErrorKind, GResult};
use serde::{Deserialize, Serialize};
use std::path::Path;

fn default_version() -> String {
    "0.1.0".to_string()
}

fn default_vm() -> String {
    "Wasm".to_string()
}

fn default_render() -> String {
    "SpriteStack".to_string()
}

fn default_width() -> u32 {
    1280
}

fn default_height() -> u32 {
    720
}

/// 游戏类型枚举
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum GameType {
    /// 视觉小说
    VisualNovel,
    /// 动作角色扮演
    ARPG,
    /// 弹幕射击
    STG,
    /// 自定义类型
    Custom(String),
}

/// 引擎元数据节
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EngineSection {
    /// 引擎名称（必填，非空）
    pub name: String,
    /// 引擎版本
    #[serde(default = "default_version")]
    pub version: String,
    /// 引擎描述
    #[serde(default)]
    pub description: String,
    /// 游戏类型
    pub game_type: GameType,
}

/// 模块选取节
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModulesSection {
    /// 游戏对象模型（如 "VisualNovel", "ARPG", "STG"）
    #[serde(default)]
    pub gom: String,
    /// 脚本运行时（如 "Wasm", "Lua", "BehaviorTree"）
    #[serde(default = "default_vm")]
    pub vm: String,
    /// 渲染前端（如 "SpriteStack", "Immediate2D"）
    #[serde(default = "default_render")]
    pub render: String,
    /// 插件列表
    #[serde(default)]
    pub plugins: Vec<String>,
}

/// 平台条目
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlatformEntry {
    /// 编译目标三元组
    pub target: String,
    /// 平台显示名称
    pub name: String,
    /// 平台特定的 Cargo features
    #[serde(default)]
    pub features: Vec<String>,
}

/// 工具链扩展点节
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolchainSection {
    /// 编译器扩展步骤
    #[serde(default)]
    pub compiler_steps: Vec<String>,
    /// 编辑器面板列表
    #[serde(default)]
    pub editor_panels: Vec<String>,
}

/// 显示配置节
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
    #[serde(default)]
    pub title: String,
}

impl Default for ModulesSection {
    fn default() -> Self {
        Self {
            gom: String::new(),
            vm: default_vm(),
            render: default_render(),
            plugins: Vec::new(),
        }
    }
}

impl Default for ToolchainSection {
    fn default() -> Self {
        Self {
            compiler_steps: Vec::new(),
            editor_panels: Vec::new(),
        }
    }
}

impl Default for DisplaySection {
    fn default() -> Self {
        Self {
            width: default_width(),
            height: default_height(),
            fullscreen: false,
            title: String::new(),
        }
    }
}

/// 引擎清单
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EngineManifest {
    /// 引擎元数据
    pub engine: EngineSection,
    /// 模块选取
    #[serde(default)]
    pub modules: ModulesSection,
    /// 目标平台列表
    #[serde(default)]
    pub platforms: Vec<PlatformEntry>,
    /// 工具链扩展点
    #[serde(default)]
    pub toolchain: ToolchainSection,
    /// 显示配置
    #[serde(default)]
    pub display: DisplaySection,
}

impl EngineManifest {
    /// 从 TOML 字符串解析引擎清单
    pub fn from_toml(toml_str: &str) -> GResult<Self> {
        toml::from_str(toml_str).map_err(|e| GError {
            kind: GErrorKind::Runtime,
            message: format!("Failed to parse manifest TOML: {}", e),
        })
    }

    /// 验证清单的必填字段
    pub fn validate(&self) -> GResult<()> {
        if self.engine.name.is_empty() {
            return Err(GError {
                kind: GErrorKind::Runtime,
                message: "engine.name must not be empty".to_string(),
            });
        }
        if self.modules.plugins.is_empty() {
            return Err(GError {
                kind: GErrorKind::Runtime,
                message: "modules.plugins must have at least one entry".to_string(),
            });
        }
        if self.platforms.is_empty() {
            return Err(GError {
                kind: GErrorKind::Runtime,
                message: "platforms must have at least one entry".to_string(),
            });
        }
        Ok(())
    }

    /// 从文件路径加载引擎清单
    pub fn load_from_file(path: &Path) -> GResult<Self> {
        let content = std::fs::read_to_string(path).map_err(|e| GError {
            kind: GErrorKind::Io,
            message: format!("Failed to read manifest file {:?}: {}", path, e),
        })?;
        Self::from_toml(&content)
    }
}
