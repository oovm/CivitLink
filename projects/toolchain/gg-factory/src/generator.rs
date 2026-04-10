use gg_core::{GError, GErrorKind, GResult};
use gg_manifest::EngineManifest;
use std::path::{Path, PathBuf};

/// 引擎工厂
///
/// 根据引擎清单生成完整的引擎项目代码，包括 Cargo.toml、main.rs、config.rs 和 engine.rs。
pub struct EngineFactory;

impl EngineFactory {
    /// 根据引擎清单生成引擎项目代码
    ///
    /// 在指定输出目录下生成完整的 Cargo 项目。
    pub fn generate(manifest: &EngineManifest, output_dir: &Path) -> GResult<Vec<PathBuf>> {
        let mut generated = Vec::new();

        std::fs::create_dir_all(output_dir).map_err(|e| GError {
            kind: GErrorKind::Io,
            message: format!("Failed to create output directory {:?}: {}", output_dir, e),
        })?;

        let src_dir = output_dir.join("src");
        std::fs::create_dir_all(&src_dir).map_err(|e| GError {
            kind: GErrorKind::Io,
            message: format!("Failed to create src directory {:?}: {}", src_dir, e),
        })?;

        let cargo_toml_path = output_dir.join("Cargo.toml");
        let main_rs_path = src_dir.join("main.rs");
        let config_rs_path = src_dir.join("config.rs");
        let engine_rs_path = src_dir.join("engine.rs");

        let cargo_toml_content = Self::generate_cargo_toml(manifest);
        let main_rs_content = Self::generate_main_rs(manifest);
        let config_rs_content = Self::generate_config_rs(manifest);
        let engine_rs_content = Self::generate_engine_rs(manifest);

        std::fs::write(&cargo_toml_path, cargo_toml_content)
            .map_err(|e| GError { kind: GErrorKind::Io, message: format!("Failed to write Cargo.toml: {}", e) })?;
        generated.push(cargo_toml_path);

        std::fs::write(&main_rs_path, main_rs_content)
            .map_err(|e| GError { kind: GErrorKind::Io, message: format!("Failed to write main.rs: {}", e) })?;
        generated.push(main_rs_path);

        std::fs::write(&config_rs_path, config_rs_content)
            .map_err(|e| GError { kind: GErrorKind::Io, message: format!("Failed to write config.rs: {}", e) })?;
        generated.push(config_rs_path);

        std::fs::write(&engine_rs_path, engine_rs_content)
            .map_err(|e| GError { kind: GErrorKind::Io, message: format!("Failed to write engine.rs: {}", e) })?;
        generated.push(engine_rs_path);

        Ok(generated)
    }

    /// 生成 Cargo.toml 内容
    fn generate_cargo_toml(manifest: &EngineManifest) -> String {
        let package_name = name_to_package_name(&manifest.engine.name);

        let mut deps = String::new();

        deps.push_str("gg-core = { path = \"../../core/gg-core\" }\n");
        deps.push_str("gg-ecs = { path = \"../../core/gg-ecs\" }\n");
        deps.push_str("gg-asset = { path = \"../../core/gg-asset\" }\n");
        deps.push_str("gg-platform-desktop = { path = \"../../platforms/gg-platform-desktop\" }\n");
        deps.push_str("serde = { version = \"1\", features = [\"derive\"] }\n");
        deps.push_str("toml = \"0.8\"\n");

        for plugin in &manifest.modules.plugins {
            match plugin.as_str() {
                "dialogue" => {
                    deps.push_str("gg-plugin-dialogue = { path = \"../../plugins/gg-plugin-dialogue\" }\n");
                }
                "portrait" => {
                    deps.push_str("gg-plugin-portrait = { path = \"../../plugins/gg-plugin-portrait\" }\n");
                }
                "scene-transition" => {
                    deps.push_str("gg-plugin-scene-transition = { path = \"../../plugins/gg-plugin-scene-transition\" }\n");
                }
                "save" => {
                    deps.push_str("gg-plugin-save = { path = \"../../plugins/gg-plugin-save\" }\n");
                }
                "ui" => {
                    deps.push_str("gg-ui = { path = \"../../plugins/gg-ui\" }\n");
                }
                _ => {
                    let crate_name = format!("gg-plugin-{}", plugin);
                    deps.push_str(&format!("{} = {{ path = \"../../plugins/{}\" }}\n", crate_name, crate_name));
                }
            }
        }

        if !manifest.modules.gom.is_empty() {
            match manifest.modules.gom.as_str() {
                "VisualNovel" => {
                    deps.push_str("gg-galgame-schema = { path = \"../../plugins/gg-galgame-schema\" }\n");
                }
                _ => {
                    let gom_lower = manifest.modules.gom.to_lowercase();
                    let schema_name = format!("gg-{}-schema", gom_lower);
                    deps.push_str(&format!("{} = {{ path = \"../../plugins/{}\" }}\n", schema_name, schema_name));
                }
            }
        }

        format!("[package]\nname = \"{}\"\nversion = \"0.1.0\"\nedition = \"2024\"\n\n[dependencies]\n{}", package_name, deps)
    }

