#![warn(missing_docs)]

//! GG Galgame 引擎壳程序
//! 提供 Galgame 游戏的运行时入口点

mod config;
mod engine;

use config::GalgameConfig;
use engine::GalgameEngine;
use gg_core::{GError, GErrorKind, GResult};
use std::path::PathBuf;

fn main() -> GResult<()> {
    let args: Vec<String> = std::env::args().collect();

    let mut is_editor_mode = false;
    let mut project_path = PathBuf::from(".");

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--editor" => {
                is_editor_mode = true;
            }
            "--project" => {
                if i + 1 < args.len() {
                    project_path = PathBuf::from(&args[i + 1]);
                    i += 1;
                }
            }
            _ => {}
        }
        i += 1;
    }

    if is_editor_mode {
        return run_editor_mode();
    }

    let config_path = project_path.join("game.toml");
    let config_content = std::fs::read_to_string(&config_path).map_err(|e| GError {
        kind: GErrorKind::Io,
        message: format!("Failed to read config file {:?}: {}", config_path, e),
    })?;

    let config: GalgameConfig = toml::from_str(&config_content).map_err(|e| GError {
        kind: GErrorKind::Runtime,
        message: format!("Failed to parse config file: {}", e),
    })?;

    let mut engine = GalgameEngine::new(config, is_editor_mode);
    engine.initialize()?;
    engine.run()?;

    Ok(())
}

/// 运行编辑器模式
///
/// 当前为占位实现，后续将实现完整的编辑器面板。
fn run_editor_mode() -> GResult<()> {
    Ok(())
}
