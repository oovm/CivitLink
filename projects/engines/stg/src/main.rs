//! STG 引擎主入口
//! 启动 STG 游戏引擎并运行游戏

use gg_engine_stg::engine::StgEngine;
use gg_engine_stg::config::StgConfig;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("========================================");
    println!("  GG STG 引擎 - 射击游戏演示");
    println!("========================================");
    println!();

    // 创建游戏配置
    let config = StgConfig::default();

    // 创建 STG 引擎实例
    let mut engine = StgEngine::new(config, false);

    // 初始化引擎
    engine.initialize()?;

    // 运行游戏
    engine.run()?;

    Ok(())
}
