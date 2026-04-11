//! 编辑器壳程序

use std::collections::HashMap;

use crate::{
    command::{CommandManager, ModifierState, ShortcutKey, ShortcutRegistry},
    context::{EditorConfig, EditorContext},
    docking::{DockRegion, DockingLayout},
    event::{EditorEvent, EventBus, Key, MouseButton},
    extension::ExtensionPointRegistry,
    panel::EditorPanel,
    plugin::{EditorPlugin, PluginDescriptor, PluginManager},
    service::{ServiceRegistry, WindowId, WinitWindowService},
};
use gg_core::{GError, GErrorKind, GResult};
use gg_editor_render::EditorRenderer;
use gg_render::{Color, SurfaceInfo};
use gg_ui::{LayoutEngine, LayoutResult, Style, UiNodeData, UiNodeId, UiTree};
use gg_world::GameWorld;

/// 编辑器壳程序
///
/// 微内核架构的核心，持有服务注册表、命令管理器、事件总线、面板和插件，
/// 提供统一的编辑器生命周期管理。
pub struct EditorShell {
    /// 服务注册表
    services: ServiceRegistry,
    /// 命令管理器
    commands: CommandManager,
    /// 事件总线
    events: EventBus,
    /// 已注册的面板列表
    panels: Vec<Box<dyn EditorPanel>>,
    /// 插件管理器
    plugin_manager: PluginManager,
    /// UI 节点树
    ui_tree: UiTree,
    /// 是否运行中
    is_running: bool,
    /// Winit 窗口服务
    window_service: WinitWindowService,
    /// 编辑器配置
    editor_config: EditorConfig,
    /// 编辑器渲染器实例映射（窗口 ID → 渲染器）
    renderers: HashMap<u64, EditorRenderer>,
    /// 窗口尺寸映射（窗口 ID → (宽, 高)）
    window_sizes: HashMap<u64, (u32, u32)>,
    /// Docking 布局管理器
    docking_layout: DockingLayout,
    /// 当前鼠标位置 (x, y)
    cursor_position: (f32, f32),
    /// 游戏世界实例
    world: GameWorld,
    /// 面板名称到其 UI 根节点 ID 的映射
    panel_root_nodes: Vec<(String, UiNodeId)>,
    /// 快捷键注册表
    shortcut_registry: ShortcutRegistry,
    /// 修饰键状态
    modifier_state: ModifierState,
}

impl EditorShell {
    /// 创建新的编辑器壳程序
    pub fn new() -> Self {
        let mut shortcut_registry = ShortcutRegistry::new();
        shortcut_registry.register(ShortcutKey::new(Key::Z).with_ctrl(), "undo".to_string());
        shortcut_registry.register(ShortcutKey::new(Key::Y).with_ctrl(), "redo".to_string());
        shortcut_registry.register(ShortcutKey::new(Key::S).with_ctrl(), "save".to_string());
        shortcut_registry.register(ShortcutKey::new(Key::Delete), "delete".to_string());

        let mut window_sizes = HashMap::new();
        window_sizes.insert(0, (1280, 720));

        Self {
            services: ServiceRegistry::new(),
            commands: CommandManager::new(),
            events: EventBus::new(),
            panels: Vec::new(),
            plugin_manager: PluginManager::new(),
            ui_tree: UiTree::new(),
            is_running: false,
            window_service: WinitWindowService::new(),
            editor_config: EditorConfig::default(),
            renderers: HashMap::new(),
            window_sizes,
            docking_layout: DockingLayout::default_layout(),
            cursor_position: (0.0, 0.0),
            world: GameWorld::new("editor_world".to_string()),
            panel_root_nodes: Vec::new(),
            shortcut_registry,
            modifier_state: ModifierState::default(),
            extension_points: ExtensionPointRegistry::new(),
        }
    }

