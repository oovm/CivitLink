//! GG 引擎基本示例
//! 使用 RuntimeBuilder 构建运行时，演示阶段调度器和脚本集成

use gg_core::plugin::Plugin;
use gg_ecs::World;
use gg_runtime::{RuntimeBuilder, Stage};
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
const GAME_SCRIPT: &str = r#"
micro init() {
    let entity1 = spawn_entity()
    add_component(entity1, "Position")
    set_field(entity1, "Position", "x", 0.0)
    set_field(entity1, "Position", "y", 0.0)
    add_component(entity1, "Velocity")
    set_field(entity1, "Velocity", "dx", 1.0)
    set_field(entity1, "Velocity", "dy", 1.0)
    print("Game initialized with entity")
}

micro update() {
    print("Update tick")
}
"#;

/// 启动阶段系统：打印启动信息
fn startup_system(_world: &mut World) -> gg_core::GResult<()> {
    println!("[Startup] Game systems initializing...");
    Ok(())
}

/// 更新阶段系统：打印帧计数
fn update_system(_world: &mut World) -> gg_core::GResult<()> {
    println!("[Update] Processing game logic...");
    Ok(())
}

fn main() -> gg_core::GResult<()> {
    println!("GG Engine Basic Example");

    let mut runtime = RuntimeBuilder::new().build()?;

    let plugin = Arc::new(ExamplePlugin);
    runtime.register_plugin(plugin)?;

    runtime.stage_scheduler_mut().add_system_to_stage("startup", Box::new(startup_system), Stage::Startup);
    runtime.stage_scheduler_mut().add_system_to_stage("game_logic", Box::new(update_system), Stage::Update);

    runtime.load_script_string(GAME_SCRIPT, "game")?;
    println!("Valkyrie script loaded");

    if runtime.script_engine().has_function("init") {
        match runtime.call_script_function("init") {
            VmResult::Ok | VmResult::Return(_) => {
                println!("Script init executed successfully");
            }
            VmResult::Error { message, source_location: _ } => {
                eprintln!("Script init error: {}", message);
            }
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
