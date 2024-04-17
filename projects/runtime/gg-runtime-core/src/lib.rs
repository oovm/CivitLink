#![warn(missing_docs)]

//! GG 引擎运行时核心模块
//!
//! 提供脚本驱动的运行时系统和游戏循环，
//! 支持动态渲染后端、音频后端、HMR 热更新和组件注册表。

pub mod app;
pub mod builder;
pub mod hmr;
pub mod hmr_state;
pub mod plugin;
pub mod registry;
pub mod scheduler;
pub mod stage;

pub use app::{App, RuntimePlugin};
pub use builder::RuntimeBuilder;
pub use hmr::{HmrEvent, HmrManager, HmrMigrationResult};
pub use hmr_state::{StateMigrator, StateSnapshot};
pub use plugin::{PluginConfig, PluginId, PluginManager};
pub use registry::{ComponentAccessor, ComponentRegistry};
pub use scheduler::StageScheduler;
pub use stage::{Stage, SystemDescriptor, SystemFn, SystemSet, SystemSetId};

use gg_asset::AssetServer;
use gg_bytecode::{BytecodeModule, BytecodeValue, Host};
use gg_core::{GError, GErrorKind, GResult, plugin::Plugin};
use gg_ecs::{Entity, World};
use gg_render::{RenderContext, Renderer};
use gg_runtime_audio::{AudioContext, AudioEngine};
use gg_script::ScriptLoader;
use gg_vm::{Vm, VmResult};
use std::{
    sync::Arc,
    time::{Duration, Instant},
};

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
        Self { world: World::new(), registry: ComponentRegistry::new() }
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

    fn add_component(&mut self, entity_id: u64, component_type: &str, _value: BytecodeValue) {
        self.registry.add_default(&mut self.world, entity_id as Entity, component_type);
    }

    fn get_component_field(&mut self, entity_id: u64, component_type: &str, field: &str) -> Option<BytecodeValue> {
        self.registry.get_field(&self.world, entity_id as Entity, component_type, field)
    }

    fn set_component_field(&mut self, entity_id: u64, component_type: &str, field: &str, value: BytecodeValue) {
        self.registry.set_field(&mut self.world, entity_id as Entity, component_type, field, value);
    }

    fn call_host_function(&mut self, name: &str, args: Vec<BytecodeValue>) -> Option<BytecodeValue> {
        match name {
            "print" => {
                for arg in &args {
                    match arg {
                        BytecodeValue::String(s) => println!("{}", s),
                        BytecodeValue::Int(i) => println!("{}", i),
                        BytecodeValue::Float(f) => println!("{}", f),
                        BytecodeValue::Bool(b) => println!("{}", b),
                        BytecodeValue::Entity(e) => println!("Entity({})", e),
                        BytecodeValue::Null => println!("null"),
                    }
                }
                None
            }
            "spawn_entity" => {
                let entity_id = self.spawn_entity();
                Some(BytecodeValue::Entity(entity_id))
            }
            "add_component" => {
                if args.len() >= 2 {
                    if let (BytecodeValue::Entity(entity_id), BytecodeValue::String(component_type)) = (&args[0], &args[1]) {
                        self.add_component(*entity_id, component_type, BytecodeValue::Null);
                    }
                }
                None
            }
            "set_field" => {
                if args.len() >= 4 {
                    if let (
                        BytecodeValue::Entity(entity_id),
                        BytecodeValue::String(component_type),
                        BytecodeValue::String(field),
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
                    if let (
                        BytecodeValue::Entity(entity_id),
                        BytecodeValue::String(component_type),
                        BytecodeValue::String(field),
                    ) = (&args[0], &args[1], &args[2])
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
    /// 已编译的字节码模块
    module: Option<BytecodeModule>,
    /// 虚拟机
    vm: Vm,
}

impl ScriptEngine {
    /// 创建新的脚本引擎
    pub fn new() -> Self {
        Self { loader: ScriptLoader::new(), module: None, vm: Vm::new() }
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
        }
        else {
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

    /// 替换当前字节码模块（用于 HMR 热更新）
    pub fn replace_module(&mut self, module: BytecodeModule) {
        self.module = Some(module);
    }

    /// 获取当前字节码模块的引用
    pub fn module(&self) -> Option<&BytecodeModule> {
        self.module.as_ref()
    }
}

/// 帧间隔计时器
///
/// 精确追踪帧间隔时间，为游戏循环提供准确的 delta 值。
pub struct DeltaTimer {
    /// 上一帧时间戳
    last_frame: Instant,
    /// 累计运行时间
    elapsed: Duration,
}

impl DeltaTimer {
    /// 创建新的帧间隔计时器
    pub fn new() -> Self {
        Self { last_frame: Instant::now(), elapsed: Duration::ZERO }
    }

    /// 记录一帧，返回自上次 tick 以来的时间间隔
    pub fn tick(&mut self) -> Duration {
        let now = Instant::now();
        let delta = now.duration_since(self.last_frame);
        self.last_frame = now;
        self.elapsed += delta;
        delta
    }

    /// 返回累计运行时间
    pub fn elapsed(&self) -> Duration {
        self.elapsed
    }

    /// 将 Duration 转换为 f32 秒数
    pub fn delta_seconds(delta: Duration) -> f32 {
        delta.as_secs_f32()
    }
}

/// 运行时系统
///
/// 管理游戏循环的核心运行时，集成脚本引擎、阶段调度器、
/// 渲染后端、音频后端、平台服务和 HMR 热更新支持。
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
    /// 平台服务
    platform_services: gg_core::platform::PlatformServices,
    /// HMR 管理器
    hmr_manager: Option<HmrManager>,
    /// 插件列表
    plugins: Vec<Arc<dyn Plugin>>,
    /// 脚本插件管理器
    plugin_manager: plugin::PluginManager,
    /// 运行状态
    running: bool,
    /// 帧间隔计时器
    delta_timer: DeltaTimer,
}

impl Runtime {
    /// 从构建器创建 Runtime 实例
    pub fn from_builder(builder: builder::RuntimeBuilder) -> GResult<Self> {
        let renderer = builder.renderer;
        let audio_engine = builder.audio_engine;
        let hmr_manager = if builder.hmr_enabled { Some(HmrManager::new()) } else { None };

        let surface_width = renderer.as_ref().map(|r| r.surface_info().width).unwrap_or(800);
        let surface_height = renderer.as_ref().map(|r| r.surface_info().height).unwrap_or(600);

        let platform_services = builder.platform_services.unwrap_or_else(|| {
            #[cfg(feature = "desktop")]
            { gg_platform_desktop::DesktopPlatformServices::create() }
            #[cfg(feature = "web")]
            { gg_platform_web::WebPlatformServices::create("/assets") }
            #[cfg(not(any(feature = "desktop", feature = "web")))]
            { panic!("No platform services provided. Use RuntimeBuilder::with_platform(), .desktop(), or .web() to specify platform services.") }
        });

        let asset_server = AssetServer::new();

        Ok(Self {
            script_engine: ScriptEngine::new(),
            host: EngineHost::new(),
            stage_scheduler: StageScheduler::new(),
            asset_server,
            render_context: RenderContext::new(surface_width, surface_height),
            renderer,
            audio_engine,
            audio_context: AudioContext::new(),
            platform_services,
            hmr_manager,
            plugins: Vec::new(),
            plugin_manager: plugin::PluginManager::new(),
            running: false,
            delta_timer: DeltaTimer::new(),
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

    /// 获取平台服务的可变引用
    pub fn platform_services(&mut self) -> &mut gg_core::platform::PlatformServices {
        &mut self.platform_services
    }

    /// 获取 HMR 管理器的可变引用
    pub fn hmr_manager(&mut self) -> Option<&mut HmrManager> {
        self.hmr_manager.as_mut()
    }

    /// 获取插件管理器的可变引用
    pub fn plugin_manager_mut(&mut self) -> &mut plugin::PluginManager {
        &mut self.plugin_manager
    }

    /// 获取插件管理器的不可变引用
    pub fn plugin_manager(&self) -> &plugin::PluginManager {
        &self.plugin_manager
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
                    return Err(GError { kind: GErrorKind::Runtime, message: format!("Script init error: {}", e) });
                }
            }
        }

        // 初始化所有脚本插件
        for plugin in self.plugin_manager.plugins() {
            if let Some(entry_fn) = plugin.bytecode.find_function(&plugin.config.entry_function) {
                let mut vm = Vm::new();
                match vm.execute(&plugin.bytecode, &plugin.config.entry_function, &mut self.host) {
                    VmResult::Ok | VmResult::Return(_) => {
                        eprintln!("Plugin initialized: {}", plugin.config.name);
                    }
                    VmResult::Error(e) => {
                        eprintln!("Plugin initialization error ({}): {}", plugin.config.name, e);
                    }
                }
            }
        }

        self.running = true;
        self.run()
    }

    /// 停止运行时
    pub fn stop(&mut self) {
        // 执行所有插件的 shutdown 函数
        for plugin in self.plugin_manager.plugins() {
            if plugin.bytecode.find_function("shutdown").is_some() {
                let mut vm = Vm::new();
                match vm.execute(&plugin.bytecode, "shutdown", &mut self.host) {
                    VmResult::Ok | VmResult::Return(_) => {
                        eprintln!("Plugin shutdown: {}", plugin.config.name);
                    }
                    VmResult::Error(e) => {
                        eprintln!("Plugin shutdown error ({}): {}", plugin.config.name, e);
                    }
                }
            }
        }

        self.running = false;
    }

    /// 目标帧时间（约 60 FPS），在无垂直同步时使用
    const TARGET_FRAME_TIME: Duration = Duration::from_nanos(16_666_667);

    /// 运行游戏循环
    pub fn run(&mut self) -> GResult<()> {
        while self.running {
            let frame_start = Instant::now();
            let delta = self.delta_timer.tick();

            self.tick(delta)?;

            if self.renderer.is_none() {
                let frame_elapsed = frame_start.elapsed();
                if frame_elapsed < TARGET_FRAME_TIME {
                    std::thread::sleep(TARGET_FRAME_TIME - frame_elapsed);
                }
            }
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
    /// 按顺序执行：平台服务更新 → HMR 事件处理 → 阶段调度 → 脚本更新 → 音频处理 → 渲染。
    pub fn tick(&mut self, delta: Duration) -> GResult<()> {
        self.platform_services.time.update();

        let _ = delta;

        let _input_events = self.platform_services.input.poll_events();

        if let Some(ref mut hmr) = self.hmr_manager {
            if let Some(new_module) = hmr.process_script_reload() {
                let old_module = self.script_engine.module().cloned();
                self.script_engine.replace_module(new_module.clone());

                if self.script_engine.has_function("on_hot_reload_in") {
                    match self.script_engine.call_function("on_hot_reload_in", &mut self.host) {
                        VmResult::Ok | VmResult::Return(_) => {
                            hmr.confirm_script_reload(new_module);
                        }
                        VmResult::Error(e) => {
                            eprintln!("HMR state migration failed: {}, rolling back", e);
                            if let Some(old) = old_module {
                                self.script_engine.replace_module(old);
                            }
                            return Ok(());
                        }
                    }
                }
                else {
                    hmr.confirm_script_reload(new_module);
                }
            }

            let changed_assets = hmr.process_asset_reload();
            for asset_path in &changed_assets {
                let lower = asset_path.to_lowercase();
                if lower.ends_with(".png")
                    || lower.ends_with(".jpg")
                    || lower.ends_with(".jpeg")
                    || lower.ends_with(".bmp")
                    || lower.ends_with(".webp")
                {
                    if let Some(ref mut renderer) = self.renderer {
                        match renderer.reload_texture(asset_path) {
                            Ok(()) => eprintln!("HMR: Texture reloaded: {}", asset_path),
                            Err(e) => eprintln!("HMR: Failed to reload texture {}: {:?}", asset_path, e),
                        }
                    }
                }
                else if lower.ends_with(".wav")
                    || lower.ends_with(".ogg")
                    || lower.ends_with(".mp3")
                    || lower.ends_with(".flac")
                {
                    if let Some(ref mut audio_engine) = self.audio_engine {
                        match audio_engine.reload_sound(asset_path) {
                            Ok(()) => eprintln!("HMR: Audio reloaded: {}", asset_path),
                            Err(e) => eprintln!("HMR: Failed to reload audio {}: {:?}", asset_path, e),
                        }
                    }
                }
                else {
                    eprintln!("HMR: Unknown asset type, skipping: {}", asset_path);
                }
            }
        }

        self.stage_scheduler.tick(self.host.world_mut())?;

        if self.script_engine.has_function("update") {
            match self.script_engine.call_function("update", &mut self.host) {
                VmResult::Ok | VmResult::Return(_) => {}
                VmResult::Error(e) => {
                    eprintln!("Script update error: {}", e);
                }
            }
        }

        // 执行所有插件的 update 函数
        for plugin in self.plugin_manager.plugins() {
            if plugin.bytecode.find_function("update").is_some() {
                let mut vm = Vm::new();
                match vm.execute(&plugin.bytecode, "update", &mut self.host) {
                    VmResult::Ok | VmResult::Return(_) => {}
                    VmResult::Error(e) => {
                        eprintln!("Plugin update error ({}): {}", plugin.config.name, e);
                    }
                }
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
