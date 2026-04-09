//! GG 引擎基本示例
//! 使用 Valkyrie 脚本定义游戏逻辑

use gg_runtime_core::Runtime;
use gg_core::plugin::Plugin;
use gg_vm::VmResult;
use std::sync::Arc;

/// 示例插件
struct ExamplePlugin;

impl Plugin for ExamplePlugin {
    /// 插件名称
    fn name(&self) -> &str {
        "ExamplePlugin"
    }

    /// 初始化插件
    fn initialize(&self) -> gg_core::GResult<()> {
        println!("ExamplePlugin initialized");
        Ok(())
    }

    /// 关闭插件
    fn shutdown(&self) -> gg_core::GResult<()> {
        println!("ExamplePlugin shutdown");
        Ok(())
    }
}

/// Valkyrie 游戏脚本
const GAME_SCRIPT: &str = r#"micro add(x: i32, y: i32) -> i32 { x + y }"#;

fn main() -> gg_core::GResult<()> {
    println!("GG Engine Basic Example");

    let mut runtime = Runtime::new()?;

    let plugin = Arc::new(ExamplePlugin);
    runtime.register_plugin(plugin)?;

    runtime.load_script_string(GAME_SCRIPT, "game")?;
    println!("Valkyrie script loaded");

    runtime.render_system().init()?;

    if runtime.script_engine().has_function("add") {
        match runtime.call_script_function("add") {
            VmResult::Ok | VmResult::Return(_) => {
                println!("Script add executed successfully");
            }
            VmResult::Error(e) => {
                eprintln!("Script add error: {}", e);
            }
            _ => {}
        }
    }

    for i in 0..10 {
        println!("Frame {}", i);
        runtime.tick(std::time::Duration::from_millis(16))?;
        std::thread::sleep(std::time::Duration::from_millis(100));
    }

    println!("Example completed");
    Ok(())
}
