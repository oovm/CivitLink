#![warn(missing_docs)]

//! GG 引擎核心模块

use gg_asset::AssetServer;
use gg_core::{GResult, plugin::PluginManager};
use gg_ecs::World;
use gg_render::RenderContext;
use gg_render_wgpu::WgpuRenderContext;
use winit::window::Window;

use crate::config::EngineConfig;

/// 引擎核心结构
pub struct Engine {
    /// 引擎配置
    config: EngineConfig,
    /// ECS 世界
    world: World,
    /// 资产服务器
    asset_server: AssetServer,
    /// 渲染上下文
    render_context: Box<dyn RenderContext>,
    /// 插件管理器
    plugin_manager: PluginManager,
    /// 窗口
    window: Option<Window>,
}

impl Engine {
    /// 创建新的引擎实例
    pub fn new(config: EngineConfig) -> GResult<Self> {
        // 初始化世界
        let world = World::new();

        // 初始化资产服务器
        let asset_server = AssetServer::new(&config.asset.asset_dir)?;

        // 初始化渲染上下文
        let render_context = Box::new(WgpuRenderContext::new()?);

        // 初始化插件管理器
        let plugin_manager = PluginManager::new();

        Ok(Self { config, world, asset_server, render_context, plugin_manager, window: None })
    }

    /// 启动引擎
    pub fn run(&mut self) -> GResult<()> {
        // 初始化窗口
        self.init_window()?;

        // 加载插件
        self.load_plugins()?;

        // 初始化系统
        self.init_systems()?;

        // 进入主循环
        self.main_loop()?;

        Ok(())
    }

    /// 初始化窗口
    fn init_window(&mut self) -> GResult<()> {
        // 这里应该实现窗口初始化逻辑
        Ok(())
    }

    /// 加载插件
    fn load_plugins(&mut self) -> GResult<()> {
        // 这里应该实现插件加载逻辑
        Ok(())
    }

    /// 初始化系统
    fn init_systems(&mut self) -> GResult<()> {
        // 这里应该实现系统初始化逻辑
        Ok(())
    }

    /// 主循环
    fn main_loop(&mut self) -> GResult<()> {
        // 这里应该实现主循环逻辑
        Ok(())
    }
}

impl Default for Engine {
    fn default() -> Self {
        Self::new(EngineConfig::default()).expect("Failed to create default engine")
    }
}