    /// 生成 main.rs 内容
    fn generate_main_rs(manifest: &EngineManifest) -> String {
        let engine_name = &manifest.engine.name;
        let struct_name = name_to_struct_name(&manifest.engine.name);

        format!(
            r#"#![warn(missing_docs)]

//! {engine_name} 引擎壳程序

mod config;
mod engine;

use config::GameConfig;
use engine::{struct_name};
use gg_core::GResult;
use std::path::PathBuf;

fn main() -> GResult<()> {{
    let args: Vec<String> = std::env::args().collect();
    let mut is_editor_mode = false;
    let mut project_path = PathBuf::from(".");
    let mut i = 1;
    while i < args.len() {{
        match args[i].as_str() {{
            "--editor" => is_editor_mode = true,
            "--project" => {{
                if i + 1 < args.len() {{
                    project_path = PathBuf::from(&args[i + 1]);
                    i += 1;
                }}
            }}
            _ => {{}}
        }}
        i += 1;
    }}
    let config_path = project_path.join("game.toml");
    let config_content = std::fs::read_to_string(&config_path).map_err(|e| gg_core::GError {{
        kind: gg_core::GErrorKind::Io,
        message: format!("Failed to read config file {{:?}}: {{}}", config_path, e),
    }})?;
    let config: GameConfig = toml::from_str(&config_content).map_err(|e| gg_core::GError {{
        kind: gg_core::GErrorKind::Runtime,
        message: format!("Failed to parse config file: {{}}", e),
    }})?;
    let mut engine = {struct_name}::new(config, is_editor_mode);
    engine.initialize()?;
    engine.run()
}}
"#
        )
    }

    /// 生成 config.rs 内容
    fn generate_config_rs(manifest: &EngineManifest) -> String {
        let engine_name = &manifest.engine.name;
        let width = manifest.display.width;
        let height = manifest.display.height;
        let fullscreen = manifest.display.fullscreen;

        format!(
            r#"#![warn(missing_docs)]

//! {engine_name} 引擎配置模块

use serde::Deserialize;

/// 游戏信息
#[derive(Debug, Clone, Deserialize)]
pub struct GameSection {{
    /// 游戏名称
    pub name: String,
    /// 版本
    pub version: String,
    /// 初始场景
    pub initial_scene: String,
}}

/// 显示配置
#[derive(Debug, Clone, Deserialize)]
pub struct DisplaySection {{
    /// 宽度
    #[serde(default = "default_width")]
    pub width: u32,
    /// 高度
    #[serde(default = "default_height")]
    pub height: u32,
    /// 全屏
    #[serde(default)]
    pub fullscreen: bool,
}}

fn default_width() -> u32 {{ {width} }}
fn default_height() -> u32 {{ {height} }}

impl Default for DisplaySection {{
    fn default() -> Self {{
        Self {{
            width: {width},
            height: {height},
            fullscreen: {fullscreen},
        }}
    }}
}}

/// 音频配置
#[derive(Debug, Clone, Deserialize)]
pub struct AudioSection {{
    /// 主音量
    #[serde(default = "default_master_volume")]
    pub master_volume: f32,
    /// BGM 音量
    #[serde(default = "default_bgm_volume")]
    pub bgm_volume: f32,
    /// SE 音量
    #[serde(default = "default_se_volume")]
    pub se_volume: f32,
}}

fn default_master_volume() -> f32 {{ 1.0 }}
fn default_bgm_volume() -> f32 {{ 0.8 }}
fn default_se_volume() -> f32 {{ 1.0 }}

impl Default for AudioSection {{
    fn default() -> Self {{
        Self {{
            master_volume: 1.0,
            bgm_volume: 0.8,
            se_volume: 1.0,
        }}
    }}
}}

/// 游戏配置
#[derive(Debug, Clone, Deserialize)]
pub struct GameConfig {{
    /// 游戏信息
    pub game: GameSection,
    /// 显示配置
    #[serde(default)]
    pub display: DisplaySection,
    /// 音频配置
    #[serde(default)]
    pub audio: AudioSection,
}}
"#
        )
    }