    /// 创建新窗口
    ///
    /// 通过窗口服务创建浮动窗口，并将窗口尺寸记录到 `window_sizes` 中。
    /// 实际的 `EditorRenderer` 创建发生在 `run_with_renderer` 中窗口被 winit 创建时。
    pub fn create_window(&mut self, title: &str, size: (f32, f32)) -> WindowId {
        let window_id = self.window_service.create_floating_window(title, size);
        self.window_sizes.insert(window_id.0, (size.0 as u32, size.1 as u32));
        window_id
    }

    /// 销毁窗口
    ///
    /// 移除窗口对应的渲染器和尺寸记录，通过窗口服务销毁窗口，
    /// 并发布 `WindowDestroyed` 事件。
    pub fn destroy_window(&mut self, window_id: u64) {
        self.renderers.remove(&window_id);
        self.window_sizes.remove(&window_id);
        self.window_service.destroy_window(WindowId(window_id));
        self.events.publish(EditorEvent::WindowDestroyed { window_id });
    }

    /// 注册面板
    ///
    /// 将面板添加到面板列表，调用其 `on_register` 方法，并发布 `PanelRegistered` 事件。
    pub fn register_panel(&mut self, panel: Box<dyn EditorPanel>) {
        let panel_name = panel.name().to_string();
        self.panels.push(panel);
        let mut context = EditorContext::new(
            &mut self.services,
            &mut self.commands,
            &mut self.events,
            &mut self.world,
            &mut self.extension_points,
        );
        if let Some(panel) = self.panels.last_mut() {
            panel.on_register(&mut context);
        }
        self.events.publish(EditorEvent::PanelRegistered { panel_name });
    }

    /// 动态注册面板
    ///
    /// 与 `register_panel` 功能相同，用于运行时动态注册面板，
    /// 调用 `on_register` 方法并发布 `PanelRegistered` 事件。
    pub fn register_panel_dynamic(&mut self, panel: Box<dyn EditorPanel>) {
        self.register_panel(panel);
    }

    /// 注销面板
    ///
    /// 根据面板名称移除面板，调用其 `on_unregister` 方法，并发布 `PanelUnregistered` 事件。
    pub fn unregister_panel(&mut self, name: &str) {
        if let Some(pos) = self.panels.iter().position(|p| p.name() == name) {
            let mut panel = self.panels.remove(pos);
            let mut context = EditorContext::new(
                &mut self.services,
                &mut self.commands,
                &mut self.events,
                &mut self.world,
                &mut self.extension_points,
            );
            panel.on_unregister(&mut context);
            self.events.publish(EditorEvent::PanelUnregistered { panel_name: name.to_string() });
        }
    }

    /// 注册插件
    pub fn register_plugin(&mut self, plugin: Box<dyn EditorPlugin>) {
        let name = plugin.name().to_string();
        let descriptor = PluginDescriptor {
            name: name.clone(),
            version: "0.1.0".to_string(),
            dependencies: plugin.dependencies().iter().map(|s| s.to_string()).collect(),
        };
        self.plugin_manager.register(plugin, descriptor);
    }

    /// 加载插件
    ///
    /// 注册并激活插件，发布 `PluginLoaded` 事件。
    pub fn load_plugin(&mut self, plugin: Box<dyn EditorPlugin>, descriptor: PluginDescriptor) -> GResult<()> {
        let name = plugin.name().to_string();
        self.plugin_manager.register(plugin, descriptor);
        {
            let mut context = EditorContext::new(
                &mut self.services,
                &mut self.commands,
                &mut self.events,
                &mut self.world,
                &mut self.extension_points,
            );
            self.plugin_manager.activate(&name, &mut context)?;
        }
        self.events.publish(EditorEvent::PluginLoaded { plugin_name: name });
        Ok(())
    }

    /// 卸载插件
    ///
    /// 停用并卸载插件，发布 `PluginUnloaded` 事件。
    pub fn unload_plugin(&mut self, name: &str) -> GResult<()> {
        {
            let mut context = EditorContext::new(
                &mut self.services,
                &mut self.commands,
                &mut self.events,
                &mut self.world,
                &mut self.extension_points,
            );
            self.plugin_manager.unload(name, &mut context)?;
        }
        self.events.publish(EditorEvent::PluginUnloaded { plugin_name: name.to_string() });
        Ok(())
    }

