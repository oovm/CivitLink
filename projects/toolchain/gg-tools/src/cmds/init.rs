//! `gg init` 命令实现
//!
//! 创建新的引擎项目目录，生成 Engine.toml、game.toml 和子目录。

use crate::{GError, GErrorKind, GResult};
use std::path::PathBuf;

const VISUAL_NOVEL_START_MDX: &str = r#"// 起始场景
label start {
    scene bg school_gate

    say "清晨的阳光洒在校门口，新的一天开始了。"

    character alice "早上好！今天天气真不错呢。"
    character bob "是啊，走吧，要迟到了。"

    choice {
        "和 alice 一起走" => goto walk_with_alice
        "独自前往教室" => goto go_alone
    }
}

label walk_with_alice {
    scene bg school_hallway
    character alice "今天放学后要不要一起去图书馆？"
    say "你决定和 alice 一起走。"
    goto ending
}

label go_alone {
    scene bg classroom
    say "你独自走向教室，思考着今天的课程。"
    goto ending
}

label ending {
    scene bg school_gate
    say "一天结束了。"
}
"#;

const ARPG_START_MDX: &str = r#"// 起始场景
label start {
    scene bg town_square
    say "你站在城镇广场中央，四周人来人往。"

    choice {
        "前往武器店" => goto weapon_shop
        "探索城镇" => goto explore_town
    }
}

label weapon_shop {
    scene bg weapon_shop
    say "武器店的老板向你招手。"
    goto ending
}

label explore_town {
    scene bg town_street
    say "你沿着街道探索，发现了一个隐藏的巷子。"
    goto ending
}

label ending {
    scene bg town_square
    say "冒险才刚刚开始。"
}
"#;

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

    match game_type {
        "VisualNovel" | "vn" => init_visual_novel(&project_dir)?,
        "ARPG" | "arpg" => init_arpg(&project_dir)?,
        _ => {}
    }

    println!("Created engine project '{}' with game type '{}'", engine_name, game_type);
    println!("  Engine.toml - engine manifest");
    println!("  game.toml - game configuration");
    println!("  scripts/ - script directory");
    println!("  assets/ - asset directory");
    Ok(())
}

/// 初始化 VisualNovel 类型的项目模板
///
/// 生成起始对话脚本和资源子目录。
fn init_visual_novel(project_dir: &PathBuf) -> GResult<()> {
    std::fs::write(project_dir.join("scripts/start.mdx"), VISUAL_NOVEL_START_MDX)
        .map_err(|e| GError { kind: GErrorKind::Io, message: format!("Failed to write scripts/start.mdx: {}", e) })?;

    for dir in gg_manifest::visual_novel_asset_dirs() {
        let asset_dir = project_dir.join("assets").join(dir);
        std::fs::create_dir_all(&asset_dir).map_err(|e| GError {
            kind: GErrorKind::Io,
            message: format!("Failed to create assets/{} directory: {}", dir, e),
        })?;
        std::fs::write(asset_dir.join(".gitkeep"), "")
            .map_err(|e| GError { kind: GErrorKind::Io, message: format!("Failed to write assets/{}/.gitkeep: {}", dir, e) })?;
    }

    Ok(())
}

/// 初始化 ARPG 类型的项目模板
///
/// 生成基础脚本和资源子目录。
fn init_arpg(project_dir: &PathBuf) -> GResult<()> {
    std::fs::write(project_dir.join("scripts/start.mdx"), ARPG_START_MDX)
        .map_err(|e| GError { kind: GErrorKind::Io, message: format!("Failed to write scripts/start.mdx: {}", e) })?;

    for dir in gg_manifest::arpg_asset_dirs() {
        let asset_dir = project_dir.join("assets").join(dir);
        std::fs::create_dir_all(&asset_dir).map_err(|e| GError {
            kind: GErrorKind::Io,
            message: format!("Failed to create assets/{} directory: {}", dir, e),
        })?;
        std::fs::write(asset_dir.join(".gitkeep"), "")
            .map_err(|e| GError { kind: GErrorKind::Io, message: format!("Failed to write assets/{}/.gitkeep: {}", dir, e) })?;
    }

    Ok(())
}
