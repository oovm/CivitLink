//! Platformer 引擎主入口
//! 启动 Platformer 游戏引擎并运行游戏

use gg_engine_platformer::engine::PlatformerEngine;
use gg_engine_platformer::config::PlatformerConfig;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("========================================");
    println!("  GG Platformer 引擎 - 平台跳跃游戏演示");
    println!("========================================");
    println!();

    // 创建游戏配置
    let config = PlatformerConfig::default();

    // 创建 Platformer 引擎实例
    let mut engine = PlatformerEngine::new(config, false);

    // 初始化引擎
    engine.initialize()?;

    // 运行游戏
    engine.run()?;

    Ok(())
}
