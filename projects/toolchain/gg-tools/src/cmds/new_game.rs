//! `gg new-game` 命令实现
//!
//! 创建新的游戏项目目录，生成 game.toml、起始脚本和资源目录。

use crate::{GError, GErrorKind, GResult};
use std::path::PathBuf;

/// 执行 `new-game` 子命令
///
/// 创建新的游戏项目目录，生成 game.toml、起始脚本和资源目录。
pub fn cmd_new_game(game_name: &str) -> GResult<()> {
    let game_dir = PathBuf::from(game_name);

    std::fs::create_dir_all(&game_dir).map_err(|e| GError {
        kind: GErrorKind::Io,
        message: format!("Failed to create game directory '{}': {}", game_name, e),
    })?;

    let game_toml = format!(
        r#"[game]
name = "{}"
version = "0.1.0"
initial_scene = "start"

[display]
width = 1280
height = 720
fullscreen = false

[audio]
master_volume = 1.0
bgm_volume = 0.8
se_volume = 1.0
"#,
        game_name
    );
    std::fs::write(game_dir.join("game.toml"), game_toml)
        .map_err(|e| GError { kind: GErrorKind::Io, message: format!("Failed to write game.toml: {}", e) })?;

    std::fs::create_dir_all(game_dir.join("scripts"))
        .map_err(|e| GError { kind: GErrorKind::Io, message: format!("Failed to create scripts directory: {}", e) })?;

    let start_script = r#"// Start script
label start {
    say "Hello, World!"
}
"#;
    std::fs::write(game_dir.join("scripts/start.gscript"), start_script)
        .map_err(|e| GError { kind: GErrorKind::Io, message: format!("Failed to write start.gscript: {}", e) })?;

    std::fs::create_dir_all(game_dir.join("assets"))
        .map_err(|e| GError { kind: GErrorKind::Io, message: format!("Failed to create assets directory: {}", e) })?;

    println!("Created game project '{}'", game_name);
    Ok(())
}
