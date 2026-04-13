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

        deps.push_str("gg-core = { path = \"../../projects/core/gg-core\" }\n");
        deps.push_str("gg-ecs = { path = \"../../projects/core/gg-ecs\" }\n");
        deps.push_str("gg-asset = { path = \"../../projects/core/gg-asset\" }\n");
        deps.push_str("gg-runtime = { path = \"../../projects/runtime/gg-runtime\" }\n");
        deps.push_str("gg-schedule = { path = \"../../projects/core/gg-schedule\" }\n");

        if has_desktop {
            deps.push_str("gg-platform-desktop = { path = \"../../projects/platforms/gg-platform-desktop\", optional = true }\n");
        }
        if has_web {
            deps.push_str("gg-platform-web = { path = \"../../projects/platforms/gg-platform-web\", optional = true }\n");
            deps.push_str("wasm-bindgen = { version = \"0.2\", optional = true }\n");
        }
        if has_mobile {
            deps.push_str("gg-platform-ios = { path = "../../projects/compiler/gg-platform-ios", optional = true }\n");
            deps.push_str("gg-platform-android = { path = "../../projects/compiler/gg-platform-android", optional = true }\n");
        }

        deps.push_str("serde = { version = \"1\", features = [\"derive\"] }\n");
        deps.push_str("toml = \"0.8\"\n");

        for plugin in &manifest.modules.plugins {
            match plugin.as_str() {
                "dialogue" => {
                    deps.push_str("gg-plugin-dialogue = { path = \"../../projects/plugins/gg-plugin-dialogue\" }\n");
                }
                "portrait" => {
                    deps.push_str("gg-plugin-portrait = { path = \"../../projects/plugins/gg-plugin-portrait\" }\n");
                }
                "scene-transition" => {
                    deps.push_str("gg-plugin-scene-transition = { path = \"../../projects/plugins/gg-plugin-scene-transition\" }\n");
                }
                "save" => {
                    deps.push_str("gg-plugin-save = { path = \"../../projects/plugins/gg-plugin-save\" }\n");
                }
                "ui" => {
                    deps.push_str("gg-ui = { path = \"../../projects/plugins/gg-ui\" }\n");
                }
                _ => {
                    let crate_name = format!("gg-plugin-{}", plugin);
                    deps.push_str(&format!("{} = {{ path = \"../../projects/plugins/{}\" }}\n", crate_name, crate_name));
                }
            }
        }

        if !manifest.modules.gom.is_empty() {
            match manifest.modules.gom.as_str() {
                "VisualNovel" => {
                    deps.push_str("galgame-schema = { path = \"../../projects/plugins/galgame-schema\" }\n");
                }
                _ => {
                    let gom_lower = manifest.modules.gom.to_lowercase();
                    let schema_name = format!("gg-{}-schema", gom_lower);
                    deps.push_str(&format!("{} = {{ path = \"../../projects/plugins/{}\" }}\n", schema_name, schema_name));
                }
            }
        }

        if !manifest.toolchain.editor_panels.is_empty() {
            deps.push_str("gg-editor-shell = { path = \"../../projects/editor/gg-editor-shell\" }\n");
            for panel_name in &manifest.toolchain.editor_panels {
                match panel_name.as_str() {
                    "scene-view" => {
                        deps.push_str("gg-editor-scene = { path = \"../../projects/editor/gg-editor-scene\" }\n");
                    }
                    "inspector" => {
                        deps.push_str("gg-editor-inspector = { path = \"../../projects/editor/gg-editor-inspector\" }\n");
                    }
                    "asset-browser" => {
                        deps.push_str("gg-editor-asset-browser = { path = \"../../projects/editor/gg-editor-asset-browser\" }\n");
                    }
                    "preview" => {
                        deps.push_str("gg-editor-preview = { path = \"../../projects/editor/gg-editor-preview\" }\n");
                    }
                    "character" => {
                        deps.push_str("gg-editor-character = { path = \"../../projects/editor/gg-editor-character\" }\n");
                    }
                    "script" => {
                        deps.push_str("gg-editor-script = { path = \"../../projects/editor/gg-editor-script\" }\n");
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
                features_section.push_str("ios = [\"gg-platform-ios\"]\n");
                features_section.push_str("android = [\"gg-platform-android\"]\n");
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
        let has_web = has_web_target(manifest);

        let cfg_prefix = if has_web { "#[cfg(not(feature = \"web\"))]\n" } else { "" };

        format!(
            r#"#![warn(missing_docs)]

//! {engine_name} 引擎壳程序

mod config;
mod engine;

use gg_core::GResult;

{cfg_prefix}fn main() -> GResult<()> {{
    engine::run()
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

        let mut plugin_imports = String::new();
        let mut plugin_add_lines = Vec::new();

        for plugin in &manifest.modules.plugins {
            match plugin.as_str() {
                "dialogue" => {
                    plugin_imports.push_str("use gg_plugin_dialogue::plugin::DialoguePlugin;\n");
                    plugin_add_lines.push("    app.add_plugin(Box::new(DialoguePlugin));".to_string());
                }
                "portrait" => {
                    plugin_imports.push_str("use gg_plugin_portrait::plugin::PortraitPlugin;\n");
                    plugin_add_lines.push("    app.add_plugin(Box::new(PortraitPlugin));".to_string());
                }
                "scene-transition" => {
                    plugin_imports.push_str("use gg_plugin_scene_transition::plugin::SceneTransitionPlugin;\n");
                    plugin_add_lines.push("    app.add_plugin(Box::new(SceneTransitionPlugin));".to_string());
                }
                "save" => {
                    plugin_imports.push_str("use gg_plugin_save::plugin::SavePlugin;\n");
                    plugin_add_lines.push("    app.add_plugin(Box::new(SavePlugin));".to_string());
                }
                _ => {}
            }
        }

        let plugin_add_block = plugin_add_lines.join("\n");

        let system_registration = generate_system_registration(&manifest.modules.gom);
        let component_registration = generate_component_registration(&manifest.modules.gom);
        let asset_loader_registration = generate_asset_loader_registration();
        let editor_panel_code = generate_editor_panel_code_app(manifest);

        format!(
            r#"#![warn(missing_docs)]

//! {engine_name} 引擎核心模块

use gg_core::GResult;
use gg_runtime_core::App;
use gg_schedule::SystemSet;
{plugin_imports}
/// 运行 {engine_name} 引擎
pub fn run() -> GResult<()> {{
    let mut app = App::new();

    // 注册插件
{plugin_add_block}

    // 注册系统
{system_registration}

    // 注册组件
{component_registration}

    // 注册资源加载器
{asset_loader_registration}
{editor_panel_code}
    app.run()
}}
"#
        )
    }

    /// 生成 lib.rs 内容（Web 目标入口）
    fn generate_lib_rs(manifest: &EngineManifest) -> String {
        let engine_name = &manifest.engine.name;

        format!(
            r#"#![warn(missing_docs)]

//! {engine_name} 引擎 Web 入口

mod config;
mod engine;

use gg_core::GResult;

#[cfg_attr(target_arch = "wasm32", wasm_bindgen::prelude::wasm_bindgen(start))]
pub async fn run() -> GResult<()> {{
    engine::run()
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

/// 根据 GOM 类型生成系统注册代码
pub fn generate_system_registration(gom: &str) -> String {
    if gom.is_empty() {
        return String::new();
    }

    let mut lines = Vec::new();

    match gom {
        "VisualNovel" => {
            lines.push("    app.add_system(SystemSet::Update, gg_plugin_dialogue::systems::dialogue_system);".to_string());
            lines.push("    app.add_system(SystemSet::Update, gg_plugin_portrait::systems::portrait_system);".to_string());
            lines.push("    app.add_system(SystemSet::Update, gg_plugin_scene_transition::systems::scene_transition_system);".to_string());
        }
        "Platformer" => {
            lines.push("    app.add_system(SystemSet::Update, gg_ecs::systems::movement_system);".to_string());
            lines.push("    app.add_system(SystemSet::Update, gg_ecs::systems::physics_system);".to_string());
        }
        "STG" => {
            lines.push("    app.add_system(SystemSet::Update, gg_ecs::systems::bullet_system);".to_string());
            lines.push("    app.add_system(SystemSet::Update, gg_ecs::systems::collision_system);".to_string());
        }
        _ => {}
    }

    lines.join("\n")
}

/// 根据 GOM 类型生成组件注册代码
pub fn generate_component_registration(gom: &str) -> String {
    if gom.is_empty() {
        return String::new();
    }

    let mut lines = Vec::new();

    match gom {
        "VisualNovel" => {
            lines.push("    app.register_component::<gg_plugin_dialogue::components::DialogueComponent>();".to_string());
            lines.push("    app.register_component::<gg_plugin_portrait::components::PortraitComponent>();".to_string());
            lines.push("    app.register_component::<gg_plugin_scene_transition::components::SceneTransitionComponent>();".to_string());
        }
        "Platformer" => {
            lines.push("    app.register_component::<gg_ecs::components::TransformComponent>();".to_string());
            lines.push("    app.register_component::<gg_ecs::components::VelocityComponent>();".to_string());
            lines.push("    app.register_component::<gg_ecs::components::ColliderComponent>();".to_string());
        }
        "STG" => {
            lines.push("    app.register_component::<gg_ecs::components::TransformComponent>();".to_string());
            lines.push("    app.register_component::<gg_ecs::components::BulletComponent>();".to_string());
            lines.push("    app.register_component::<gg_ecs::components::HitboxComponent>();".to_string());
        }
        _ => {}
    }

    lines.join("\n")
}

/// 生成资源加载器注册代码
pub fn generate_asset_loader_registration() -> String {
    let lines = vec![
        "    app.register_loader::<gg_asset::loader::TextLoader>();",
        "    app.register_loader::<gg_asset::loader::BinaryLoader>();",
        "    app.register_loader::<gg_asset::loader::ImageLoader>();",
        "    app.register_loader::<gg_asset::loader::AudioLoader>();",
    ];
    lines.join("\n")
}

/// 生成编辑器面板注册代码（App builder 模式）
fn generate_editor_panel_code_app(manifest: &EngineManifest) -> String {
    if manifest.toolchain.editor_panels.is_empty() {
        return String::new();
    }

    let mut register_lines = Vec::new();

    for panel_name in &manifest.toolchain.editor_panels {
        match panel_name.as_str() {
            "scene-view" => {
                register_lines.push("    app.add_editor_panel(Box::new(gg_editor_scene::BaseSceneView::new()));".to_string());
            }
            "inspector" => {
                register_lines.push("    app.add_editor_panel(Box::new(gg_editor_inspector::InspectorPanel::new()));".to_string());
            }
            "asset-browser" => {
                register_lines.push("    app.add_editor_panel(Box::new(gg_editor_asset_browser::panel::AssetBrowserPanel::new()));".to_string());
            }
            "preview" => {
                register_lines.push("    app.add_editor_panel(Box::new(gg_editor_preview::panel::PreviewPanel::new()));".to_string());
            }
            "character" => {
                register_lines.push("    app.add_editor_panel(Box::new(gg_editor_character::panel::CharacterManagerPanel::new()));".to_string());
            }
            "script" => {
                register_lines.push("    app.add_editor_panel(Box::new(gg_editor_script::ScriptEditorPanel::new()));".to_string());
            }
            _ => {}
        }
    }

    if register_lines.is_empty() {
        return String::new();
    }

    let register_block = register_lines.join("\n");

    format!(
        r#"
    // 注册编辑器面板
{register_block}"#
    )
}

/// 将引擎名称转换为 PascalCase 结构体名称
///
/// 例如 "My Galgame" → "MyGalgameEngine"，"my-galgame" → "MyGalgameEngine"
pub fn name_to_struct_name(name: &str) -> String {
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
pub fn name_to_package_name(name: &str) -> String {
    name.to_lowercase().split(|c: char| c.is_whitespace()).filter(|s| !s.is_empty()).collect::<Vec<_>>().join("-")
}
