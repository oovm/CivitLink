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

        if has_web_target(manifest) {
            let lib_rs_path = src_dir.join("lib.rs");
            let lib_rs_content = Self::generate_lib_rs(manifest);
            std::fs::write(&lib_rs_path, lib_rs_content)
                .map_err(|e| GError { kind: GErrorKind::Io, message: format!("Failed to write lib.rs: {}", e) })?;
            generated.push(lib_rs_path);
        }

        Ok(generated)
    }

    /// 生成 Cargo.toml 内容
    fn generate_cargo_toml(manifest: &EngineManifest) -> String {
        let package_name = name_to_package_name(&manifest.engine.name);

        let has_desktop = has_desktop_target(manifest);
        let has_web = has_web_target(manifest);
        let has_mobile = has_mobile_target(manifest);

        let mut deps = String::new();

        deps.push_str("gg-core = { path = \"../../core/gg-core\" }\n");
        deps.push_str("gg-ecs = { path = \"../../core/gg-ecs\" }\n");
        deps.push_str("gg-asset = { path = \"../../core/gg-asset\" }\n");

        if has_desktop {
            deps.push_str("gg-platform-desktop = { path = \"../../platforms/gg-platform-desktop\", optional = true }\n");
        }
        if has_web {
            deps.push_str("gg-platform-web = { path = \"../../platforms/gg-platform-web\", optional = true }\n");
            deps.push_str("wasm-bindgen = { version = \"0.2\", optional = true }\n");
        }
        if has_mobile {
            deps.push_str("gg-platform-mobile = { path = \"../../platforms/gg-platform-mobile\", optional = true }\n");
        }

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

        if !manifest.toolchain.editor_panels.is_empty() {
            deps.push_str("gg-editor-shell = { path = \"../../editor/gg-editor-shell\" }\n");
            for panel_name in &manifest.toolchain.editor_panels {
                match panel_name.as_str() {
                    "scene-view" => {
                        deps.push_str("gg-editor-scene = { path = \"../../editor/gg-editor-scene\" }\n");
                    }
                    "inspector" => {
                        deps.push_str("gg-editor-inspector = { path = \"../../editor/gg-editor-inspector\" }\n");
                    }
                    "asset-browser" => {
                        deps.push_str("gg-editor-asset-browser = { path = \"../../editor/gg-editor-asset-browser\" }\n");
                    }
                    "preview" => {
                        deps.push_str("gg-editor-preview = { path = \"../../editor/gg-editor-preview\" }\n");
                    }
                    "character" => {
                        deps.push_str("gg-editor-character = { path = \"../../editor/gg-editor-character\" }\n");
                    }
                    "script" => {
                        deps.push_str("gg-editor-script = { path = \"../../editor/gg-editor-script\" }\n");
                    }
                    _ => {}
                }
            }
        }

        let mut features_section = String::new();
        let mut feature_deps = Vec::new();

        if has_desktop {
            feature_deps.push("desktop");
        }
        if has_web {
            feature_deps.push("web");
        }
        if has_mobile {
            feature_deps.push("mobile");
        }

        if !feature_deps.is_empty() {
            features_section.push_str("\n[features]\n");
            features_section.push_str("default = [");
            let default_features: Vec<&str> = feature_deps.iter().map(|s| *s).collect();
            features_section.push_str(&default_features.iter().map(|f| format!("\"{}\"", f)).collect::<Vec<_>>().join(", "));
            features_section.push_str("]\n");

            if has_desktop {
                features_section.push_str("desktop = [\"gg-platform-desktop\"]\n");
            }
            if has_web {
                features_section.push_str("web = [\"gg-platform-web\", \"wasm-bindgen\"]\n");
            }
            if has_mobile {
                features_section.push_str("mobile = [\"gg-platform-mobile\"]\n");
            }
        }

        let mut lib_section = String::new();
        if has_web {
            lib_section.push_str("\n[lib]\ncrate-type = [\"cdylib\"]\n");
        }

        format!(
            "[package]\nname = \"{}\"\nversion = \"0.1.0\"\nedition = \"2024\"\n\n[dependencies]\n{}{}{}",
            package_name, deps, features_section, lib_section
        )
    }

    /// 生成 main.rs 内容
    fn generate_main_rs(manifest: &EngineManifest) -> String {
        let engine_name = &manifest.engine.name;
        let struct_name = name_to_struct_name(&manifest.engine.name);
        let has_web = has_web_target(manifest);

        let cfg_prefix = if has_web { "#[cfg(not(feature = \"web\"))]\n" } else { "" };

        format!(
            r#"#![warn(missing_docs)]

//! {engine_name} 引擎壳程序

mod config;
mod engine;

use config::GameConfig;
use engine::{struct_name};
use gg_core::GResult;
use std::path::PathBuf;

{cfg_prefix}fn main() -> GResult<()> {{
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

        let has_desktop = has_desktop_target(manifest);
        let has_web = has_web_target(manifest);
        let has_mobile = has_mobile_target(manifest);
        let multi_platform = count_platform_types(manifest) > 1;

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

        let platform_imports = generate_platform_imports(has_desktop, has_web, has_mobile, multi_platform);
        let platform_services_init = generate_platform_services_init(has_desktop, has_web, has_mobile, multi_platform);

        let editor_panel_code = generate_editor_panel_code(manifest);

        format!(
            r#"#![warn(missing_docs)]

//! {engine_name} 引擎核心模块

use crate::config::GameConfig;
use gg_asset::AssetManager;
use gg_core::plugin::Plugin;
use gg_core::GResult;
use gg_ecs::Scheduler;
{platform_imports}
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
            asset_manager: AssetManager::new({platform_services_init}),
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
{editor_panel_code}
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

    /// 生成 lib.rs 内容（Web 目标入口）
    fn generate_lib_rs(manifest: &EngineManifest) -> String {
        let engine_name = &manifest.engine.name;
        let struct_name = name_to_struct_name(&manifest.engine.name);

        format!(
            r#"#![warn(missing_docs)]

//! {engine_name} 引擎 Web 入口

mod config;
mod engine;

use config::GameConfig;
use engine::{struct_name};
use gg_core::GResult;

#[cfg_attr(target_arch = "wasm32", wasm_bindgen::prelude::wasm_bindgen(start))]
pub async fn run() -> GResult<()> {{
    let config_content = include_str!("../game.toml");
    let config: GameConfig = toml::from_str(config_content).map_err(|e| gg_core::GError {{
        kind: gg_core::GErrorKind::Runtime,
        message: format!("Failed to parse config file: {{}}", e),
    }})?;
    let mut engine = {struct_name}::new(config, false);
    engine.initialize()?;
    engine.run()
}}
"#
        )
    }
}

/// 判断目标平台是否为桌面平台
fn is_desktop_target(target: &str) -> bool {
    target.contains("windows") || target.contains("darwin") || target.contains("linux")
}

/// 判断目标平台是否为 Web 平台
fn is_web_target(target: &str) -> bool {
    target.contains("wasm32")
}

/// 判断目标平台是否为移动平台
fn is_mobile_target(target: &str) -> bool {
    target.contains("android") || target.contains("ios")
}

/// 判断清单是否包含桌面平台目标
fn has_desktop_target(manifest: &EngineManifest) -> bool {
    manifest.platforms.iter().any(|p| is_desktop_target(&p.target))
}

/// 判断清单是否包含 Web 平台目标
fn has_web_target(manifest: &EngineManifest) -> bool {
    manifest.platforms.iter().any(|p| is_web_target(&p.target))
}

/// 判断清单是否包含移动平台目标
fn has_mobile_target(manifest: &EngineManifest) -> bool {
    manifest.platforms.iter().any(|p| is_mobile_target(&p.target))
}

/// 统计清单中不同平台类型的数量
fn count_platform_types(manifest: &EngineManifest) -> usize {
    let mut count = 0;
    if has_desktop_target(manifest) {
        count += 1;
    }
    if has_web_target(manifest) {
        count += 1;
    }
    if has_mobile_target(manifest) {
        count += 1;
    }
    count
}

/// 生成平台服务 import 语句
fn generate_platform_imports(has_desktop: bool, has_web: bool, has_mobile: bool, multi_platform: bool) -> String {
    let mut imports = String::new();

    if multi_platform {
        if has_desktop {
            imports.push_str("#[cfg(feature = \"desktop\")]\n");
            imports.push_str("use gg_platform_desktop::DesktopPlatformServices;\n");
        }
        if has_web {
            imports.push_str("#[cfg(feature = \"web\")]\n");
            imports.push_str("use gg_platform_web::WebPlatformServices;\n");
        }
        if has_mobile {
            imports.push_str("#[cfg(feature = \"mobile\")]\n");
            imports.push_str("use gg_platform_mobile::MobilePlatformServices;\n");
        }
    }
    else if has_desktop {
        imports.push_str("use gg_platform_desktop::DesktopPlatformServices;\n");
    }
    else if has_web {
        imports.push_str("use gg_platform_web::WebPlatformServices;\n");
    }
    else if has_mobile {
        imports.push_str("use gg_platform_mobile::MobilePlatformServices;\n");
    }

    imports
}

/// 生成平台服务初始化表达式
fn generate_platform_services_init(has_desktop: bool, has_web: bool, has_mobile: bool, multi_platform: bool) -> String {
    if multi_platform {
        let mut branches = Vec::new();

        if has_desktop {
            branches.push("    #[cfg(feature = \"desktop\")]\n    Box::new(DesktopPlatformServices::create())".to_string());
        }
        if has_web {
            branches.push("    #[cfg(feature = \"web\")]\n    Box::new(WebPlatformServices::create(\"/assets/\"))".to_string());
        }
        if has_mobile {
            branches.push("    #[cfg(feature = \"mobile\")]\n    Box::new(MobilePlatformServices::create())".to_string());
        }

        format!("match () {{\n{}\n}}", branches.join("\n"))
    }
    else if has_desktop {
        "Box::new(DesktopPlatformServices::create())".to_string()
    }
    else if has_web {
        "Box::new(WebPlatformServices::create(\"/assets/\"))".to_string()
    }
    else if has_mobile {
        "Box::new(MobilePlatformServices::create())".to_string()
    }
    else {
        "Box::new(DesktopPlatformServices::create())".to_string()
    }
}

/// 生成编辑器面板注册代码
fn generate_editor_panel_code(manifest: &EngineManifest) -> String {
    if manifest.toolchain.editor_panels.is_empty() {
        return String::new();
    }

    let mut imports = String::new();
    let mut register_lines = Vec::new();

    for panel_name in &manifest.toolchain.editor_panels {
        match panel_name.as_str() {
            "scene-view" => {
                imports.push_str("use gg_editor_scene::BaseSceneView;\n");
                register_lines.push("        editor.register_panel(Box::new(BaseSceneView::new()));".to_string());
            }
            "inspector" => {
                imports.push_str("use gg_editor_inspector::InspectorPanel;\n");
                register_lines.push("        editor.register_panel(Box::new(InspectorPanel::new()));".to_string());
            }
            "asset-browser" => {
                imports.push_str("use gg_editor_asset_browser::panel::AssetBrowserPanel;\n");
                register_lines.push("        editor.register_panel(Box::new(AssetBrowserPanel::new()));".to_string());
            }
            "preview" => {
                imports.push_str("use gg_editor_preview::panel::PreviewPanel;\n");
                register_lines.push("        editor.register_panel(Box::new(PreviewPanel::new()));".to_string());
            }
            "character" => {
                imports.push_str("use gg_editor_character::panel::CharacterManagerPanel;\n");
                register_lines.push("        editor.register_panel(Box::new(CharacterManagerPanel::new()));".to_string());
            }
            "script" => {
                imports.push_str("use gg_editor_script::ScriptEditorPanel;\n");
                register_lines.push("        editor.register_panel(Box::new(ScriptEditorPanel::new()));".to_string());
            }
            _ => {}
        }
    }

    if register_lines.is_empty() {
        return String::new();
    }

    let register_block = register_lines.join("\n");

    format!(
        r#"        if self.is_editor_mode {{
{imports}
            let mut editor = gg_editor_shell::EditorShell::new();
{register_block}
        }}"#
    )
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
