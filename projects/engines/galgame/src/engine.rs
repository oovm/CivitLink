//! Galgame 引擎核心模块
//! 提供引擎初始化、主循环和插件管理功能

use crate::config::GalgameConfig;
use gg_asset::AssetManager;
use gg_core::{GResult, plugin::Plugin};
use gg_ecs::World;
use gg_platform_desktop::DesktopFileSystem;
use gg_plugin_dialogue::plugin::DialoguePlugin;
use gg_plugin_portrait::plugin::PortraitPlugin;
use gg_plugin_save::plugin::SavePlugin;
use gg_plugin_scene_transition::plugin::SceneTransitionPlugin;
use gg_render::{RenderContext, Renderer, SurfaceInfo};
use gg_render_wgpu::WgpuRenderer;
use gg_ui::{LayoutEngine, UiRenderer, UiTree};
use winit::{event::Event, event_loop::EventLoop};

/// Galgame 引擎
///
/// 负责管理游戏的生命周期，包括：
/// - 初始化渲染器和窗口
/// - 初始化所有插件
/// - 加载游戏资源
/// - 执行主循环（事件处理 → 逻辑更新 → 渲染 → 呈现）
pub struct GalgameEngine {
    /// 游戏配置
    pub config: GalgameConfig,
    /// ECS 世界
    pub world: World,
    /// 资源管理器
    pub asset_manager: AssetManager,
    /// 是否编辑器模式
    pub is_editor_mode: bool,
    /// WGPU 渲染器
    renderer: Option<WgpuRenderer>,
    /// UI 节点树
    ui_tree: UiTree,
}

impl GalgameEngine {
    /// 创建新的 Galgame 引擎实例
    ///
    /// # 参数
    ///
    /// - `config` - 游戏配置
    /// - `is_editor_mode` - 是否启用编辑器模式
    pub fn new(config: GalgameConfig, is_editor_mode: bool) -> Self {
        Self {
            config,
            world: GgWorld::new(),
            asset_manager: AssetManager::new(Box::new(DesktopFileSystem::new())),
            is_editor_mode,
            renderer: None,
            ui_tree: UiTree::new(),
        }
    }

    /// 初始化引擎
    ///
    /// 创建渲染器、窗口，注册所有插件并加载游戏资源。
    pub fn initialize(&mut self) -> GResult<()> {
        let screen_width = self.config.display.width as f32;
        let screen_height = self.config.display.height as f32;

        let plugins: Vec<Box<dyn Plugin>> = vec![
            Box::new(DialoguePlugin),
            Box::new(PortraitPlugin::new(screen_width, screen_height)),
            Box::new(SceneTransitionPlugin),
            Box::new(SavePlugin),
        ];

        for plugin in plugins {
            plugin.initialize()?;
        }

        Ok(())
    }

    /// 使用事件循环初始化渲染器
    ///
    /// 必须在 `initialize` 之后、`run` 之前调用。
    ///
    /// # 参数
    ///
    /// - `event_loop` - winit 事件循环
    pub fn initialize_renderer(&mut self, event_loop: &EventLoop<()>) -> GResult<()> {
        let surface_info =
            SurfaceInfo::new(self.config.display.width, self.config.display.height, self.config.game.name.clone())
                .with_fullscreen(self.config.display.fullscreen);

        let renderer = WgpuRenderer::new(event_loop, surface_info)?;
        self.renderer = Some(renderer);
        Ok(())
    }

    /// 执行一帧
    ///
    /// 执行所有已注册的 ECS 系统。
    pub fn tick(&mut self) -> GResult<()> {
        self.world.run_systems()
    }

    /// 运行主循环
    ///
    /// 使用 winit 事件循环驱动主循环：
    /// 1. 处理窗口事件
    /// 2. 执行游戏逻辑（tick）
    /// 3. 渲染一帧
    /// 4. 呈现到屏幕
    pub fn run(mut self) -> GResult<()> {
        let event_loop = EventLoop::new().map_err(|e| gg_core::GError {
            kind: gg_core::GErrorKind::Platform,
            message: format!("Failed to create event loop: {}", e),
        })?;

        self.initialize_renderer(&event_loop)?;

        event_loop
            .run(move |event, elwt| {
                if let Some(renderer) = &mut self.renderer {
                    match event {
                        Event::WindowEvent { event, .. } => {
                            renderer.handle_window_event(&event);
                            if renderer.should_close() {
                                elwt.exit();
                            }
                        }
                        Event::AboutToWait => {
                            if let Err(_) = self.tick() {
                                elwt.exit();
                            }

                            if let Err(_) = self.render_frame() {
                                elwt.exit();
                            }
                        }
                        _ => {}
                    }
                }
            })
            .map_err(|e| gg_core::GError { kind: gg_core::GErrorKind::Runtime, message: format!("Event loop error: {}", e) })
    }

    /// 渲染一帧
    ///
    /// 执行渲染流程：开始帧 → 绘制 → 结束帧 → 呈现
    fn render_frame(&mut self) -> GResult<()> {
        let renderer = self.renderer.as_mut().ok_or_else(|| gg_core::GError {
            kind: gg_core::GErrorKind::Runtime,
            message: "Renderer not initialized".to_string(),
        })?;

        renderer.begin_frame()?;

        let mut context = RenderContext::new(renderer.surface_info().width, renderer.surface_info().height);

        LayoutEngine::compute(&mut self.ui_tree, renderer.surface_info().width as f32, renderer.surface_info().height as f32);

        UiRenderer::render(&self.ui_tree, &mut context);

        renderer.draw(&context)?;
        renderer.end_frame()?;
        renderer.present()?;

        Ok(())
    }
}
