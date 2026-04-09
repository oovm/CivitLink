//! GG 引擎运行时核心模块
//! 提供基本的运行时系统和游戏循环

use gg_core::{GResult, plugin::Plugin};
use gg_ecs::{Scheduler, World, System, Entity};
use gg_asset::AssetManager;
use gg_render::{RenderSystem, RenderComponent};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

/// 运行时系统
pub struct Runtime {
    /// 调度器
    scheduler: Scheduler,
    /// 资源管理器
    asset_manager: AssetManager,
    /// 渲染系统
    render_system: RenderSystem,
    /// 插件列表
    plugins: Vec<Arc<dyn Plugin>>,
    /// 运行状态
    running: bool,
    /// 上一帧时间
    last_frame_time: Instant,
}

impl Runtime {
    /// 创建新的运行时
    pub fn new() -> GResult<Self> {
        Ok(Self {
            scheduler: Scheduler::new(),
            asset_manager: AssetManager::new(),
            render_system: RenderSystem::new()?,
            plugins: Vec::new(),
            running: false,
            last_frame_time: Instant::now(),
        })
    }
    
    /// 获取调度器
    pub fn scheduler(&mut self) -> &mut Scheduler {
        &mut self.scheduler
    }
    
    /// 获取资源管理器
    pub fn asset_manager(&mut self) -> &mut AssetManager {
        &mut self.asset_manager
    }
    
    /// 获取渲染系统
    pub fn render_system(&mut self) -> &mut RenderSystem {
        &mut self.render_system
    }
    
    /// 注册插件
    pub fn register_plugin(&mut self, plugin: Arc<dyn Plugin>) -> GResult<()> {
        plugin.initialize()?;
        self.plugins.push(plugin);
        Ok(())
    }
    
    /// 启动运行时
    pub fn start(&mut self) -> GResult<()> {
        // 初始化渲染系统
        self.render_system.init()?;
        
        self.running = true;
        self.run()
    }
    
    /// 停止运行时
    pub fn stop(&mut self) {
        self.running = false;
    }
    
    /// 运行游戏循环
    pub fn run(&mut self) -> GResult<()> {
        let mut last_frame = Instant::now();
        
        while self.running {
            // 计算 delta 时间
            let now = Instant::now();
            let delta = now.duration_since(last_frame);
            last_frame = now;
            
            // 执行一帧
            self.tick(delta)?;
            
            // 简单的帧率控制
            std::thread::sleep(Duration::from_millis(16)); // 约 60 FPS
        }
        
        // 关闭插件
        for plugin in &self.plugins {
            plugin.shutdown()?;
        }
        
        Ok(())
    }
    
    /// 执行一帧
    pub fn tick(&mut self, delta: Duration) -> GResult<()> {
        // 执行 ECS 系统
        self.scheduler.tick()?;
        
        // 渲染一帧
        self.render_system.render()?;
        
        Ok(())
    }
}

/// 示例系统：打印实体数量
pub struct EntityCountSystem {
    last_count: usize,
}

impl EntityCountSystem {
    /// 创建新的实体计数系统
    pub fn new() -> Self {
        Self {
            last_count: 0,
        }
    }
}

impl System for EntityCountSystem {
    fn name(&self) -> &str {
        "EntityCountSystem"
    }
    
    fn execute(&mut self, world: &mut World) -> GResult<()> {
        let count = world.entities().len();
        if count != self.last_count {
            println!("Entity count: {}", count);
            self.last_count = count;
        }
        Ok(())
    }
}

/// 示例组件：位置
#[derive(Debug)]
pub struct Position {
    pub x: f32,
    pub y: f32,
}

impl gg_ecs::Component for Position {}

/// 示例组件：速度
#[derive(Debug)]
pub struct Velocity {
    pub dx: f32,
    pub dy: f32,
}

impl gg_ecs::Component for Velocity {}

/// 示例系统：移动系统
pub struct MovementSystem {
    delta_time: f32,
}

impl MovementSystem {
    /// 创建新的移动系统
    pub fn new() -> Self {
        Self {
            delta_time: 0.016, // 约 60 FPS
        }
    }
}

impl System for MovementSystem {
    fn name(&self) -> &str {
        "MovementSystem"
    }
    
    fn execute(&mut self, world: &mut World) -> GResult<()> {
        // 先获取所有实体的副本，避免不可变借用和可变借用冲突
        let entities: Vec<Entity> = world.entities().iter().copied().collect();
        
        // 简单的移动逻辑
        for entity in entities {
            // 为每个实体单独处理，避免借用冲突
            // 先检查实体是否同时拥有 Position 和 Velocity 组件
            if world.get_component::<Position>(entity).is_some() && world.get_component::<Velocity>(entity).is_some() {
                // 先获取 velocity 的值
                let velocity = world.get_component::<Velocity>(entity).unwrap();
                let dx = velocity.dx;
                let dy = velocity.dy;
                
                // 再获取并修改 position
                if let Some(position) = world.get_component_mut::<Position>(entity) {
                    position.x += dx * self.delta_time;
                    position.y += dy * self.delta_time;
                }
            }
        }
        Ok(())
    }
}
