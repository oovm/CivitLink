//! `gg init` 命令实现
//!
//! 创建新的引擎项目目录，生成 Engine.toml、game.toml 和子目录。

use crate::GError;
use crate::GErrorKind;
use crate::GResult;
use gg_manifest::EngineManifest;
use std::path::PathBuf;

/// 执行 `init` 子命令
///
/// 创建新的引擎项目目录，生成 Engine.toml、game.toml 和子目录。
pub fn cmd_init(engine_name: &str, game_type: &str) -> GResult<()> {
    let manifest = match game_type {
        "VisualNovel" | "vn" => gg_manifest::visual_novel_template(engine_name),
        "ARPG" | "arpg" => gg_manifest::arpg_template(engine_name),
        "Custom" | "custom" => gg_manifest::custom_template(engine_name),
        _ => gg_manifest::custom_template(engine_name),
    };

    let project_dir = PathBuf::from(engine_name);
    std::fs::create_dir_all(&project_dir).map_err(|e| GError {
        kind: GErrorKind::Io,
        message: format!("Failed to create project directory '{}': {}", engine_name, e),
    })?;

    let engine_toml_content = toml::to_string_pretty(&manifest)
        .map_err(|e| GError { kind: GErrorKind::Runtime, message: format!("Failed to serialize Engine.toml: {}", e) })?;
    std::fs::write(project_dir.join("Engine.toml"), engine_toml_content)
        .map_err(|e| GError { kind: GErrorKind::Io, message: format!("Failed to write Engine.toml: {}", e) })?;

    let game_toml = format!(
        r#"[game]
name = "{}"
version = "0.1.0"
initial_scene = "start"

[display]
width = {}
height = {}
fullscreen = false

[audio]
master_volume = 1.0
bgm_volume = 0.8
se_volume = 1.0
"#,
        engine_name, manifest.display.width, manifest.display.height
    );
    std::fs::write(project_dir.join("game.toml"), game_toml)
        .map_err(|e| GError { kind: GErrorKind::Io, message: format!("Failed to write game.toml: {}", e) })?;

    std::fs::create_dir_all(project_dir.join("scripts"))
        .map_err(|e| GError { kind: GErrorKind::Io, message: format!("Failed to create scripts directory: {}", e) })?;
    std::fs::create_dir_all(project_dir.join("assets"))
        .map_err(|e| GError { kind: GErrorKind::Io, message: format!("Failed to create assets directory: {}", e) })?;

    println!("Created engine project '{}' with game type '{}'", engine_name, game_type);
    println!("  Engine.toml - engine manifest");
    println!("  game.toml - game configuration");
    println!("  scripts/ - script directory");
    println!("  assets/ - asset directory");
    Ok(())
}