    /// 重载插件
    ///
    /// 停用后重新激活插件，发布 `PluginReloaded` 事件。
    pub fn reload_plugin(&mut self, name: &str) -> GResult<()> {
        {
            let mut context = EditorContext::new(
                &mut self.services,
                &mut self.commands,
                &mut self.events,
                &mut self.world,
                &mut self.extension_points,
            );
            self.plugin_manager.reload(name, &mut context)?;
        }
        self.events.publish(EditorEvent::PluginReloaded { plugin_name: name.to_string() });
        Ok(())
    }

    /// 加载动态库插件
    ///
    /// 从指定路径加载 `plugin.json` 清单和动态库，注册并激活插件。
    #[cfg(not(target_arch = "wasm32"))]
    pub fn load_dynamic_plugin(&mut self, path: &std::path::Path) -> GResult<()> {
        let name;
        {
            let mut context = EditorContext::new(
                &mut self.services,
                &mut self.commands,
                &mut self.events,
                &mut self.world,
                &mut self.extension_points,
            );
            self.plugin_manager.load_dynamic(path, &mut context)?;
            name = self.plugin_manager.iter_active().last().map(|p| p.name().to_string()).unwrap_or_default();
        }
        self.events.publish(EditorEvent::PluginLoaded { plugin_name: name });
        Ok(())
    }

    /// 卸载动态库插件
    ///
    /// 停用并卸载动态库插件，释放动态库句柄。
    #[cfg(not(target_arch = "wasm32"))]
    pub fn unload_dynamic_plugin(&mut self, name: &str) -> GResult<()> {
        {
            let mut context = EditorContext::new(
                &mut self.services,
                &mut self.commands,
                &mut self.events,
                &mut self.world,
                &mut self.extension_points,
            );
            self.plugin_manager.unload_dynamic(name, &mut context)?;
        }
        self.events.publish(EditorEvent::PluginUnloaded { plugin_name: name.to_string() });
        Ok(())
    }

    /// 保存布局到文件
    ///
    /// 将当前 Docking 布局序列化为 JSON 并写入指定路径，发布 `LayoutSaved` 事件。
    pub fn save_layout(&mut self, path: &str) -> GResult<()> {
        let snapshot = self.docking_layout.to_snapshot();
        let json = serde_json::to_string_pretty(&snapshot)
            .map_err(|e| GError { kind: GErrorKind::Io, message: format!("布局序列化失败: {}", e) })?;
        std::fs::write(path, json)
            .map_err(|e| GError { kind: GErrorKind::Io, message: format!("布局文件写入失败: {}", e) })?;
        self.editor_config.last_layout_path = Some(path.to_string());
        self.events.publish(EditorEvent::LayoutSaved { path: path.to_string() });
        Ok(())
    }

    /// 从文件加载布局
    ///
    /// 从指定路径读取 JSON 并反序列化为 DockingLayout，发布 `LayoutLoaded` 事件。
    /// 如果文件不存在或格式错误，使用默认布局并发布 `LayoutReset` 事件。
    pub fn load_layout(&mut self, path: &str) -> GResult<()> {
        match std::fs::read_to_string(path) {
            Ok(json) => match serde_json::from_str::<crate::docking::LayoutSnapshot>(&json) {
                Ok(snapshot) => {
                    self.docking_layout = DockingLayout::from_snapshot(snapshot);
                    self.editor_config.last_layout_path = Some(path.to_string());
                    self.events.publish(EditorEvent::LayoutLoaded { path: path.to_string() });
                }
                Err(_) => {
                    self.docking_layout = DockingLayout::default_layout();
                    self.events.publish(EditorEvent::LayoutReset);
                }
            },
            Err(_) => {
                self.docking_layout = DockingLayout::default_layout();
                self.events.publish(EditorEvent::LayoutReset);
            }
        }
        Ok(())
    }

