//! Platformer 引擎核心模块
//! 提供引擎初始化、主循环和插件管理功能

use crate::components::*;
use crate::config::PlatformerConfig;
use crate::systems::*;
use gg_asset::AssetManager;
use gg_core::{GResult, plugin::Plugin};
use gg_ecs::World;
use gg_platform_desktop::DesktopFileSystem;
use gg_render::{RenderContext, Renderer, SurfaceInfo};
use gg_render_wgpu::WgpuRenderer;
use gg_ui::{LayoutEngine, UiRenderer, UiTree};
use winit::{event::Event, event_loop::EventLoop};

/// Platformer 引擎
///
/// 负责管理游戏的生命周期，包括：
/// - 初始化渲染器和窗口
/// - 初始化所有系统
/// - 加载游戏资源
/// - 执行主循环（事件处理 → 逻辑更新 → 渲染 → 呈现）
pub struct PlatformerEngine {
    /// 游戏配置
    pub config: PlatformerConfig,
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
    /// 输入系统
    input_system: InputSystem,
    /// 物理系统
    physics_system: PhysicsSystem,
    /// 移动系统
    movement_system: MovementSystem,
    /// 碰撞系统
    collision_system: CollisionSystem,
    /// 收集系统
    collectible_system: CollectibleSystem,
    /// AI 系统
    ai_system: AISystem,
    /// 渲染系统
    render_system: RenderSystem,
}

impl PlatformerEngine {
    /// 创建新的 Platformer 引擎实例
    ///
    /// # 参数
    ///
    /// - `config` - 游戏配置
    /// - `is_editor_mode` - 是否启用编辑器模式
    pub fn new(config: PlatformerConfig, is_editor_mode: bool) -> Self {
        let screen_width = config.display.width as f32;
        let screen_height = config.display.height as f32;

        Self {
            config,
            world: World::new(),
            asset_manager: AssetManager::new(Box::new(DesktopFileSystem::new())),
            is_editor_mode,
            renderer: None,
            ui_tree: UiTree::new(),
            input_system: InputSystem::new(config.input.move_speed, config.input.jump_force),
            physics_system: PhysicsSystem::new(
                config.gameplay.gravity,
                config.gameplay.max_fall_speed,
                config.gameplay.ground_friction,
                config.gameplay.air_friction,
            ),
            movement_system: MovementSystem::new(screen_width, screen_height),
            collision_system: CollisionSystem::new(0xFFFF),
            collectible_system: CollectibleSystem::new(),
            ai_system: AISystem::new(screen_width, screen_height),
            render_system: RenderSystem::new(screen_width, screen_height),
        }
    }

    /// 初始化引擎
    ///
    /// 创建渲染器、窗口，注册所有系统并加载游戏资源。
    pub fn initialize(&mut self) -> GResult<()> {
        // 注册系统
        self.world.add_system("input", Box::new(&mut self.input_system));
        self.world.add_system("physics", Box::new(&mut self.physics_system));
        self.world.add_system("movement", Box::new(&mut self.movement_system));
        self.world.add_system("collision", Box::new(&mut self.collision_system));
        self.world.add_system("collectible", Box::new(&mut self.collectible_system));
        self.world.add_system("ai", Box::new(&mut self.ai_system));
        self.world.add_system("render", Box::new(&mut self.render_system));

        // 创建玩家实体
        self.create_player();

        // 创建平台实体
        self.create_platforms();

        // 创建可收集物品实体
        self.create_collectibles();

        Ok(())
    }

    /// 创建玩家实体
    fn create_player(&mut self) {
        let player_entity = self.world.spawn();

        // 添加变换组件
        self.world.add_component(player_entity, Transform {
            x: 100.0,
            y: 400.0,
            rotation: 0.0,
            scale: 1.0,
        }).unwrap();

        // 添加速度组件
        self.world.add_component(player_entity, Velocity::default()).unwrap();

        // 添加精灵组件
        self.world.add_component(player_entity, Sprite {
            path: "player.png".to_string(),
            width: 32.0,
            height: 32.0,
            visible: true,
        }).unwrap();

        // 添加碰撞体组件
        self.world.add_component(player_entity, Collider {
            collider_type: ColliderType::Rectangle { width: 32.0, height: 32.0 },
            layer: self.config.physics.player_layer,
            mask: self.config.physics.platform_layer | self.config.physics.collectible_layer,
        }).unwrap();

        // 添加物理体组件
        self.world.add_component(player_entity, PhysicsBody {
            mass: 1.0,
            gravity_scale: 1.0,
            affected_by_gravity: true,
            is_static: false,
            on_ground: false,
            ground_normal: (0.0, 1.0),
        }).unwrap();

        // 添加生命值组件
        self.world.add_component(player_entity, Health {
            current: self.config.gameplay.player_health,
            max: self.config.gameplay.player_health,
        }).unwrap();

        // 添加玩家组件
        self.world.add_component(player_entity, Player {
            id: 1,
            can_jump: true,
            jump_count: 0,
            max_jump_count: 2,
        }).unwrap();
    }

