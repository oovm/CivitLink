//! 编辑器壳程序

use crate::{
    command::CommandManager,
    context::{EditorConfig, EditorContext},
    event::{EditorEvent, EventBus},
    panel::EditorPanel,
    plugin::EditorPlugin,
    service::{DefaultWindowService, ServiceRegistry},
};
use gg_core::GResult;
use gg_ui::{LayoutEngine, UiTree};

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
    /// 已注册的插件列表
    plugins: Vec<Box<dyn EditorPlugin>>,
    /// UI 节点树
    ui_tree: UiTree,
    /// 是否运行中
    is_running: bool,
    /// 默认窗口服务
    window_service: DefaultWindowService,
    /// 编辑器配置
    editor_config: EditorConfig,
}

impl EditorShell {
    /// 创建新的编辑器壳程序
    pub fn new() -> Self {
        Self {
            services: ServiceRegistry::new(),
            commands: CommandManager::new(),
            events: EventBus::new(),
            panels: Vec::new(),
            plugins: Vec::new(),
            ui_tree: UiTree::new(),
            is_running: false,
            window_service: DefaultWindowService::new(),
            editor_config: EditorConfig::default(),
        }
    }

    /// 注册面板
    ///
    /// 将面板添加到面板列表，调用其 `on_register` 方法，并发布 `PanelRegistered` 事件。
    pub fn register_panel(&mut self, panel: Box<dyn EditorPanel>) {
        let panel_name = panel.name().to_string();
        self.panels.push(panel);
        let mut context = EditorContext::new(&mut self.services, &mut self.commands, &mut self.events);
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
            let mut context = EditorContext::new(&mut self.services, &mut self.commands, &mut self.events);
            panel.on_unregister(&mut context);
            self.events.publish(EditorEvent::PanelUnregistered { panel_name: name.to_string() });
        }
    }

    /// 注册插件
    pub fn register_plugin(&mut self, plugin: Box<dyn EditorPlugin>) {
        self.plugins.push(plugin);
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
    pub fn window_service(&self) -> &DefaultWindowService {
        &self.window_service
    }

    /// 获取窗口服务可变引用
    pub fn window_service_mut(&mut self) -> &mut DefaultWindowService {
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

    /// 执行一帧
    ///
    /// 先处理待处理事件，再处理窗口焦点事件，
    /// 然后构建所有可见面板的 UI 节点树，计算布局，最后渲染。
    pub fn tick(&mut self) -> GResult<()> {
        self.events.process_pending();
        for window_id in self.window_service.drain_pending_focus() {
            self.events.publish(EditorEvent::WindowFocused { window_id });
        }
        let mut context = EditorContext::new(&mut self.services, &mut self.commands, &mut self.events);
        for panel in &mut self.panels {
            if panel.is_visible() {
                panel.build_ui(&mut context, &mut self.ui_tree)?;
            }
        }
        LayoutEngine::compute(&mut self.ui_tree, 800.0, 600.0);
        Ok(())
    }

    /// 运行主循环
    ///
    /// 依次调用所有插件的 `initialize` 方法，进入帧循环，
    /// 退出后调用所有插件的 `shutdown` 方法。
    pub fn run(&mut self) -> GResult<()> {
        self.is_running = true;
        {
            let mut context = EditorContext::new(&mut self.services, &mut self.commands, &mut self.events);
            for plugin in &mut self.plugins {
                plugin.initialize(&mut context);
            }
        }
        while self.is_running {
            self.tick()?;
        }
        {
            let mut context = EditorContext::new(&mut self.services, &mut self.commands, &mut self.events);
            for plugin in &mut self.plugins {
                plugin.shutdown(&mut context);
            }
        }
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