    /// 获取服务注册表引用
    pub fn service_registry(&self) -> &ServiceRegistry {
        &self.services
    }

    /// 获取服务注册表可变引用
    pub fn service_registry_mut(&mut self) -> &mut ServiceRegistry {
        &mut self.services
    }

    /// 获取命令管理器引用
    pub fn command_manager(&self) -> &CommandManager {
        &self.commands
    }

    /// 获取命令管理器可变引用
    pub fn command_manager_mut(&mut self) -> &mut CommandManager {
        &mut self.commands
    }

    /// 获取事件总线引用
    pub fn event_bus(&self) -> &EventBus {
        &self.events
    }

    /// 获取事件总线可变引用
    pub fn event_bus_mut(&mut self) -> &mut EventBus {
        &mut self.events
    }

    /// 获取窗口服务引用
    pub fn window_service(&self) -> &WinitWindowService {
        &self.window_service
    }

    /// 获取窗口服务可变引用
    pub fn window_service_mut(&mut self) -> &mut WinitWindowService {
        &mut self.window_service
    }

    /// 获取编辑器配置引用
    pub fn editor_config(&self) -> &EditorConfig {
        &self.editor_config
    }

    /// 获取编辑器配置可变引用
    pub fn editor_config_mut(&mut self) -> &mut EditorConfig {
        &mut self.editor_config
    }

    /// 获取 Docking 布局管理器引用
    pub fn docking_layout(&self) -> &DockingLayout {
        &self.docking_layout
    }

    /// 获取 Docking 布局管理器可变引用
    pub fn docking_layout_mut(&mut self) -> &mut DockingLayout {
        &mut self.docking_layout
    }

    /// 获取游戏世界引用
    pub fn world(&self) -> &GameWorld {
        &self.world
    }

    /// 获取游戏世界可变引用
    pub fn world_mut(&mut self) -> &mut GameWorld {
        &mut self.world
    }

    /// 获取快捷键注册表引用
    pub fn shortcut_registry(&self) -> &ShortcutRegistry {
        &self.shortcut_registry
    }

    /// 获取快捷键注册表可变引用
    pub fn shortcut_registry_mut(&mut self) -> &mut ShortcutRegistry {
        &mut self.shortcut_registry
    }

    /// 获取插件管理器引用
    pub fn plugin_manager(&self) -> &PluginManager {
        &self.plugin_manager
    }

    /// 获取插件管理器可变引用
    pub fn plugin_manager_mut(&mut self) -> &mut PluginManager {
        &mut self.plugin_manager
    }

    /// 获取主窗口渲染器引用
    pub fn editor_renderer(&self) -> Option<&EditorRenderer> {
        self.renderers.get(&0)
    }

    /// 获取主窗口渲染器可变引用
    pub fn editor_renderer_mut(&mut self) -> Option<&mut EditorRenderer> {
        self.renderers.get_mut(&0)
    }

