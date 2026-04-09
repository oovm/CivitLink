#![warn(missing_docs)]

//! GG 元游戏引擎命令行工具
//!
//! 提供子命令用于初始化引擎项目、生成引擎代码、构建和创建游戏项目。

use gg_core::{GError, GErrorKind, GResult};
use gg_factory::EngineFactory;
use gg_manifest::EngineManifest;
use std::path::PathBuf;

/// CLI 主入口
fn main() -> GResult<()> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        print_usage();
        return Ok(());
    }
    match args[1].as_str() {
        "init" => cmd_init(&args[2..]),
        "generate" => cmd_generate(&args[2..]),
        "build" => cmd_build(&args[2..]),
        "new-game" => cmd_new_game(&args[2..]),
        "help" | "--help" | "-h" => {
            print_usage();
            Ok(())
        }
        _ => {
            eprintln!("Unknown command: {}", args[1]);
            print_usage();
            Ok(())
        }
    }
}

/// 执行 `init` 子命令
///
/// 创建新的引擎项目目录，生成 Engine.toml、game.toml 和子目录。
fn cmd_init(args: &[String]) -> GResult<()> {
    if args.is_empty() {
        return Err(GError {
            kind: GErrorKind::Runtime,
            message: "Missing engine name. Usage: gg-cli init <engine-name> [--type <game-type>]"
                .to_string(),
        });
    }
    let engine_name = &args[0];
    let game_type = parse_flag(args, "--type").unwrap_or_else(|| "VisualNovel".to_string());

    let manifest = match game_type.as_str() {
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

    let engine_toml_content = toml::to_string_pretty(&manifest).map_err(|e| GError {
        kind: GErrorKind::Runtime,
        message: format!("Failed to serialize Engine.toml: {}", e),
    })?;
    std::fs::write(project_dir.join("Engine.toml"), engine_toml_content).map_err(|e| GError {
        kind: GErrorKind::Io,
        message: format!("Failed to write Engine.toml: {}", e),
    })?;

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
    std::fs::write(project_dir.join("game.toml"), game_toml).map_err(|e| GError {
        kind: GErrorKind::Io,
        message: format!("Failed to write game.toml: {}", e),
    })?;

    std::fs::create_dir_all(project_dir.join("scripts")).map_err(|e| GError {
        kind: GErrorKind::Io,
        message: format!("Failed to create scripts directory: {}", e),
    })?;
    std::fs::create_dir_all(project_dir.join("assets")).map_err(|e| GError {
        kind: GErrorKind::Io,
        message: format!("Failed to create assets directory: {}", e),
    })?;

    println!(
        "Created engine project '{}' with game type '{}'",
        engine_name, game_type
    );
    println!("  Engine.toml - engine manifest");
    println!("  game.toml - game configuration");
    println!("  scripts/ - script directory");
    println!("  assets/ - asset directory");
    Ok(())
}

/// 执行 `generate` 子命令
///
/// 读取 Engine.toml，解析并验证清单，调用工厂生成引擎代码。
fn cmd_generate(args: &[String]) -> GResult<()> {
    let manifest_path = parse_flag(args, "--manifest").unwrap_or_else(|| ".".to_string());
    let path = PathBuf::from(&manifest_path).join("Engine.toml");

    let manifest = EngineManifest::load_from_file(&path)?;
    manifest.validate()?;

    let output_dir = PathBuf::from(&manifest_path).join("generated");
    let files = EngineFactory::generate(&manifest, &output_dir)?;

    println!("Generated engine project for '{}'", manifest.engine.name);
    println!("  Plugins: {}", manifest.modules.plugins.join(", "));
    println!("  Generated files:");
    for file in &files {
        println!("    {}", file.display());
    }
    Ok(())
}

/// 执行 `build` 子命令
///
/// 若 `generated/` 不存在则先自动生成，然后调用 cargo build 构建生成的项目。
fn cmd_build(args: &[String]) -> GResult<()> {
    let manifest_path = parse_flag(args, "--manifest").unwrap_or_else(|| ".".to_string());
    let platform = parse_flag(args, "--platform");
    let release = args.contains(&"--release".to_string());

    let project_dir = PathBuf::from(&manifest_path);
    let generated_dir = project_dir.join("generated");

    if !generated_dir.exists() {
        cmd_generate(args)?;
    }

    let mut cmd = std::process::Command::new("cargo");
    cmd.arg("build");
    cmd.arg("--manifest-path")
        .arg(generated_dir.join("Cargo.toml"));

    if let Some(target) = platform {
        cmd.arg("--target").arg(target);
    }
    if release {
        cmd.arg("--release");
    }

    let status = cmd.status().map_err(|e| GError {
        kind: GErrorKind::Runtime,
        message: format!("Failed to run cargo build: {}", e),
    })?;

    if !status.success() {
        return Err(GError {
            kind: GErrorKind::Runtime,
            message: "Build failed".to_string(),
        });
    }

    println!("Build completed successfully");
    Ok(())
}

/// 执行 `new-game` 子命令
///
/// 创建新的游戏项目目录，生成 game.toml、起始脚本和资源目录。
fn cmd_new_game(args: &[String]) -> GResult<()> {
    if args.is_empty() {
        return Err(GError {
            kind: GErrorKind::Runtime,
            message: "Missing game name. Usage: gg-cli new-game <game-name>".to_string(),
        });
    }
    let game_name = &args[0];
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
    std::fs::write(game_dir.join("game.toml"), game_toml).map_err(|e| GError {
        kind: GErrorKind::Io,
        message: format!("Failed to write game.toml: {}", e),
    })?;

    std::fs::create_dir_all(game_dir.join("scripts")).map_err(|e| GError {
        kind: GErrorKind::Io,
        message: format!("Failed to create scripts directory: {}", e),
    })?;

    let start_script = r#"// Start script
label start {
    say "Hello, World!"
}
"#;
    std::fs::write(game_dir.join("scripts/start.gscript"), start_script).map_err(|e| GError {
        kind: GErrorKind::Io,
        message: format!("Failed to write start.gscript: {}", e),
    })?;

    std::fs::create_dir_all(game_dir.join("assets")).map_err(|e| GError {
        kind: GErrorKind::Io,
        message: format!("Failed to create assets directory: {}", e),
    })?;

    println!("Created game project '{}'", game_name);
    Ok(())
}

/// 从命令行参数中解析指定标志的值
///
/// 查找 `flag` 在 `args` 中的位置，返回其后的参数值。
fn parse_flag(args: &[String], flag: &str) -> Option<String> {
    for i in 0..args.len() {
        if args[i] == flag && i + 1 < args.len() {
            return Some(args[i + 1].clone());
        }
    }
    None
}

/// 打印 CLI 使用说明
fn print_usage() {
    println!("GG Meta-Game Engine CLI");
    println!();
    println!("Usage: gg-cli <command> [options]");
    println!();
    println!("Commands:");
    println!("  init <name> [--type <game-type>]  Initialize a new engine project");
    println!("  generate [--manifest <path>]      Generate engine code from manifest");
    println!(
        "  build [--platform <target>] [--release]  Build the generated engine"
    );
    println!("  new-game <name>                   Create a new game project");
    println!("  help                              Show this help message");
    println!();
    println!("Game types: VisualNovel (vn), ARPG (arpg), Custom (custom)");
}
