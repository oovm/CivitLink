#![warn(missing_docs)]

//! GG 引擎运行时核心模块
//!
//! 提供脚本驱动的运行时系统和游戏循环，
//! 支持动态渲染后端、音频后端、HMR 热更新和组件注册表。

pub mod app;
pub mod builder;
pub mod hmr;
pub mod hmr_state;
pub mod registry;
pub mod scheduler;
pub mod stage;
/// WASM 沙箱运行时模块
pub mod wasm;

pub use app::{App, RuntimePlugin};
pub use builder::RuntimeBuilder;
pub use hmr::{HmrEvent, HmrManager, HmrMigrationResult};
pub use hmr_state::{StateMigrator, StateSnapshot};
pub use registry::{ComponentAccessor, ComponentRegistry};
pub use scheduler::StageScheduler;
pub use stage::{Stage, SystemDescriptor, SystemFn, SystemSet, SystemSetId};
pub use wasm::{
    WasmError, WasmHostFunctions, WasmInstanceId, WasmModuleId, WasmRuntime, WasmSandboxConfig,
};

use gg_core::{GError, GErrorKind, GResult, plugin::Plugin};
use gg_ecs::{World, Entity};
use gg_asset::AssetServer;
use gg_render::{Renderer, RenderContext};
use gg_runtime_audio::{AudioEngine, AudioContext};
use gg_script::ScriptLoader;
use gg_ir::{IrModule, IrValue};
use gg_vm::{Vm, Host, VmResult};
use std::sync::Arc;
use std::time::{Duration, Instant};

/// 引擎宿主，实现 Host trait，将 VM 指令桥接到 ECS 世界
///
/// 通过 ComponentRegistry 实现动态组件操作，
/// 替代硬编码的组件类型匹配，支持脚本和 WASM 沙箱按名称访问组件。
pub struct EngineHost {
    /// ECS 世界
    world: World,
    /// 组件注册表
    registry: ComponentRegistry,
}

