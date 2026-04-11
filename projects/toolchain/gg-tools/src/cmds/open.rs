//! `gg open` 命令实现
//!
//! 验证并打开已有的 GG 项目目录。

use crate::{GError, GErrorKind, GResult};
use std::path::PathBuf;

/// 执行 `open` 子命令
///
/// 验证目标目录是否为有效的 GG 项目，并输出项目信息。
pub fn cmd_open(path: Option<&str>) -> GResult<()> {
    let project_dir = PathBuf::from(path.unwrap_or("."));

    if !project_dir.exists() {
        return Err(GError {
            kind: GErrorKind::Io,
            message: format!("Directory does not exist: {}", project_dir.display()),
        });
    }

    let has_engine_toml = project_dir.join("Engine.toml").exists();
    let has_game_toml = project_dir.join("game.toml").exists();

    if !has_engine_toml && !has_game_toml {
        return Err(GError {
            kind: GErrorKind::Runtime,
            message: "Not a valid GG project: missing Engine.toml or game.toml".to_string(),
        });
    }

    println!("Opening project: {}", project_dir.display());

    if has_engine_toml {
        match gg_manifest::EngineManifest::load_from_file(&project_dir.join("Engine.toml")) {
            Ok(manifest) => {
                println!("  Engine: {} v{}", manifest.engine.name, manifest.engine.version);
                println!("  Game type: {:?}", manifest.engine.game_type);
                println!("  Resolution: {}x{}", manifest.display.width, manifest.display.height);
            }
            Err(e) => {
                println!("  Warning: Failed to parse Engine.toml: {}", e);
            }
        }
    }

    if has_game_toml {
        match gg_manifest::GameConfig::load_from_file(&project_dir.join("game.toml")) {
            Ok(config) => {
                println!("  Game: {} v{}", config.game.name, config.game.version);
                println!("  Initial scene: {}", config.game.initial_scene);
                println!(
                    "  Display: {}x{}{}",
                    config.display.width,
                    config.display.height,
                    if config.display.fullscreen { " fullscreen" } else { "" }
                );
            }
            Err(e) => {
                println!("  Warning: Failed to parse game.toml: {}", e);
            }
        }
    }

    Ok(())
}