    /// 执行一帧
    ///
    /// 先处理待处理事件，再处理窗口焦点事件，
    /// 然后构建所有可见面板的 UI 节点树，使用 DockingLayout 计算面板布局，
    /// 将布局结果应用到面板根节点，对每个面板子树进行 Flexbox 布局计算，
    /// 最后添加区域分隔条 UI 节点。
    pub fn tick(&mut self) -> GResult<()> {
        self.ui_tree.clear();

        let pending_events = self.events.process_pending();

        for window_id in self.window_service.drain_pending_focus() {
            self.events.publish(EditorEvent::WindowFocused { window_id });
        }

        for event in &pending_events {
            let mut context = EditorContext::new(
                &mut self.services,
                &mut self.commands,
                &mut self.events,
                &mut self.world,
                &mut self.extension_points,
            );
            for panel in &mut self.panels {
                panel.on_event(event, &mut context);
            }
        }

        let editor_root_id = self.ui_tree.create_node(
            "editor_root",
            Style::new().with_background_color(Color::new(0.5, 0.0, 1.0, 1.0)),
            UiNodeData::Container,
        );

        let mut context = EditorContext::new(
            &mut self.services,
            &mut self.commands,
            &mut self.events,
            &mut self.world,
            &mut self.extension_points,
        );
        self.panel_root_nodes.clear();

        for panel in self.panels.iter_mut() {
            if panel.is_visible() {
                if let Some(panel_root_id) = panel.build_ui(&mut context, &mut self.ui_tree) {
                    self.ui_tree.add_child(editor_root_id, panel_root_id);
                    self.panel_root_nodes.push((panel.name().to_string(), panel_root_id));
                }
            }
        }

        self.ui_tree.set_root(editor_root_id);

        let (win_w, win_h) = self.window_sizes.get(&0).copied().unwrap_or((1280, 720));

        if let Some(node) = self.ui_tree.get_mut(editor_root_id) {
            node.layout_result = Some(LayoutResult::new(0.0, 0.0, win_w as f32, win_h as f32));
        }

        let panel_layouts = self.docking_layout.compute(&self.panels, win_w as f32, win_h as f32);

        for layout in panel_layouts.iter() {
            if let Some((_, node_id)) = self.panel_root_nodes.iter().find(|(name, _)| name == &layout.name) {
                LayoutEngine::compute_subtree(&mut self.ui_tree, *node_id, layout.width, layout.height);

                if let Some(node) = self.ui_tree.get_mut(*node_id) {
                    node.layout_result = Some(LayoutResult::new(layout.x, layout.y, layout.width, layout.height));
                }
            }
        }

        let split_color = Color::new(0.5, 0.5, 0.5, 1.0);
        let split_thickness = 2.0f32;
        let win_w_f = win_w as f32;
        let win_h_f = win_h as f32;
        let bottom_height = self.docking_layout.bottom.as_ref().map(|b| b.size).unwrap_or(0.0);

        for split in &self.docking_layout.splits {
            let (x, y, w, h) = match split.region {
                DockRegion::Left => (split.position - split_thickness / 2.0, 0.0, split_thickness, win_h_f - bottom_height),
                DockRegion::Right => {
                    (win_w_f - split.position - split_thickness / 2.0, 0.0, split_thickness, win_h_f - bottom_height)
                }
                DockRegion::Bottom => (0.0, win_h_f - split.position - split_thickness / 2.0, win_w_f, split_thickness),
                DockRegion::Center => continue,
            };

            let region_name = match split.region {
                DockRegion::Left => "left",
                DockRegion::Right => "right",
                DockRegion::Bottom => "bottom",
                DockRegion::Center => continue,
            };
            let split_node_id = self.ui_tree.create_node(
                format!("split_{}", region_name),
                Style::new().with_background_color(split_color),
                UiNodeData::Container,
            );
            self.ui_tree.add_child(editor_root_id, split_node_id);

            if let Some(node) = self.ui_tree.get_mut(split_node_id) {
                node.layout_result = Some(LayoutResult::new(x, y, w, h));
            }
        }

        Ok(())
    }

    /// 运行主循环
    ///
    /// 激活所有已注册插件，进入帧循环，
    /// 退出后停用所有插件。
    pub fn run(&mut self) -> GResult<()> {
        self.is_running = true;
        {
            let mut context = EditorContext::new(
                &mut self.services,
                &mut self.commands,
                &mut self.events,
                &mut self.world,
                &mut self.extension_points,
            );
            self.plugin_manager.activate_all(&mut context)?;
        }
        while self.is_running {
            self.tick()?;
        }
        {
            let mut context = EditorContext::new(
                &mut self.services,
                &mut self.commands,
                &mut self.events,
                &mut self.world,
                &mut self.extension_points,
            );
            self.plugin_manager.deactivate_all(&mut context);
        }
        Ok(())
    }

