//! STG 引擎核心模块
//! 提供引擎初始化、主循环和插件管理功能

use crate::{
    boss::{Boss, BossMovePattern, BossPhase, PhaseTransition},
    bullet_pattern::{BulletEmitter, BulletPattern, BulletPool, PatternType},
    components::*,
    config::StgConfig,
    item::ItemSpawner,
    systems::*,
};
use gg_asset::AssetServer;
use gg_core::GResult;
use gg_ecs::World;
#[cfg(feature = "runtime")]
use gg_render::{RenderContext, Renderer, SurfaceInfo};
#[cfg(feature = "runtime")]
use gg_render_wgpu::WgpuRenderer;
#[cfg(feature = "runtime")]
use gg_ui::{LayoutEngine, UiRenderer, UiTree};
#[cfg(feature = "runtime")]
use winit::event::{ElementState, Event, WindowEvent};
#[cfg(feature = "runtime")]
use winit::event_loop::EventLoop;

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
    /// 资源服务器
    pub asset_server: AssetServer,
    /// 是否编辑器模式
    pub is_editor_mode: bool,
    /// WGPU 渲染器
    #[cfg(feature = "runtime")]
    renderer: Option<WgpuRenderer>,
    /// UI 节点树
    #[cfg(feature = "runtime")]
    ui_tree: UiTree,
}

impl StgEngine {
    /// 创建新的 STG 引擎实例
    ///
    /// # 参数
    ///
    /// - `config` - 游戏配置
    /// - `is_editor_mode` - 是否启用编辑器模式
    pub fn new(config: StgConfig, is_editor_mode: bool) -> Self {
        Self {
            config,
            world: World::new(),
            asset_server: AssetServer::new(),
            is_editor_mode,
            #[cfg(feature = "runtime")]
            renderer: None,
            #[cfg(feature = "runtime")]
            ui_tree: UiTree::new(),
        }
    }

    /// 初始化引擎
    ///
    /// 注册资源，创建并注册所有系统，创建游戏实体。
    pub fn initialize(&mut self) -> GResult<()> {
        let screen_width = self.config.display.width as f32;
        let screen_height = self.config.display.height as f32;

        self.world.insert_resource(InputState::default());
        self.world.insert_resource(ScreenBounds { width: screen_width, height: screen_height });
        self.world.insert_resource(GameState::default());
        self.world.insert_resource(DrawCommandBuffer::new());
        self.world.insert_resource(BulletPool::default());
        self.world.insert_resource(ItemSpawner::default());
        self.world.insert_resource(AudioBus::new());
        self.world.insert_resource(LevelConfig::default());

        self.world.register_system(Box::new(InputSystem::new(self.config.input.move_speed)));
        self.world.register_system(Box::new(MovementSystem));
        self.world.register_system(Box::new(WeaponSystem));
        self.world.register_system(Box::new(BulletEmitterSystem::new()));
        self.world.register_system(Box::new(BulletLifetimeSystem));
        self.world.register_system(Box::new(CollisionSystem::new(self.config.collision.cell_size)));
        self.world.register_system(Box::new(AISystem::new()));
        self.world.register_system(Box::new(BehaviorTreeSystem::new()));
        self.world.register_system(Box::new(BossAISystem));
        self.world.register_system(Box::new(ItemSystem));
        self.world.register_system(Box::new(InvulnerabilitySystem));
        self.world.register_system(Box::new(BoundarySystem));
        self.world.register_system(Box::new(RenderSystem));
        self.world.register_system(Box::new(AudioSystem::new(&self.config.audio)));
        self.world.register_system(Box::new(WaveSpawner::new(self.config.gameplay.enemy_speed)));
        self.world.register_system(Box::new(VfxSystem));
        self.world.register_system(Box::new(PauseSystem));
        self.world.register_system(Box::new(GameOverSystem));
        self.world.register_system(Box::new(BombSystem));

        self.create_player();
        self.create_enemies();
        self.create_boss();

        Ok(())
    }

    /// 创建玩家实体
    fn create_player(&mut self) {
        let player_entity = self.world.spawn().id();

        let _ = self.world.add_component(
            player_entity,
            Transform {
                x: self.config.display.width as f32 / 2.0,
                y: self.config.display.height as f32 - 50.0,
                rotation: 0.0,
                scale: 1.0,
            },
        );

        let _ = self.world.add_component(player_entity, Velocity::default());

        let _ = self
            .world
            .add_component(player_entity, Sprite { path: "player.png".to_string(), width: 32.0, height: 32.0, visible: true });

        let _ = self.world.add_component(
            player_entity,
            Collider {
                collider_type: ColliderType::Circle { radius: 16.0 },
                layer: self.config.collision.player_layer,
                mask: self.config.collision.enemy_layer
                    | self.config.collision.enemy_bullet_layer
                    | self.config.collision.item_layer,
            },
        );

        let _ = self.world.add_component(
            player_entity,
            Health { current: self.config.gameplay.player_health, max: self.config.gameplay.player_health },
        );

        let _ = self.world.add_component(
            player_entity,
            Player { id: player_entity, invulnerable: false, invulnerable_frames: 0, power_level: 1, bomb_count: 3 },
        );

        let _ = self.world.add_component(player_entity, Weapon::default());
    }

