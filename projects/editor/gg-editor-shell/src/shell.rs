//! 编辑器壳程序

use crate::command::CommandManager;
use crate::context::EditorContext;
use crate::event::EventBus;
use crate::panel::EditorPanel;
use crate::plugin::EditorPlugin;
use crate::service::ServiceRegistry;
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
        }
    }

    /// 注册面板
    ///
    /// 将面板添加到面板列表，并调用其 `on_register` 方法。
    pub fn register_panel(&mut self, panel: Box<dyn EditorPanel>) {
        self.panels.push(panel);
        let mut context =
            EditorContext::new(&mut self.services, &mut self.commands, &mut self.events);
        if let Some(panel) = self.panels.last_mut() {
            panel.on_register(&mut context);
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

    /// 执行一帧
    ///
    /// 先处理待处理事件，再构建所有可见面板的 UI 节点树，
    /// 然后计算布局，最后渲染。
    pub fn tick(&mut self) -> GResult<()> {
        self.events.process_pending();
        let mut context =
            EditorContext::new(&mut self.services, &mut self.commands, &mut self.events);
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
            let mut context =
                EditorContext::new(&mut self.services, &mut self.commands, &mut self.events);
            for plugin in &mut self.plugins {
                plugin.initialize(&mut context);
            }
        }
        while self.is_running {
            self.tick()?;
        }
        {
            let mut context =
                EditorContext::new(&mut self.services, &mut self.commands, &mut self.events);
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