    /// 使用延迟初始化模式运行主循环
    ///
    /// 先创建窗口并显示，再按优先级顺序初始化面板和插件。
    pub fn run_lazy(&mut self) -> GResult<()> {
        self.is_running = true;

        let mut panel_priorities: Vec<(u32, String)> =
            self.panels.iter().map(|p| (p.priority(), p.name().to_string())).collect();
        panel_priorities.sort_by_key(|(priority, _)| *priority);

        for (_, name) in &panel_priorities {
            if let Some(panel) = self.panels.iter_mut().find(|p| p.name() == *name) {
                let mut context = EditorContext::new(
                    &mut self.services,
                    &mut self.commands,
                    &mut self.events,
                    &mut self.world,
                    &mut self.extension_points,
                );
                panel.on_register(&mut context);
                let _ = self.tick();
            }
        }

        {
            let mut context = EditorContext::new(
                &mut self.services,
                &mut self.commands,
                &mut self.events,
                &mut self.world,
                &mut self.extension_points,
            );
            self.plugin_manager.activate_all(&mut context)?;
        }

        while self.is_running {
            self.tick()?;
        }

        {
            let mut context = EditorContext::new(
                &mut self.services,
                &mut self.commands,
                &mut self.events,
                &mut self.world,
                &mut self.extension_points,
            );
            self.plugin_manager.deactivate_all(&mut context);
        }
        Ok(())
    }