    /// 创建平台实体
    fn create_platforms(&mut self) {
        // 创建地面平台
        let ground_entity = self.world.spawn();
        self.world.add_component(ground_entity, Transform {
            x: 400.0,
            y: 550.0,
            rotation: 0.0,
            scale: 1.0,
        }).unwrap();
        self.world.add_component(ground_entity, Collider {
            collider_type: ColliderType::Rectangle { width: 800.0, height: 50.0 },
            layer: self.config.physics.platform_layer,
            mask: self.config.physics.player_layer,
        }).unwrap();
        self.world.add_component(ground_entity, PhysicsBody {
            mass: 0.0,
            gravity_scale: 0.0,
            affected_by_gravity: false,
            is_static: true,
            on_ground: false,
            ground_normal: (0.0, 1.0),
        }).unwrap();
        self.world.add_component(ground_entity, Platform::default()).unwrap();

        // 创建平台 1
        let platform1_entity = self.world.spawn();
        self.world.add_component(platform1_entity, Transform {
            x: 200.0,
            y: 400.0,
            rotation: 0.0,
            scale: 1.0,
        }).unwrap();
        self.world.add_component(platform1_entity, Collider {
            collider_type: ColliderType::Rectangle { width: 150.0, height: 20.0 },
            layer: self.config.physics.platform_layer,
            mask: self.config.physics.player_layer,
        }).unwrap();
        self.world.add_component(platform1_entity, PhysicsBody {
            mass: 0.0,
            gravity_scale: 0.0,
            affected_by_gravity: false,
            is_static: true,
            on_ground: false,
            ground_normal: (0.0, 1.0),
        }).unwrap();
        self.world.add_component(platform1_entity, Platform::default()).unwrap();

        // 创建平台 2
        let platform2_entity = self.world.spawn();
        self.world.add_component(platform2_entity, Transform {
            x: 500.0,
            y: 300.0,
            rotation: 0.0,
            scale: 1.0,
        }).unwrap();
        self.world.add_component(platform2_entity, Collider {
            collider_type: ColliderType::Rectangle { width: 150.0, height: 20.0 },
            layer: self.config.physics.platform_layer,
            mask: self.config.physics.player_layer,
        }).unwrap();
        self.world.add_component(platform2_entity, PhysicsBody {
            mass: 0.0,
            gravity_scale: 0.0,
            affected_by_gravity: false,
            is_static: true,
            on_ground: false,
            ground_normal: (0.0, 1.0),
        }).unwrap();
        self.world.add_component(platform2_entity, Platform::default()).unwrap();
    }

    /// 创建可收集物品实体
    fn create_collectibles(&mut self) {
        // 创建金币 1
        let coin1_entity = self.world.spawn();
        self.world.add_component(coin1_entity, Transform {
            x: 200.0,
            y: 350.0,
            rotation: 0.0,
            scale: 1.0,
        }).unwrap();
        self.world.add_component(coin1_entity, Collider {
            collider_type: ColliderType::Circle { radius: 10.0 },
            layer: self.config.physics.collectible_layer,
            mask: self.config.physics.player_layer,
        }).unwrap();
        self.world.add_component(coin1_entity, Collectible::default()).unwrap();

        // 创建金币 2
        let coin2_entity = self.world.spawn();
        self.world.add_component(coin2_entity, Transform {
            x: 500.0,
            y: 250.0,
            rotation: 0.0,
            scale: 1.0,
        }).unwrap();
        self.world.add_component(coin2_entity, Collider {
            collider_type: ColliderType::Circle { radius: 10.0 },
            layer: self.config.physics.collectible_layer,
            mask: self.config.physics.player_layer,
        }).unwrap();
        self.world.add_component(coin2_entity, Collectible::default()).unwrap();
    }

    /// 使用事件循环初始化渲染器
    ///
    /// 必须在 `initialize` 之后、`run` 之前调用。
    ///
    /// # 参数
    ///
    /// - `event_loop` - winit 事件循环
    pub fn initialize_renderer(&mut self, event_loop: &EventLoop<()>) -> GResult<()> {
        let surface_info = SurfaceInfo::new(
            self.config.display.width,
            self.config.display.height,
            self.config.game.name.clone(),
        ).with_fullscreen(self.config.display.fullscreen);

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
            .map_err(|e| gg_core::GError {
                kind: gg_core::GErrorKind::Runtime,
                message: format!("Event loop error: {}", e),
            })
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

        let mut context = RenderContext::new(
            renderer.surface_info().width,
            renderer.surface_info().height,
        );

        LayoutEngine::compute(
            &mut self.ui_tree,
            renderer.surface_info().width as f32,
            renderer.surface_info().height as f32,
        );

        UiRenderer::render(&self.ui_tree, &mut context);

        renderer.draw(&context)?;
        renderer.end_frame()?;
        renderer.present()?;

        Ok(())
    }

    /// 设置按键状态
    pub fn set_key_state(&mut self, key: &str, pressed: bool) {
        self.input_system.set_key_state(key, pressed);
    }
}
