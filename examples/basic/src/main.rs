//! GG 引擎基本示例
//! 测试引擎的基本功能

use gg_runtime_core::{Runtime, EntityCountSystem, MovementSystem, Position, Velocity};
use gg_core::plugin::Plugin;
use gg_render::RenderComponent;
use std::sync::Arc;

/// 示例插件
struct ExamplePlugin;

impl Plugin for ExamplePlugin {
    fn name(&self) -> &str {
        "ExamplePlugin"
    }
    
    fn initialize(&self) -> gg_core::GResult<()> {
        println!("ExamplePlugin initialized");
        Ok(())
    }
    
    fn shutdown(&self) -> gg_core::GResult<()> {
        println!("ExamplePlugin shutdown");
        Ok(())
    }
}

fn main() -> gg_core::GResult<()> {
    println!("GG Engine Basic Example");
    
    // 创建运行时
    let mut runtime = Runtime::new()?;
    
    // 注册插件
    let plugin = Arc::new(ExamplePlugin);
    runtime.register_plugin(plugin)?;
    
    // 初始化渲染系统
    runtime.render_system().init()?;
    
    // 注册系统和创建实体
    {
        let world = runtime.scheduler().world();
        
        // 注册系统
        world.register_system(Box::new(EntityCountSystem::new()));
        world.register_system(Box::new(MovementSystem::new()));
        
        // 创建实体
        let entity1 = world.spawn();
        world.add_component(entity1, Position { x: 0.0, y: 0.0 })?;
        world.add_component(entity1, Velocity { dx: 1.0, dy: 1.0 })?;
        world.add_component(entity1, RenderComponent { width: 50.0, height: 50.0, color: [1.0, 0.0, 0.0, 1.0] })?;
        
        let entity2 = world.spawn();
        world.add_component(entity2, Position { x: 10.0, y: 10.0 })?;
        world.add_component(entity2, Velocity { dx: -0.5, dy: -0.5 })?;
        world.add_component(entity2, RenderComponent { width: 50.0, height: 50.0, color: [0.0, 1.0, 0.0, 1.0] })?;
        
        println!("Created entities: {} and {}", entity1, entity2);
    }
    
    // 运行 10 帧
    for i in 0..10 {
        println!("Frame {}", i);
        runtime.tick(std::time::Duration::from_millis(16))?;
        
        // 打印实体位置
        {
            let world = runtime.scheduler().world();
            if let Some(pos) = world.get_component::<Position>(0) {
                println!("Entity 1 position: ({}, {})", pos.x, pos.y);
            }
            if let Some(pos) = world.get_component::<Position>(1) {
                println!("Entity 2 position: ({}, {})", pos.x, pos.y);
            }
        }
        
        // 简单的延迟
        std::thread::sleep(std::time::Duration::from_millis(100));
    }
    
    println!("Example completed");
    Ok(())
}