    /// 使用渲染器运行主循环（桌面平台）
    ///
    /// 创建 winit 事件循环和编辑器渲染器，进入桌面事件驱动的主循环。
    /// 处理窗口事件（调整大小、关闭等），每帧执行 tick 和渲染。
    ///
    /// 由于 winit 的事件循环需要获取 EditorShell 的所有权，
    /// 调用此方法后 `self` 将被替换为默认的空壳程序。
    #[cfg(not(target_arch = "wasm32"))]
    pub fn run_with_renderer(&mut self) -> GResult<()> {
        let event_loop = winit::event_loop::EventLoop::new()
            .map_err(|e| GError { kind: GErrorKind::Platform, message: format!("无法创建事件循环: {}", e) })?;

        let (w, h) = self.window_sizes.get(&0).copied().unwrap_or((1280, 720));
        let surface_info = SurfaceInfo::new(w, h, "GG Editor");
        let renderer = EditorRenderer::new(&event_loop, surface_info)?;
        self.renderers.insert(0, renderer);
        self.is_running = true;

        {
            let mut context = EditorContext::new(
                &mut self.services,
                &mut self.commands,
                &mut self.events,
                &mut self.world,
                &mut self.extension_points,
            );
            self.plugin_manager.activate_all(&mut context)?;
        }

        let mut shell = std::mem::take(self);
        let mut plugins_shut_down = false;

        #[allow(deprecated)]
        event_loop
            .run(move |event, elwt| match event {
                winit::event::Event::WindowEvent { event, .. } => match event {
                    winit::event::WindowEvent::Resized(physical_size) => {
                        shell.window_sizes.insert(0, (physical_size.width, physical_size.height));
                        if let Some(renderer) = shell.renderers.get_mut(&0) {
                            renderer.resize(physical_size.width, physical_size.height, None);
                        }
                        shell.events.publish(EditorEvent::WindowResized {
                            window_id: 0,
                            width: physical_size.width,
                            height: physical_size.height,
                        });
                    }
                    winit::event::WindowEvent::CloseRequested => {
                        shell.shutdown();
                    }
                    winit::event::WindowEvent::Destroyed => {
                        shell.events.publish(EditorEvent::WindowDestroyed { window_id: 0 });
                    }
                    winit::event::WindowEvent::Focused(_focused) => {}
                    winit::event::WindowEvent::MouseInput { state, button, .. } => {
                        let mouse_button = match button {
                            winit::event::MouseButton::Left => MouseButton::Left,
                            winit::event::MouseButton::Right => MouseButton::Right,
                            winit::event::MouseButton::Middle => MouseButton::Middle,
                            _ => return,
                        };
                        let event = match state {
                            winit::event::ElementState::Pressed => {
                                EditorEvent::MouseDown { button: mouse_button, position: shell.cursor_position }
                            }
                            winit::event::ElementState::Released => {
                                EditorEvent::MouseUp { button: mouse_button, position: shell.cursor_position }
                            }
                        };
                        shell.events.publish(event);
                    }
                    winit::event::WindowEvent::CursorMoved { position, .. } => {
                        shell.cursor_position = (position.x as f32, position.y as f32);
                        shell.events.publish(EditorEvent::MouseMove { position: shell.cursor_position });
                    }
                    winit::event::WindowEvent::MouseWheel { delta, .. } => {
                        let d = match delta {
                            winit::event::MouseScrollDelta::LineDelta(x, y) => (x * 20.0, y * 20.0),
                            winit::event::MouseScrollDelta::PixelDelta(p) => (p.x as f32, p.y as f32),
                        };
                        shell.events.publish(EditorEvent::MouseWheel { delta: d, position: shell.cursor_position });
                    }
                    winit::event::WindowEvent::KeyboardInput { event, .. } => {
                        let key = winit_key_to_key(event.logical_key);
                        if let Some(key) = key {
                            match event.state {
                                winit::event::ElementState::Pressed => {
                                    match key {
                                        Key::Control => shell.modifier_state.ctrl = true,
                                        Key::Shift => shell.modifier_state.shift = true,
                                        Key::Alt => shell.modifier_state.alt = true,
                                        _ => {}
                                    }
                                    if let Some(command_name) =
                                        shell.shortcut_registry.find_command(&key, &shell.modifier_state)
                                    {
                                        shell.events.publish(EditorEvent::Custom {
                                            name: "ShortcutTriggered".to_string(),
                                            data: Box::new(command_name.to_string()),
                                        });
                                    }
                                    shell.events.publish(EditorEvent::KeyDown { key });
                                }
                                winit::event::ElementState::Released => {
                                    match key {
                                        Key::Control => shell.modifier_state.ctrl = false,
                                        Key::Shift => shell.modifier_state.shift = false,
                                        Key::Alt => shell.modifier_state.alt = false,
                                        _ => {}
                                    }
                                    shell.events.publish(EditorEvent::KeyUp { key });
                                }
                            }
                        }
                    }
                    _ => {}
                },
                winit::event::Event::AboutToWait => {
                    if !shell.is_running && !plugins_shut_down {
                        {
                            let mut context = EditorContext::new(
                                &mut shell.services,
                                &mut shell.commands,
                                &mut shell.events,
                                &mut shell.world,
                                &mut shell.extension_points,
                            );
                            shell.plugin_manager.deactivate_all(&mut context);
                        }
                        plugins_shut_down = true;
                        elwt.exit();
                        return;
                    }
                    if shell.is_running {
                        for create in shell.window_service.drain_pending_creates() {
                            shell.events.publish(EditorEvent::WindowCreated { window_id: create.window_id.0 });
                        }
                        for destroy_id in shell.window_service.drain_pending_destroys() {
                            shell.events.publish(EditorEvent::WindowDestroyed { window_id: destroy_id.0 });
                        }
                        let _ = shell.tick();
                        for renderer in shell.renderers.values_mut() {
                            let _ = renderer.render_frame_with_scene(&shell.ui_tree, |_, _| {});
                        }
                    }
                }
                _ => {}
            })
            .map_err(|e| GError { kind: GErrorKind::Platform, message: format!("事件循环错误: {}", e) })?;

        Ok(())
    }

    /// 关闭编辑器
    pub fn shutdown(&mut self) {
        self.is_running = false;
    }
}

impl Default for EditorShell {
    fn default() -> Self {
        Self::new()
    }
}