    /// 创建敌人实体
    fn create_enemies(&mut self) {
        for i in 0..5 {
            let enemy_entity = self.world.spawn().id();

            let _ = self
                .world
                .add_component(enemy_entity, Transform { x: 50.0 + i as f32 * 150.0, y: 50.0, rotation: 0.0, scale: 1.0 });

            let _ = self.world.add_component(enemy_entity, Velocity::default());

            let _ = self.world.add_component(
                enemy_entity,
                Sprite { path: "enemy.png".to_string(), width: 32.0, height: 32.0, visible: true },
            );

            let _ = self.world.add_component(
                enemy_entity,
                Collider {
                    collider_type: ColliderType::Circle { radius: 16.0 },
                    layer: self.config.collision.enemy_layer,
                    mask: self.config.collision.player_layer | self.config.collision.player_bullet_layer,
                },
            );

            let _ = self.world.add_component(enemy_entity, Health { current: 2, max: 2 });

            let _ = self.world.add_component(enemy_entity, Enemy::default());

            let _ = self.world.add_component(
                enemy_entity,
                AI {
                    behavior: AIBehavior::Patrol,
                    patrol_path: vec![
                        (50.0 + i as f32 * 150.0, 50.0),
                        (50.0 + i as f32 * 150.0, 200.0),
                        (50.0 + i as f32 * 150.0, 50.0),
                    ],
                    current_path_index: 0,
                    move_speed: self.config.gameplay.enemy_speed,
                },
            );

            let pattern = if i % 3 == 0 {
                BulletPattern {
                    pattern_type: PatternType::Aimed,
                    speed: 2.5,
                    count: 1,
                    interval: 90,
                    angle_offset: std::f32::consts::PI,
                }
            }
            else if i % 3 == 1 {
                BulletPattern {
                    pattern_type: PatternType::Spread { spread_angle: std::f32::consts::FRAC_PI_6 },
                    speed: 2.0,
                    count: 3,
                    interval: 120,
                    angle_offset: std::f32::consts::PI,
                }
            }
            else {
                BulletPattern { pattern_type: PatternType::Ring, speed: 2.0, count: 8, interval: 150, angle_offset: 0.0 }
            };

            let _ = self.world.add_component(
                enemy_entity,
                BulletEmitter {
                    patterns: vec![pattern],
                    current_pattern_index: 0,
                    current_pattern_fire_count: 0,
                    pattern_fire_limit: 0,
                    fire_timer: 0,
                    active: true,
                    spiral_angle: 0.0,
                },
            );
        }
    }

