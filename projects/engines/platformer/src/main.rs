//! Platformer 引擎主入口
//! 启动 Platformer 游戏引擎并运行游戏

use gg_engine_platformer::engine::PlatformerEngine;
use gg_engine_platformer::config::PlatformerConfig;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("========================================");
    println!("  GG Platformer 引擎 - 平台跳跃游戏演示");
    println!("========================================");
    println!();

    let config = load_config().unwrap_or_else(|e| {
        eprintln!("Failed to load config: {}, using defaults", e);
        PlatformerConfig::default()
    });

    let mut engine = PlatformerEngine::new(config, false);

    engine.initialize()?;

    engine.run()?;

    Ok(())
}

/// 从 game.toml 加载配置
///
/// 依次尝试从当前目录和 template 子目录加载 game.toml，
/// 若均失败则返回错误。
fn load_config() -> Result<PlatformerConfig, Box<dyn std::error::Error>> {
    let paths = ["game.toml", "template/game.toml"];
    for path in &paths {
        if let Ok(content) = std::fs::read_to_string(path) {
            let config: PlatformerConfig = toml::from_str(&content)?;
            println!("Loaded config from: {}", path);
            return Ok(config);
        }
    }
    Err("No game.toml found".into())
}
