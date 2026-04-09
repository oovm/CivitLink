#![warn(missing_docs)]

//! GG 引擎运行时核心模块
//! 提供脚本驱动的运行时系统和游戏循环

pub mod app;
pub mod hmr;
pub mod hmr_state;
pub mod scheduler;
pub mod stage;
pub mod wasm;

pub use app::{App, RuntimePlugin};
pub use hmr::{HmrEvent, HmrManager, HmrMigrationResult};
pub use hmr_state::{StateMigrator, StateSnapshot};
pub use scheduler::StageScheduler;
pub use stage::{Stage, SystemDescriptor, SystemFn, SystemSet, SystemSetId};
pub use wasm::{WasmError, WasmHostFunctions, WasmInstanceId, WasmModuleId, WasmRuntime, WasmSandboxConfig};

use gg_core::{GResult, GError, GErrorKind, plugin::Plugin};
use gg_ecs::{Scheduler, World, Entity, Component};
use gg_asset::AssetManager;
use gg_render::{RenderSystem, RenderComponent};
use gg_script::ScriptLoader;
use gg_ir::{IrModule, IrValue};
use gg_vm::{Vm, Host, VmResult};
use std::sync::Arc;
use std::time::{Duration, Instant};

/// 引擎宿主，实现 Host trait，将 VM 指令桥接到 ECS 世界
pub struct EngineHost {
    /// ECS 世界
    world: World,
}

impl EngineHost {
    /// 创建新的引擎宿主
    pub fn new() -> Self {
        Self {
            world: World::new(),
        }
    }

    /// 获取世界的不可变引用
    pub fn world(&self) -> &World {
        &self.world
    }

    /// 获取世界的可变引用
    pub fn world_mut(&mut self) -> &mut World {
        &mut self.world
    }
}

impl Host for EngineHost {
    fn spawn_entity(&mut self) -> u64 {
        let entity = self.world.spawn();
        entity
    }

    fn despawn_entity(&mut self, entity_id: u64) {
        self.world.despawn(entity_id as Entity).ok();
    }

    fn add_component(&mut self, entity_id: u64, component_type: &str, _value: IrValue) {
        match component_type {
            "Position" => {
                self.world.add_component(entity_id as Entity, Position { x: 0.0, y: 0.0 }).ok();
            }
            "Velocity" => {
                self.world.add_component(entity_id as Entity, Velocity { dx: 0.0, dy: 0.0 }).ok();
            }
            "RenderComponent" => {
                self.world.add_component(entity_id as Entity, RenderComponent { width: 50.0, height: 50.0, color: [1.0, 1.0, 1.0, 1.0] }).ok();
            }
            _ => {
                eprintln!("Warning: Unknown component type: {}", component_type);
            }
        }
    }