    /// 生成 engine.rs 内容
    fn generate_engine_rs(manifest: &EngineManifest) -> String {
        let engine_name = &manifest.engine.name;
        let struct_name = name_to_struct_name(&manifest.engine.name);

        let mut plugin_imports = String::new();
        let mut plugin_init_lines = Vec::new();

        for plugin in &manifest.modules.plugins {
            match plugin.as_str() {
                "dialogue" => {
                    plugin_imports.push_str("use gg_plugin_dialogue::plugin::DialoguePlugin;\n");
                    plugin_init_lines.push("            Box::new(DialoguePlugin)".to_string());
                }
                "portrait" => {
                    plugin_imports.push_str("use gg_plugin_portrait::plugin::PortraitPlugin;\n");
                    plugin_init_lines.push("            Box::new(PortraitPlugin)".to_string());
                }
                "scene-transition" => {
                    plugin_imports.push_str("use gg_plugin_scene_transition::plugin::SceneTransitionPlugin;\n");
                    plugin_init_lines.push("            Box::new(SceneTransitionPlugin)".to_string());
                }
                "save" => {
                    plugin_imports.push_str("use gg_plugin_save::plugin::SavePlugin;\n");
                    plugin_init_lines.push("            Box::new(SavePlugin)".to_string());
                }
                _ => {
                    plugin_init_lines.push(format!(
                        "            // TODO: unknown plugin '{}' - add manual import and initialization",
                        plugin
                    ));
                }
            }
        }

        let plugin_init_block = if plugin_init_lines.is_empty() { String::new() } else { plugin_init_lines.join(",\n") };

        format!(
            r#"#![warn(missing_docs)]

//! {engine_name} 引擎核心模块

use crate::config::GameConfig;
use gg_asset::AssetManager;
use gg_core::plugin::Plugin;
use gg_core::GResult;
use gg_ecs::Scheduler;
use gg_platform_desktop::DesktopFileSystem;

{plugin_imports}
/// {engine_name} 引擎
pub struct {struct_name} {{
    /// 游戏配置
    pub config: GameConfig,
    /// ECS 调度器
    pub scheduler: Scheduler,
    /// 资源管理器
    pub asset_manager: AssetManager,
    /// 是否编辑器模式
    pub is_editor_mode: bool,
}}

impl {struct_name} {{
    /// 创建新的引擎实例
    pub fn new(config: GameConfig, is_editor_mode: bool) -> Self {{
        Self {{
            config,
            scheduler: Scheduler::new(),
            asset_manager: AssetManager::new(Box::new(DesktopFileSystem::new())),
            is_editor_mode,
        }}
    }}

    /// 初始化引擎
    pub fn initialize(&mut self) -> GResult<()> {{
        let plugins: Vec<Box<dyn Plugin>> = vec![
{plugin_init_block}
        ];
        for plugin in plugins {{
            plugin.initialize()?;
        }}
        Ok(())
    }}

    /// 执行一帧
    pub fn tick(&mut self) -> GResult<()> {{
        self.scheduler.tick()
    }}

    /// 运行主循环
    pub fn run(&mut self) -> GResult<()> {{
        self.tick()
    }}
}}
"#
        )
    }
}

/// 将引擎名称转换为 PascalCase 结构体名称
///
/// 例如 "My Galgame" → "MyGalgameEngine"，"my-galgame" → "MyGalgameEngine"
fn name_to_struct_name(name: &str) -> String {
    let mut result = String::new();
    for word in name.split(|c: char| c.is_whitespace() || c == '-' || c == '_') {
        if word.is_empty() {
            continue;
        }
        let mut chars = word.chars();
        if let Some(first) = chars.next() {
            for c in first.to_uppercase() {
                result.push(c);
            }
            for c in chars {
                result.push(c);
            }
        }
    }
    result.push_str("Engine");
    result
}

/// 将引擎名称转换为 Cargo 包名
///
/// 例如 "My Galgame" → "my-galgame"
fn name_to_package_name(name: &str) -> String {
    name.to_lowercase().split(|c: char| c.is_whitespace()).filter(|s| !s.is_empty()).collect::<Vec<_>>().join("-")
}
