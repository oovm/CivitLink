//! Galgame 引擎核心模块
//! 提供引擎初始化、主循环和插件管理功能

use std::collections::HashMap;
use std::path::Path;
use std::time::Instant;

use crate::config::GalgameConfig;
use gg_asset::{AssetServer, Handle, ImageAsset};
use gg_core::plugin::PluginManager;
use gg_core::{GError, GErrorKind, GResult};
use gg_ecs::{Entity, World};
use gg_galgame_schema::components::SceneBackground;
use gg_plugin_dialogue::plugin::DialoguePlugin;
use gg_plugin_dialogue::schema::{ChoiceState, DeltaTime, DialogueHistory};
use gg_plugin_dialogue::typewriter::TypewriterState;
use gg_plugin_portrait::plugin::PortraitPlugin;
use gg_plugin_portrait::systems::PortraitRenderSystem;
use gg_plugin_save::plugin::SavePlugin;
use gg_plugin_scene_transition::plugin::SceneTransitionPlugin;
use gg_plugin_scene_transition::systems::TransitionSystem;
use gg_plugin_ui::{EventSystemResource, UiPlugin, UiTreeResource};
use gg_render::{Color, DrawCommand, RenderContext, Renderer, SurfaceInfo, TextureId, Transform};
use gg_render_wgpu::WgpuRenderer;
use gg_ui::{
    FlexAlign, FlexDirection, FontStyle, LayoutEngine, LayoutStyle, SizeValue, Style, UiEvent,
    UiNodeData, UiNodeId, UiRenderer, UiTree,
};
use winit::event::{ElementState, Event, MouseButton};
use winit::event_loop::EventLoop;

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
    /// 资源服务器
    pub asset_server: AssetServer,
    /// 是否编辑器模式
    pub is_editor_mode: bool,
    /// WGPU 渲染器
    renderer: Option<WgpuRenderer>,
    /// UI 节点树
    ui_tree: UiTree,
    /// 插件管理器
    plugin_manager: PluginManager,
    /// 上一帧的时间戳
    last_frame_time: Option<Instant>,
    /// 鼠标位置
    mouse_position: [f32; 2],
    /// 待处理的推进对话请求
    pending_advance: bool,
    /// 待处理的 UI 点击分发请求
    pending_ui_click: bool,
    /// 纹理路径到 TextureId 的映射
    texture_map: HashMap<String, TextureId>,
    /// 纹理尺寸映射
    texture_sizes: HashMap<TextureId, [f32; 2]>,
    /// 对话面板节点 ID
    dialogue_panel_id: Option<UiNodeId>,
    /// 说话者名称节点 ID
    speaker_text_id: Option<UiNodeId>,
    /// 对话文本节点 ID
    dialogue_text_id: Option<UiNodeId>,
    /// 选项按钮容器节点 ID
    choice_container_id: Option<UiNodeId>,
    /// 选项按钮节点 ID 列表
    choice_button_ids: Vec<UiNodeId>,
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
            world: World::new(),
            asset_server: AssetServer::new(),
            is_editor_mode,
            renderer: None,
            ui_tree: UiTree::new(),
            plugin_manager: PluginManager::new(),
            last_frame_time: None,
            mouse_position: [0.0, 0.0],
            pending_advance: false,
            pending_ui_click: false,
            texture_map: HashMap::new(),
            texture_sizes: HashMap::new(),
            dialogue_panel_id: None,
            speaker_text_id: None,
            dialogue_text_id: None,
            choice_container_id: None,
            choice_button_ids: Vec::new(),
        }
    }

    /// 初始化引擎
    ///
    /// 注册所有插件，构建插件系统并初始化。
    pub fn initialize(&mut self) -> GResult<()> {
        let screen_width = self.config.display.width as f32;
        let screen_height = self.config.display.height as f32;

        self.plugin_manager.register(Box::new(DialoguePlugin))?;
        self.plugin_manager.register(Box::new(PortraitPlugin::new(screen_width, screen_height)))?;
        self.plugin_manager.register(Box::new(SceneTransitionPlugin))?;
        self.plugin_manager.register(Box::new(SavePlugin))?;
        self.plugin_manager.register(Box::new(UiPlugin))?;

        self.plugin_manager.build_all(&mut self.world)?;
        self.plugin_manager.initialize_all()?;

        self.build_dialogue_ui();

        Ok(())
    }

    /// 加载纹理到渲染器
    ///
    /// 从文件加载图像并注册到 WgpuRenderer 的纹理缓存中。
    /// 返回的 TextureId 可用于后续的 DrawCommand::Sprite 渲染。
    ///
    /// # 参数
    ///
    /// - `path` - 图像文件路径
    pub fn load_texture(&mut self, path: &str) -> GResult<TextureId> {
        if let Some(&id) = self.texture_map.get(path) {
            return Ok(id);
        }

        let renderer = self.renderer.as_mut().ok_or_else(|| GError {
            kind: GErrorKind::Runtime,
            message: "Renderer not initialized".to_string(),
        })?;

        let texture_id = renderer.load_texture(Path::new(path))?;

        let img_data = std::fs::read(path).ok();
        let size = if let Some(data) = img_data {
            image::load_from_memory(&data)
                .map(|img| [img.width() as f32, img.height() as f32])
                .unwrap_or([200.0, 400.0])
        } else {
            [200.0, 400.0]
        };
        self.texture_sizes.insert(texture_id, size);

        self.texture_map.insert(path.to_string(), texture_id);

        Ok(texture_id)
    }

    /// 查询纹理尺寸
    ///
    /// 根据纹理标识符查询对应的图像尺寸。
    /// 如果纹理未加载则返回 None。
    ///
    /// # 参数
    ///
    /// - `texture_id` - 纹理标识符
    pub fn texture_size(&self, texture_id: TextureId) -> Option<[f32; 2]> {
        self.texture_sizes.get(&texture_id).copied()
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
    /// 计算帧间隔时间并更新 DeltaTime 资源，然后执行所有已注册的 ECS 系统。
    pub fn tick(&mut self) -> GResult<()> {
        let now = Instant::now();
        let delta = match self.last_frame_time {
            Some(last) => now.duration_since(last).as_secs_f32(),
            None => 1.0 / 60.0,
        };
        self.last_frame_time = Some(now);

        if let Some(dt) = self.world.get_resource_mut::<DeltaTime>() {
            dt.secs = delta;
        }

        self.world.run_systems()?;

        self.update_dialogue_ui();
        self.update_choice_buttons();

        Ok(())
    }

    /// 处理输入事件
    ///
    /// 检查是否需要推进对话或跳过打字机效果。
    fn handle_input(&mut self) {
        let should_advance = self.pending_advance;
        if !should_advance {
            return;
        }
        self.pending_advance = false;

        if let Some(state) = self.world.get_resource_mut::<TypewriterState>() {
            if !state.is_complete() {
                state.skip();
                return;
            }
        }

        let has_active_choices = self.world
            .get_component::<ChoiceState>(Entity::new(0, 0))
            .map(|c| c.is_active)
            .unwrap_or(false);

        if !has_active_choices {
            self.world.remove_resource::<TypewriterState>();
        }
    }

    /// 将鼠标点击事件分发到 UI 事件系统
    ///
    /// 先将点击事件分发到 UI 事件系统，然后检测是否点击了选项按钮。
    /// 如果点击了选项按钮，设置 ChoiceState 的 selected_index 并停用选项。
    fn dispatch_click_to_ui(&mut self) {
        let tree = self.world.get_resource::<UiTreeResource>().map(|r| r.0.clone());
        if let Some(event_sys_res) = self.world.get_resource_mut::<EventSystemResource>() {
            if let Some(ref tree) = tree {
                event_sys_res.0.dispatch(
                    &UiEvent::Click {
                        x: self.mouse_position[0],
                        y: self.mouse_position[1],
                    },
                    tree,
                );
            }
        }

        let click_x = self.mouse_position[0];
        let click_y = self.mouse_position[1];

        let button_layouts: Vec<(usize, f32, f32, f32, f32)> = self
            .choice_button_ids
            .iter()
            .enumerate()
            .filter_map(|(i, &button_id)| {
                let node = self.ui_tree.get(button_id)?;
                let layout = node.layout_result?;
                Some((i, layout.x, layout.y, layout.width, layout.height))
            })
            .collect();

        for (i, x, y, w, h) in &button_layouts {
            if click_x >= *x && click_x <= *x + *w && click_y >= *y && click_y <= *y + *h {
                if let Some(choice_state) = self.world.get_resource_mut::<ChoiceState>() {
                    choice_state.selected_index = Some(*i);
                    choice_state.is_active = false;
                }
                return;
            }
        }
    }

    /// 构建对话 UI 布局
    ///
    /// 创建底部对话文本框和说话者名称的 UI 节点。
    /// 布局结构为：根容器（纵向，底部对齐）→ 选项容器 → 对话面板（说话者 + 文本）。
    fn build_dialogue_ui(&mut self) {
        if self.ui_tree.root().is_none() {
            let root_style = Style {
                layout: LayoutStyle {
                    direction: FlexDirection::Column,
                    justify_content: FlexAlign::End,
                    width: SizeValue::Percent(1.0),
                    height: SizeValue::Percent(1.0),
                    ..Default::default()
                },
                ..Default::default()
            };
            let root_id = self.ui_tree.create_node("root", root_style, UiNodeData::Container);
            self.ui_tree.set_root(root_id);
        }
        let root_id = self.ui_tree.root().unwrap();

        let choice_container_style = Style {
            layout: LayoutStyle {
                direction: FlexDirection::Column,
                width: SizeValue::Percent(1.0),
                height: SizeValue::Auto,
                padding: 10.0,
                gap: 5.0,
                ..Default::default()
            },
            ..Default::default()
        };
        let choice_container_id = self.ui_tree.create_node(
            "choice_container",
            choice_container_style,
            UiNodeData::Container,
        );
        self.ui_tree.add_child(root_id, choice_container_id);
        self.choice_container_id = Some(choice_container_id);

        let panel_style = Style {
            layout: LayoutStyle {
                direction: FlexDirection::Column,
                width: SizeValue::Percent(1.0),
                height: SizeValue::Px(200.0),
                padding: 10.0,
                ..Default::default()
            },
            background_color: Some(Color::new(0.0, 0.0, 0.0, 0.7)),
            ..Default::default()
        };
        let panel_id = self.ui_tree.create_node("dialogue_panel", panel_style, UiNodeData::Container);
        self.ui_tree.add_child(root_id, panel_id);
        self.dialogue_panel_id = Some(panel_id);

        let speaker_style = Style {
            layout: LayoutStyle {
                width: SizeValue::Percent(1.0),
                height: SizeValue::Px(30.0),
                ..Default::default()
            },
            font: Some(FontStyle {
                size: 20.0,
                color: Color::new(1.0, 0.9, 0.3, 1.0),
                ..Default::default()
            }),
            ..Default::default()
        };
        let speaker_id = self.ui_tree.create_node(
            "speaker_name",
            speaker_style,
            UiNodeData::Text { content: String::new() },
        );
        self.ui_tree.add_child(panel_id, speaker_id);
        self.speaker_text_id = Some(speaker_id);

        let text_style = Style {
            layout: LayoutStyle {
                width: SizeValue::Percent(1.0),
                height: SizeValue::Px(160.0),
                ..Default::default()
            },
            font: Some(FontStyle {
                size: 18.0,
                color: Color::WHITE,
                ..Default::default()
            }),
            ..Default::default()
        };
        let text_id = self.ui_tree.create_node(
            "dialogue_text",
            text_style,
            UiNodeData::Text { content: String::new() },
        );
        self.ui_tree.add_child(panel_id, text_id);
        self.dialogue_text_id = Some(text_id);
    }

    /// 更新对话 UI
    ///
    /// 根据 TypewriterState 和 ChoiceState 更新对话文本框内容。
    /// 当存在打字机文本或活跃选项时显示对话面板，否则隐藏。
    fn update_dialogue_ui(&mut self) {
        let typewriter_text = self
            .world
            .get_resource::<TypewriterState>()
            .map(|t| t.current_text().to_string());
        let has_choices = self
            .world
            .get_resource::<ChoiceState>()
            .map(|c| c.is_active)
            .unwrap_or(false);
        let speaker_name = self
            .world
            .get_resource::<DialogueHistory>()
            .and_then(|h| h.entries.last().map(|e| e.speaker_name.clone().unwrap_or_default()));

        if let Some(text_id) = self.dialogue_text_id {
            if let Some(ref text) = typewriter_text {
                if let Some(node) = self.ui_tree.get_mut(text_id) {
                    if let UiNodeData::Text { ref mut content } = node.data {
                        *content = text.clone();
                    }
                }
            }
        }

        if let Some(speaker_id) = self.speaker_text_id {
            if let Some(node) = self.ui_tree.get_mut(speaker_id) {
                if let UiNodeData::Text { ref mut content } = node.data {
                    *content = speaker_name.unwrap_or_default();
                }
            }
        }

        if let Some(panel_id) = self.dialogue_panel_id {
            let should_show = typewriter_text.is_some() || has_choices;
            if let Some(node) = self.ui_tree.get_mut(panel_id) {
                node.visible = should_show;
            }
        }
    }

    /// 更新选项按钮
    ///
    /// 根据 ChoiceState 动态创建或移除选项按钮。
    /// 当选项不活跃时仅移除已有按钮，活跃时重建所有按钮。
    fn update_choice_buttons(&mut self) {
        let choice_state = self
            .world
            .get_resource::<ChoiceState>()
            .map(|c| (c.is_active, c.choices.clone()));

        let (is_active, choices) = match choice_state {
            Some((active, ch)) => (active, ch),
            None => (false, Vec::new()),
        };

        for &button_id in &self.choice_button_ids {
            self.ui_tree.remove_node(button_id);
        }
        self.choice_button_ids.clear();

        if let Some(container_id) = self.choice_container_id {
            if let Some(node) = self.ui_tree.get_mut(container_id) {
                node.visible = is_active;
            }
        }

        if !is_active {
            return;
        }

        let container_id = match self.choice_container_id {
            Some(id) => id,
            None => return,
        };

        for (i, choice) in choices.iter().enumerate() {
            let button_style = Style {
                layout: LayoutStyle {
                    width: SizeValue::Px(300.0),
                    height: SizeValue::Px(40.0),
                    ..Default::default()
                },
                background_color: Some(Color::new(0.2, 0.2, 0.4, 0.9)),
                ..Default::default()
            };

            let button_id = self.ui_tree.create_node(
                format!("choice_{}", i),
                button_style,
                UiNodeData::Text { content: choice.text.clone() },
            );
            self.ui_tree.add_child(container_id, button_id);
            self.choice_button_ids.push(button_id);
        }
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
                            match &event {
                                winit::event::WindowEvent::KeyboardInput { event, .. } => {
                                    if event.state == ElementState::Pressed {
                                        match event.physical_key {
                                            winit::keyboard::PhysicalKey::Code(
                                                winit::keyboard::KeyCode::Enter,
                                            )
                                            | winit::keyboard::PhysicalKey::Code(
                                                winit::keyboard::KeyCode::Space,
                                            ) => {
                                                self.pending_advance = true;
                                            }
                                            _ => {}
                                        }
                                    }
                                }
                                winit::event::WindowEvent::MouseInput { state, button, .. } => {
                                    if *state == ElementState::Pressed && *button == MouseButton::Left {
                                        self.pending_advance = true;
                                        self.pending_ui_click = true;
                                    }
                                }
                                winit::event::WindowEvent::CursorMoved { position, .. } => {
                                    self.mouse_position = [position.x as f32, position.y as f32];
                                }
                                _ => {}
                            }
                            if renderer.should_close() {
                                elwt.exit();
                            }
                        }
                        Event::AboutToWait => {
                            if self.pending_ui_click {
                                self.pending_ui_click = false;
                                self.dispatch_click_to_ui();
                            }

                            self.handle_input();

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

        let entities: Vec<Entity> = self.world.entities().iter().copied().collect();
        for entity in entities {
            if let Some(bg) = self.world.get_component::<SceneBackground>(entity) {
                if bg.texture_id != TextureId::INVALID {
                    let surface_width = renderer.surface_info().width as f32;
                    let surface_height = renderer.surface_info().height as f32;
                    context.draw(DrawCommand::Sprite {
                        texture_id: bg.texture_id,
                        transform: Transform {
                            position: [0.0, 0.0],
                            scale: [1.0, 1.0],
                            rotation: 0.0,
                            z_index: -1.0,
                        },
                        size: [surface_width, surface_height],
                        tint: Color::WHITE,
                        clip_rect: None,
                    });
                }
            }
        }

        let portrait_system = PortraitRenderSystem::new(
            renderer.surface_info().width as f32,
            renderer.surface_info().height as f32,
        );
        portrait_system.render_to_context(&self.world, &mut context)?;

        let transition_system = TransitionSystem::new();
        transition_system.render_to_context(&self.world, &mut context)?;

        LayoutEngine::compute(&mut self.ui_tree, renderer.surface_info().width as f32, renderer.surface_info().height as f32);

        UiRenderer::render(&self.ui_tree, &mut context);

        renderer.draw(&context)?;
        renderer.end_frame()?;
        renderer.present()?;

        Ok(())
    }
}