    fn get_component_field(&mut self, entity_id: u64, component_type: &str, field: &str) -> Option<IrValue> {
        match component_type {
            "Position" => {
                if let Some(pos) = self.world.get_component::<Position>(entity_id as Entity) {
                    match field {
                        "x" => Some(IrValue::Float(pos.x as f64)),
                        "y" => Some(IrValue::Float(pos.y as f64)),
                        _ => None,
                    }
                } else {
                    None
                }
            }
            "Velocity" => {
                if let Some(vel) = self.world.get_component::<Velocity>(entity_id as Entity) {
                    match field {
                        "dx" => Some(IrValue::Float(vel.dx as f64)),
                        "dy" => Some(IrValue::Float(vel.dy as f64)),
                        _ => None,
                    }
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    fn set_component_field(&mut self, entity_id: u64, component_type: &str, field: &str, value: IrValue) {
        match component_type {
            "Position" => {
                if let Some(pos) = self.world.get_component_mut::<Position>(entity_id as Entity) {
                    if let IrValue::Float(v) = value {
                        match field {
                            "x" => pos.x = v as f32,
                            "y" => pos.y = v as f32,
                            _ => {}
                        }
                    }
                }
            }
            "Velocity" => {
                if let Some(vel) = self.world.get_component_mut::<Velocity>(entity_id as Entity) {
                    if let IrValue::Float(v) = value {
                        match field {
                            "dx" => vel.dx = v as f32,
                            "dy" => vel.dy = v as f32,
                            _ => {}
                        }
                    }
                }
            }
            _ => {}
        }
    }

    fn call_host_function(&mut self, name: &str, args: Vec<IrValue>) -> Option<IrValue> {
        match name {
            "print" => {
                for arg in &args {
                    match arg {
                        IrValue::String(s) => println!("{}", s),
                        IrValue::Int(i) => println!("{}", i),
                        IrValue::Float(f) => println!("{}", f),
                        IrValue::Bool(b) => println!("{}", b),
                        IrValue::Entity(e) => println!("Entity({})", e),
                        IrValue::Null => println!("null"),
                    }
                }
                None
            }
            "spawn_entity" => {
                let entity_id = self.spawn_entity();
                Some(IrValue::Entity(entity_id))
            }
            "add_component" => {
                if args.len() >= 2 {
                    if let (IrValue::Entity(entity_id), IrValue::String(component_type)) = (&args[0], &args[1]) {
                        self.add_component(*entity_id, component_type, IrValue::Null);
                    }
                }
                None
            }
            "set_field" => {
                if args.len() >= 4 {
                    if let (IrValue::Entity(entity_id), IrValue::String(component_type), IrValue::String(field), value) = (&args[0], &args[1], &args[2], args[3].clone()) {
                        self.set_component_field(*entity_id, component_type, field, value);
                    }
                }
                None
            }
            "get_field" => {
                if args.len() >= 3 {
                    if let (IrValue::Entity(entity_id), IrValue::String(component_type), IrValue::String(field)) = (&args[0], &args[1], &args[2]) {
                        return self.get_component_field(*entity_id, component_type, field);
                    }
                }
                None
            }
            _ => {
                eprintln!("Warning: Unknown host function: {}", name);
                None
            }
        }
    }
}

/// 脚本引擎，管理脚本编译和执行
pub struct ScriptEngine {
    /// 脚本加载器
    loader: ScriptLoader,
    /// 已编译的 IR 模块
    module: Option<IrModule>,
    /// 虚拟机
    vm: Vm,
}

impl ScriptEngine {
    /// 创建新的脚本引擎
    pub fn new() -> Self {
        Self {
            loader: ScriptLoader::new(),
            module: None,
            vm: Vm::new(),
        }
    }

    /// 加载脚本文件
    pub fn load_script(&mut self, path: &std::path::Path) -> GResult<()> {
        let module = self.loader.load_file(path)?;
        self.module = Some(module);
        Ok(())
    }

    /// 从字符串加载脚本
    pub fn load_script_string(&mut self, source: &str, module_name: &str) -> GResult<()> {
        let module = self.loader.load_string(source, module_name)?;
        self.module = Some(module);
        Ok(())
    }

    /// 执行脚本中的指定函数
    pub fn call_function(&mut self, function_name: &str, host: &mut EngineHost) -> VmResult {
        if let Some(ref module) = self.module {
            self.vm.execute(module, function_name, host)
        } else {
            VmResult::Error("No script loaded".to_string())
        }
    }

    /// 检查是否已加载脚本
    pub fn has_script(&self) -> bool {
        self.module.is_some()
    }

    /// 检查脚本中是否存在指定函数
    pub fn has_function(&self, name: &str) -> bool {
        self.module.as_ref().map_or(false, |m| m.find_function(name).is_some())
    }
}

/// 运行时系统
pub struct Runtime {
    /// 脚本引擎
    script_engine: ScriptEngine,
    /// 引擎宿主
    host: EngineHost,
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
    #[allow(dead_code)]
    last_frame_time: Instant,
}

impl Runtime {
    /// 创建新的运行时
    pub fn new() -> GResult<Self> {
        Ok(Self {
            script_engine: ScriptEngine::new(),
            host: EngineHost::new(),
            scheduler: Scheduler::new(),
            asset_manager: AssetManager::new(),
            render_system: RenderSystem::new()?,
            plugins: Vec::new(),
            running: false,
            last_frame_time: Instant::now(),
        })
    }

    /// 获取脚本引擎的可变引用
    pub fn script_engine(&mut self) -> &mut ScriptEngine {
        &mut self.script_engine
    }

    /// 获取引擎宿主的可变引用
    pub fn host(&mut self) -> &mut EngineHost {
        &mut self.host
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

    /// 加载 Valkyrie 脚本文件
    pub fn load_script(&mut self, path: &std::path::Path) -> GResult<()> {
        self.script_engine.load_script(path)
    }

    /// 从字符串加载 Valkyrie 脚本
    pub fn load_script_string(&mut self, source: &str, module_name: &str) -> GResult<()> {
        self.script_engine.load_script_string(source, module_name)
    }

    /// 执行脚本中的指定函数
    pub fn call_script_function(&mut self, function_name: &str) -> VmResult {
        self.script_engine.call_function(function_name, &mut self.host)
    }

    /// 启动运行时
    pub fn start(&mut self) -> GResult<()> {
        self.render_system.init()?;

        if self.script_engine.has_function("init") {
            match self.script_engine.call_function("init", &mut self.host) {
                VmResult::Ok | VmResult::Return(_) => {}
                VmResult::Error(e) => {
                    return Err(GError { kind: GErrorKind::Runtime, message: format!("Script init error: {}", e) });
                }
                _ => {}
            }
        }

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
            let now = Instant::now();
            let delta = now.duration_since(last_frame);
            last_frame = now;

            self.tick(delta)?;

            std::thread::sleep(Duration::from_millis(16));
        }

        if self.script_engine.has_function("shutdown") {
            self.script_engine.call_function("shutdown", &mut self.host);
        }

        for plugin in &self.plugins {
            plugin.shutdown()?;
        }

        Ok(())
    }

    /// 执行一帧
    pub fn tick(&mut self, _delta: Duration) -> GResult<()> {
        self.scheduler.tick()?;

        if self.script_engine.has_function("update") {
            match self.script_engine.call_function("update", &mut self.host) {
                VmResult::Ok | VmResult::Return(_) => {}
                VmResult::Error(e) => {
                    eprintln!("Script update error: {}", e);
                }
                _ => {}
            }
        }

        self.render_system.render()?;

        Ok(())
    }
}

/// 位置组件
#[derive(Debug)]
pub struct Position {
    /// X 坐标
    pub x: f32,
    /// Y 坐标
    pub y: f32,
}

impl Component for Position {}

/// 速度组件
#[derive(Debug)]
pub struct Velocity {
    /// X 方向速度
    pub dx: f32,
    /// Y 方向速度
    pub dy: f32,
}

impl Component for Velocity {}