/// 将 winit 逻辑键转换为引擎 Key 枚举
fn winit_key_to_key(key: winit::keyboard::Key) -> Option<Key> {
    match key {
        winit::keyboard::Key::Character(ref c) => match c.as_str() {
            "a" | "A" => Some(Key::A),
            "b" | "B" => Some(Key::B),
            "c" | "C" => Some(Key::C),
            "d" | "D" => Some(Key::D),
            "e" | "E" => Some(Key::E),
            "f" | "F" => Some(Key::F),
            "g" | "G" => Some(Key::G),
            "h" | "H" => Some(Key::H),
            "i" | "I" => Some(Key::I),
            "j" | "J" => Some(Key::J),
            "k" | "K" => Some(Key::K),
            "l" | "L" => Some(Key::L),
            "m" | "M" => Some(Key::M),
            "n" | "N" => Some(Key::N),
            "o" | "O" => Some(Key::O),
            "p" | "P" => Some(Key::P),
            "q" | "Q" => Some(Key::Q),
            "r" | "R" => Some(Key::R),
            "s" | "S" => Some(Key::S),
            "t" | "T" => Some(Key::T),
            "u" | "U" => Some(Key::U),
            "v" | "V" => Some(Key::V),
            "w" | "W" => Some(Key::W),
            "x" | "X" => Some(Key::X),
            "y" | "Y" => Some(Key::Y),
            "z" | "Z" => Some(Key::Z),
            "0" => Some(Key::Num0),
            "1" => Some(Key::Num1),
            "2" => Some(Key::Num2),
            "3" => Some(Key::Num3),
            "4" => Some(Key::Num4),
            "5" => Some(Key::Num5),
            "6" => Some(Key::Num6),
            "7" => Some(Key::Num7),
            "8" => Some(Key::Num8),
            "9" => Some(Key::Num9),
            _ => None,
        },
        winit::keyboard::Key::Named(named) => match named {
            winit::keyboard::NamedKey::Escape => Some(Key::Escape),
            winit::keyboard::NamedKey::Enter => Some(Key::Enter),
            winit::keyboard::NamedKey::Space => Some(Key::Space),
            winit::keyboard::NamedKey::Tab => Some(Key::Tab),
            winit::keyboard::NamedKey::Backspace => Some(Key::Backspace),
            winit::keyboard::NamedKey::Delete => Some(Key::Delete),
            winit::keyboard::NamedKey::Insert => Some(Key::Insert),
            winit::keyboard::NamedKey::Home => Some(Key::Home),
            winit::keyboard::NamedKey::End => Some(Key::End),
            winit::keyboard::NamedKey::PageUp => Some(Key::PageUp),
            winit::keyboard::NamedKey::PageDown => Some(Key::PageDown),
            winit::keyboard::NamedKey::ArrowUp => Some(Key::ArrowUp),
            winit::keyboard::NamedKey::ArrowDown => Some(Key::ArrowDown),
            winit::keyboard::NamedKey::ArrowLeft => Some(Key::ArrowLeft),
            winit::keyboard::NamedKey::ArrowRight => Some(Key::ArrowRight),
            winit::keyboard::NamedKey::F1 => Some(Key::F1),
            winit::keyboard::NamedKey::F2 => Some(Key::F2),
            winit::keyboard::NamedKey::F3 => Some(Key::F3),
            winit::keyboard::NamedKey::F4 => Some(Key::F4),
            winit::keyboard::NamedKey::F5 => Some(Key::F5),
            winit::keyboard::NamedKey::F6 => Some(Key::F6),
            winit::keyboard::NamedKey::F7 => Some(Key::F7),
            winit::keyboard::NamedKey::F8 => Some(Key::F8),
            winit::keyboard::NamedKey::F9 => Some(Key::F9),
            winit::keyboard::NamedKey::F10 => Some(Key::F10),
            winit::keyboard::NamedKey::F11 => Some(Key::F11),
            winit::keyboard::NamedKey::F12 => Some(Key::F12),
            winit::keyboard::NamedKey::Shift => Some(Key::Shift),
            winit::keyboard::NamedKey::Control => Some(Key::Control),
            winit::keyboard::NamedKey::Alt => Some(Key::Alt),
            _ => None,
        },
        _ => None,
    }
}
