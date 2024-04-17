#![warn(missing_docs)]

//! GG Galgame 引擎壳程序
//! 提供 Galgame 游戏的运行时入口点

mod compiler;
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

    let config: GalgameConfig = toml::from_str(&config_content)
        .map_err(|e| GError { kind: GErrorKind::Runtime, message: format!("Failed to parse config file: {}", e) })?;

    let mut engine = GalgameEngine::new(config, is_editor_mode, Some(project_path));
    engine.initialize()?;
    engine.run()?;

    Ok(())
}

/// 运行编辑器模式
///
/// 创建 EditorShell，注册场景视图、属性检查器和资源浏览器面板，
/// 然后启动 winit 桌面事件循环。
fn run_editor_mode() -> GResult<()> {
    let mut shell = gg_editor_shell::EditorShell::new();

    shell.register_panel(Box::new(gg_editor_scene::BaseSceneView::new()));
    shell.register_panel(Box::new(gg_editor_inspector::InspectorPanel::new()));
    shell.register_panel(Box::new(gg_editor_asset_browser::AssetBrowserPanel::new()));

    shell.run_with_renderer()
}
