//! STG 引擎核心模块
//! 提供引擎初始化、主循环和插件管理功能

use crate::components::*;
use crate::config::StgConfig;
use crate::systems::*;
use gg_asset::AssetManager;
use gg_core::{GResult, plugin::Plugin};
use gg_ecs::World;
use gg_platform_desktop::DesktopFileSystem;
use gg_render::{RenderContext, Renderer, SurfaceInfo};
use gg_render_wgpu::WgpuRenderer;
use gg_ui::{LayoutEngine, UiRenderer, UiTree};
use winit::{event::Event, event_loop::EventLoop};

/// STG 引擎
///
/// 负责管理游戏的生命周期，包括：
/// - 初始化渲染器和窗口
/// - 初始化所有系统
/// - 加载游戏资源
/// - 执行主循环（事件处理 → 逻辑更新 → 渲染 → 呈现）
pub struct StgEngine {
    /// 游戏配置
    pub config: StgConfig,
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
    /// 移动系统
    movement_system: MovementSystem,
    /// 碰撞系统
    collision_system: CollisionSystem,
    /// 武器系统
    weapon_system: WeaponSystem,
    /// AI 系统
    ai_system: AISystem,
    /// 渲染系统
    render_system: RenderSystem,
}

impl StgEngine {
    /// 创建新的 STG 引擎实例
    ///
    /// # 参数
    ///
    /// - `config` - 游戏配置
    /// - `is_editor_mode` - 是否启用编辑器模式
    pub fn new(config: StgConfig, is_editor_mode: bool) -> Self {
        let screen_width = config.display.width as f32;
        let screen_height = config.display.height as f32;

        Self {
            config,
            world: World::new(),
            asset_manager: AssetManager::new(Box::new(DesktopFileSystem::new())),
            is_editor_mode,
            renderer: None,
            ui_tree: UiTree::new(),
            input_system: InputSystem::new(config.input.move_speed),
            movement_system: MovementSystem::new(screen_width, screen_height),
            collision_system: CollisionSystem::new(0xFFFF),
            weapon_system: WeaponSystem::new(config.gameplay.bullet_speed),
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
        self.world.add_system("movement", Box::new(&mut self.movement_system));
        self.world.add_system("collision", Box::new(&mut self.collision_system));
        self.world.add_system("weapon", Box::new(&mut self.weapon_system));
        self.world.add_system("ai", Box::new(&mut self.ai_system));
        self.world.add_system("render", Box::new(&mut self.render_system));

        // 创建玩家实体
        self.create_player();

        // 创建敌人实体
        self.create_enemy();

        Ok(())
    }

    /// 创建玩家实体
    fn create_player(&mut self) {
        let player_entity = self.world.spawn();

        // 添加变换组件
        self.world.add_component(player_entity, Transform {
            x: self.config.display.width as f32 / 2.0,
            y: self.config.display.height as f32 - 50.0,
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
            collider_type: ColliderType::Circle { radius: 16.0 },
            layer: 1,
            mask: 0xFFFF,
        }).unwrap();

        // 添加生命值组件
        self.world.add_component(player_entity, Health {
            current: self.config.gameplay.player_health,
            max: self.config.gameplay.player_health,
        }).unwrap();

        // 添加玩家组件
        self.world.add_component(player_entity, Player::default()).unwrap();

        // 添加武器组件
        self.world.add_component(player_entity, Weapon::default()).unwrap();
    }

    /// 创建敌人实体
    fn create_enemy(&mut self) {
        for i in 0..5 {
            let enemy_entity = self.world.spawn();

            // 添加变换组件
            self.world.add_component(enemy_entity, Transform {
                x: 50.0 + i * 150.0,
                y: 50.0,
                rotation: 0.0,
                scale: 1.0,
            }).unwrap();

            // 添加速度组件
            self.world.add_component(enemy_entity, Velocity::default()).unwrap();

            // 添加精灵组件
            self.world.add_component(enemy_entity, Sprite {
                path: "enemy.png".to_string(),
                width: 32.0,
                height: 32.0,
                visible: true,
            }).unwrap();

            // 添加碰撞体组件
            self.world.add_component(enemy_entity, Collider {
                collider_type: ColliderType::Circle { radius: 16.0 },
                layer: 4,
                mask: 0xFFFF,
            }).unwrap();

            // 添加生命值组件
            self.world.add_component(enemy_entity, Health {
                current: 2,
                max: 2,
            }).unwrap();

            // 添加敌人组件
            self.world.add_component(enemy_entity, Enemy {
                enemy_type: "basic".to_string(),
                level: 1,
                score: 100,
            }).unwrap();

            // 添加 AI 组件
            self.world.add_component(enemy_entity, AI {
                behavior: AIBehavior::Patrol,
                patrol_path: vec![
                    (50.0 + i * 150.0, 50.0),
                    (50.0 + i * 150.0, 200.0),
                    (50.0 + i * 150.0, 50.0),
                ],
                current_path_index: 0,
                move_speed: self.config.gameplay.enemy_speed,
            }).unwrap();
        }
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
