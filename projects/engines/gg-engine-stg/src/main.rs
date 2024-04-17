//! STG 引擎主入口
//! 启动 STG 游戏引擎并运行游戏

use gg_engine_stg::{config::StgConfig, engine::StgEngine};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("========================================");
    println!("  GG STG 引擎 - 射击游戏演示");
    println!("========================================");
    println!();

    let config = StgConfig::default();
    let mut engine = StgEngine::new(config, false);
    engine.initialize()?;

    #[cfg(feature = "runtime")]
    engine.run()?;

    #[cfg(not(feature = "runtime"))]
    {
        println!("Runtime feature not enabled. Running headless tick...");
        for _ in 0..10 {
            engine.tick()?;
        }
        println!("Headless simulation complete.");
    }

    Ok(())
}
