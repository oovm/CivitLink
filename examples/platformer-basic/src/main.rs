//! Platformer 引擎基本示例
//! 展示 Platformer 引擎的基本功能，包括角色移动、跳跃、平台碰撞等

use gg_engine_platformer::engine::PlatformerEngine;
use gg_engine_platformer::config::PlatformerConfig;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("========================================");
    println!("  GG Platformer 引擎 - 基本示例");
    println!("========================================");
    println!("控制说明:");
    println!("  方向键左/右: 移动玩家");
    println!("  空格键: 跳跃");
    println!();

    // 创建游戏配置
    let mut config = PlatformerConfig::default();
    config.game.name = "Platformer Basic Example".to_string();
    config.display.width = 800;
    config.display.height = 600;

    // 创建 Platformer 引擎实例
    let mut engine = PlatformerEngine::new(config, false);

    // 初始化引擎
    engine.initialize()?;

    // 运行游戏
    engine.run()?;

    Ok(())
}