    /// 创建 Boss 实体
    fn create_boss(&mut self) {
        let boss_entity = self.world.spawn().id();

        let screen_center_x = self.config.display.width as f32 / 2.0;

        let _ = self.world.add_component(boss_entity, Transform { x: screen_center_x, y: 80.0, rotation: 0.0, scale: 1.0 });

        let _ = self.world.add_component(boss_entity, Velocity::default());

        let _ = self
            .world
            .add_component(boss_entity, Sprite { path: "boss.png".to_string(), width: 64.0, height: 64.0, visible: true });

        let _ = self.world.add_component(
            boss_entity,
            Collider {
                collider_type: ColliderType::Circle { radius: 32.0 },
                layer: self.config.collision.enemy_layer,
                mask: self.config.collision.player_layer | self.config.collision.player_bullet_layer,
            },
        );

        let boss_health = self.config.gameplay.boss_base_health;
        let _ = self.world.add_component(boss_entity, Health { current: boss_health, max: boss_health });

        let _ = self.world.add_component(boss_entity, Enemy { enemy_type: "boss".to_string(), level: 1, score: 5000 });

        let phase1_patterns = vec![
            BulletPattern { pattern_type: PatternType::Ring, speed: 2.0, count: 12, interval: 60, angle_offset: 0.0 },
            BulletPattern {
                pattern_type: PatternType::Aimed,
                speed: 3.0,
                count: 3,
                interval: 45,
                angle_offset: std::f32::consts::PI,
            },
        ];

        let phase2_patterns = vec![
            BulletPattern {
                pattern_type: PatternType::Spiral { angular_speed: 0.15 },
                speed: 2.5,
                count: 1,
                interval: 5,
                angle_offset: 0.0,
            },
            BulletPattern {
                pattern_type: PatternType::Spread { spread_angle: std::f32::consts::FRAC_PI_4 },
                speed: 3.0,
                count: 7,
                interval: 80,
                angle_offset: std::f32::consts::PI,
            },
        ];

        let phases = vec![
            BossPhase {
                name: "Phase 1 - Ring & Aimed".to_string(),
                bullet_patterns: phase1_patterns,
                move_pattern: BossMovePattern::Horizontal { range: 200.0, speed: 1.5 },
                transition: PhaseTransition::HealthBelow(0.5),
                duration_frames: 0,
            },
            BossPhase {
                name: "Phase 2 - Spiral & Spread".to_string(),
                bullet_patterns: phase2_patterns,
                move_pattern: BossMovePattern::Circular { radius: 100.0, angular_speed: 0.02 },
                transition: PhaseTransition::Manual,
                duration_frames: 0,
            },
        ];

        let _ = self.world.add_component(
            boss_entity,
            Boss {
                phases: phases.clone(),
                current_phase_index: 0,
                invulnerable_frames: 60,
                remaining_invulnerable_frames: 0,
                phase_elapsed_frames: 0,
                horizontal_offset: 0.0,
                horizontal_direction: 1.0,
                circular_angle: 0.0,
            },
        );

        let _ = self.world.add_component(
            boss_entity,
            BulletEmitter {
                patterns: phases[0].bullet_patterns.clone(),
                current_pattern_index: 0,
                current_pattern_fire_count: 0,
                pattern_fire_limit: 10,
                fire_timer: 0,
                active: true,
                spiral_angle: 0.0,
            },
        );
    }

    /// 使用事件循环初始化渲染器
    ///
    /// 必须在 `initialize` 之后、`run` 之前调用。
    ///
    /// # 参数
    ///
    /// - `event_loop` - winit 事件循环
    #[cfg(feature = "runtime")]
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
    #[cfg(feature = "runtime")]
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
                            match &event {
                                WindowEvent::KeyboardInput { event, .. } => {
                                    let pressed = event.state == ElementState::Pressed;
                                    match event.physical_key {
                                        winit::keyboard::PhysicalKey::Code(code) => {
                                            if let Some(input) = self.world.get_resource_mut::<InputState>() {
                                                match code {
                                                    winit::keyboard::KeyCode::ArrowUp | winit::keyboard::KeyCode::KeyW => {
                                                        input.up_pressed = pressed;
                                                    }
                                                    winit::keyboard::KeyCode::ArrowDown | winit::keyboard::KeyCode::KeyS => {
                                                        input.down_pressed = pressed;
                                                    }
                                                    winit::keyboard::KeyCode::ArrowLeft | winit::keyboard::KeyCode::KeyA => {
                                                        input.left_pressed = pressed;
                                                    }
                                                    winit::keyboard::KeyCode::ArrowRight | winit::keyboard::KeyCode::KeyD => {
                                                        input.right_pressed = pressed;
                                                    }
                                                    winit::keyboard::KeyCode::Space => {
                                                        input.shoot_pressed = pressed;
                                                    }
                                                    winit::keyboard::KeyCode::ShiftLeft
                                                    | winit::keyboard::KeyCode::ShiftRight => {
                                                        input.special_pressed = pressed;
                                                    }
                                                    _ => {}
                                                }
                                            }
                                        }
                                        _ => {}
                                    }
                                }
                                _ => {}
                            }
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
    #[cfg(feature = "runtime")]
    fn render_frame(&mut self) -> GResult<()> {
        let renderer = self.renderer.as_mut().ok_or_else(|| gg_core::GError {
            kind: gg_core::GErrorKind::Runtime,
            message: "Renderer not initialized".to_string(),
        })?;

        renderer.begin_frame()?;

        let mut context = RenderContext::new(renderer.surface_info().width, renderer.surface_info().height);

        LayoutEngine::compute(&mut self.ui_tree, renderer.surface_info().width as f32, renderer.surface_info().height as f32);

        UiRenderer::render(&self.ui_tree, &mut context);

        if let Some(buffer) = self.world.get_resource::<DrawCommandBuffer>() {
            for command in &buffer.commands {
                context.draw(command.clone());
            }
        }

        renderer.draw(&context)?;
        renderer.end_frame()?;
        renderer.present()?;

        Ok(())
    }
}
