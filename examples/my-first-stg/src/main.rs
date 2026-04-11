//! STG 引擎基本示例
//! 展示 STG 引擎的基本功能，包括玩家控制、敌人 AI、碰撞检测等

use gg_engine_stg::engine::StgEngine;
use gg_engine_stg::config::StgConfig;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("========================================");
    println!("  GG STG 引擎 - 基本示例");
    println!("========================================");
    println!("控制说明:");
    println!("  方向键: 移动玩家");
    println!("  空格键: 射击");
    println!("  Shift: 特殊武器");
    println!();

    // 创建游戏配置
    let mut config = StgConfig::default();
    config.game.name = "STG Basic Example".to_string();
    config.display.width = 800;
    config.display.height = 600;

    // 创建 STG 引擎实例
    let mut engine = StgEngine::new(config, false);

    // 初始化引擎
    engine.initialize()?;

    // 运行游戏
    engine.run()?;

    Ok(())
}