impl EngineHost {
    /// 创建新的引擎宿主
    pub fn new() -> Self {
        Self {
            world: World::new(),
            registry: ComponentRegistry::new(),
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

    /// 获取组件注册表的不可变引用
    pub fn registry(&self) -> &ComponentRegistry {
        &self.registry
    }

    /// 获取组件注册表的可变引用
    pub fn registry_mut(&mut self) -> &mut ComponentRegistry {
        &mut self.registry
    }
}

impl Host for EngineHost {
    fn spawn_entity(&mut self) -> u64 {
        let entity = self.world.spawn();
        entity.id()
    }

    fn despawn_entity(&mut self, entity_id: u64) {
        self.world.despawn(entity_id as Entity).ok();
    }

    fn add_component(&mut self, entity_id: u64, component_type: &str, _value: IrValue) {
        self.registry.add_default(&mut self.world, entity_id as Entity, component_type);
    }

    fn get_component_field(
        &mut self,
        entity_id: u64,
        component_type: &str,
        field: &str,
    ) -> Option<IrValue> {
        self.registry.get_field(&self.world, entity_id as Entity, component_type, field)
    }

    fn set_component_field(
        &mut self,
        entity_id: u64,
        component_type: &str,
        field: &str,
        value: IrValue,
    ) {
        self.registry.set_field(&mut self.world, entity_id as Entity, component_type, field, value);
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
                    if let (IrValue::Entity(entity_id), IrValue::String(component_type)) =
                        (&args[0], &args[1])
                    {
                        self.add_component(*entity_id, component_type, IrValue::Null);
                    }
                }
                None
            }
            "set_field" => {
                if args.len() >= 4 {
                    if let (
                        IrValue::Entity(entity_id),
                        IrValue::String(component_type),
                        IrValue::String(field),
                        value,
                    ) = (&args[0], &args[1], &args[2], args[3].clone())
                    {
                        self.set_component_field(*entity_id, component_type, field, value);
                    }
                }
                None
            }
            "get_field" => {
                if args.len() >= 3 {
                    if let (IrValue::Entity(entity_id), IrValue::String(component_type), IrValue::String(field)) =
                        (&args[0], &args[1], &args[2])
                    {
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
        self.module
            .as_ref()
            .map_or(false, |m| m.find_function(name).is_some())
    }

    /// 替换当前 IR 模块（用于 HMR 热更新）
    pub fn replace_module(&mut self, module: IrModule) {
        self.module = Some(module);
    }
}

/// 运行时系统
///
/// 管理游戏循环的核心运行时，集成脚本引擎、阶段调度器、
/// 渲染后端、音频后端和 HMR 热更新支持。
pub struct Runtime {
    /// 脚本引擎
    script_engine: ScriptEngine,
    /// 引擎宿主
    host: EngineHost,
    /// 阶段调度器
    stage_scheduler: StageScheduler,
    /// 资源服务器
    asset_server: AssetServer,
    /// 渲染后端实例
    renderer: Option<Box<dyn Renderer>>,
    /// 渲染上下文，收集每帧绘制命令
    render_context: RenderContext,
    /// 音频后端实例
    audio_engine: Option<Box<dyn AudioEngine>>,
    /// 音频上下文，收集每帧音频命令
    audio_context: AudioContext,
    /// HMR 管理器
    hmr_manager: Option<HmrManager>,
    /// 插件列表
    plugins: Vec<Arc<dyn Plugin>>,
    /// 运行状态
    running: bool,
    /// 上一帧时间
    #[allow(dead_code)]
    last_frame_time: Instant,
}

impl Runtime {
    /// 从构建器创建 Runtime 实例
    pub fn from_builder(builder: builder::RuntimeBuilder) -> GResult<Self> {
        let renderer = builder.renderer;
        let audio_engine = builder.audio_engine;
        let hmr_manager = if builder.hmr_enabled {
            Some(HmrManager::new())
        } else {
            None
        };

        let surface_width = renderer
            .as_ref()
            .map(|r| r.surface_info().width)
            .unwrap_or(800);
        let surface_height = renderer
            .as_ref()
            .map(|r| r.surface_info().height)
            .unwrap_or(600);

        Ok(Self {
            script_engine: ScriptEngine::new(),
            host: EngineHost::new(),
            stage_scheduler: StageScheduler::new(),
            asset_server: AssetServer::new(),
            render_context: RenderContext::new(surface_width, surface_height),
            renderer,
            audio_engine,
            audio_context: AudioContext::new(),
            hmr_manager,
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

    /// 获取阶段调度器的不可变引用
    pub fn stage_scheduler(&self) -> &StageScheduler {
        &self.stage_scheduler
    }

    /// 获取阶段调度器的可变引用
    pub fn stage_scheduler_mut(&mut self) -> &mut StageScheduler {
        &mut self.stage_scheduler
    }

    /// 获取资源服务器的可变引用
    pub fn asset_server(&mut self) -> &mut AssetServer {
        &mut self.asset_server
    }

    /// 获取渲染后端的可变引用
    pub fn renderer(&mut self) -> Option<&mut Box<dyn Renderer>> {
        self.renderer.as_mut()
    }

    /// 获取渲染上下文的可变引用
    pub fn render_context(&mut self) -> &mut RenderContext {
        &mut self.render_context
    }

    /// 获取音频后端的可变引用
    pub fn audio_engine(&mut self) -> Option<&mut Box<dyn AudioEngine>> {
        self.audio_engine.as_mut()
    }

    /// 获取音频上下文的可变引用
    pub fn audio_context(&mut self) -> &mut AudioContext {
        &mut self.audio_context
    }

    /// 获取 HMR 管理器的可变引用
    pub fn hmr_manager(&mut self) -> Option<&mut HmrManager> {
        self.hmr_manager.as_mut()
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
    ///
    /// 初始化渲染后端，执行脚本 init 函数，然后进入游戏循环。
    pub fn start(&mut self) -> GResult<()> {
        if let Some(ref mut renderer) = self.renderer {
            renderer.begin_frame()?;
        }

        if self.script_engine.has_function("init") {
            match self.script_engine.call_function("init", &mut self.host) {
                VmResult::Ok | VmResult::Return(_) => {}
                VmResult::Error(e) => {
                    return Err(GError {
                        kind: GErrorKind::Runtime,
                        message: format!("Script init error: {}", e),
                    });
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

        self.stage_scheduler.stop(self.host.world_mut())?;

        if self.script_engine.has_function("shutdown") {
            self.script_engine.call_function("shutdown", &mut self.host);
        }

        for plugin in &self.plugins {
            plugin.shutdown()?;
        }

        Ok(())
    }

    /// 执行一帧
    ///
    /// 按顺序执行：HMR 事件处理 → 阶段调度 → 脚本更新 → 音频处理 → 渲染。
    pub fn tick(&mut self, _delta: Duration) -> GResult<()> {
        if let Some(ref mut hmr) = self.hmr_manager {
            if let Some(new_module) = hmr.process_script_reload() {
                self.script_engine.replace_module(new_module.clone());
                hmr.confirm_script_reload(new_module);
            }
        }

        self.stage_scheduler.tick(self.host.world_mut())?;

        if self.script_engine.has_function("update") {
            match self.script_engine.call_function("update", &mut self.host) {
                VmResult::Ok | VmResult::Return(_) => {}
                VmResult::Error(e) => {
                    eprintln!("Script update error: {}", e);
                }
                _ => {}
            }
        }

        if let Some(ref mut audio_engine) = self.audio_engine {
            audio_engine.update(&self.audio_context)?;
            self.audio_context.clear();
        }

        if let Some(ref mut renderer) = self.renderer {
            renderer.begin_frame()?;
            renderer.draw(&self.render_context)?;
            renderer.end_frame()?;
            renderer.present()?;
            self.render_context.clear();
        }

        Ok(())
    }
}
