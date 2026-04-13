//! 扩展 API 系统
//!
//! 提供扩展点注册表、扩展 API trait 及其默认实现，
//! 支持插件通过版本化接口注册命令、面板、事件订阅和扩展点。

use std::{any::Any, collections::HashMap};

/// 扩展点处理器类型
///
/// 接受 `Any` 数据引用，返回可选的 `Any` 结果。
pub type ExtensionPointHandler = Box<dyn FnMut(&dyn Any) -> Option<Box<dyn Any>>>;

/// 扩展点注册表
///
/// 管理扩展点名称到处理器的映射，支持注册、调用和注销扩展点。
pub struct ExtensionPointRegistry {
    /// 扩展点名称到处理器的映射
    handlers: HashMap<String, ExtensionPointHandler>,
}

impl ExtensionPointRegistry {
    /// 创建空的扩展点注册表
    pub fn new() -> Self {
        Self { handlers: HashMap::new() }
    }

    /// 注册扩展点处理器
    pub fn register(&mut self, name: &str, handler: ExtensionPointHandler) {
        self.handlers.insert(name.to_string(), handler);
    }

    /// 调用已注册的扩展点，未注册返回 None
    pub fn invoke(&mut self, name: &str, data: &dyn Any) -> Option<Box<dyn Any>> {
        self.handlers.get_mut(name).and_then(|handler| handler(data))
    }

    /// 注销扩展点
    pub fn unregister(&mut self, name: &str) {
        self.handlers.remove(name);
    }
}

impl Default for ExtensionPointRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// 扩展 API trait
///
/// 为插件提供版本化的扩展接口，通过 EditorContext 间接访问编辑器子系统。
pub trait ExtensionApi {
    /// 返回扩展 API 版本号
    fn version(&self) -> &str;

    /// 注册命令
    fn register_command(&mut self, name: String, command: Box<dyn crate::command::Command>);

    /// 注销命令
    fn unregister_command(&mut self, name: &str);

    /// 注册面板
    fn register_panel(&mut self, panel: Box<dyn crate::panel::EditorPanel>);

    /// 注销面板
    fn unregister_panel(&mut self, name: &str);

    /// 订阅事件
    fn subscribe_event(&mut self, handler: Box<dyn FnMut(&crate::event::EditorEvent)>) -> crate::event::SubscriptionId;

    /// 取消订阅事件
    fn unsubscribe_event(&mut self, id: crate::event::SubscriptionId);

    /// 注册扩展点
    fn register_extension_point(&mut self, name: &str, handler: ExtensionPointHandler);

    /// 查询服务
    fn get_service<T: Any + Send + Sync>(&self) -> Option<&T>;
}

/// 默认扩展 API 实现
///
/// 持有对编辑器子系统的可变引用，提供扩展 API 的默认实现。
pub struct DefaultExtensionApi<'a> {
    /// 命令管理器引用
    commands: &'a mut crate::command::CommandManager,
    /// 事件总线引用
    events: &'a mut crate::event::EventBus,
    /// 服务注册表引用
    services: &'a mut crate::service::ServiceRegistry,
    /// 扩展点注册表引用
    extension_points: &'a mut ExtensionPointRegistry,
    /// 待注册的命令列表
    pending_commands: Vec<(String, Box<dyn crate::command::Command>)>,
    /// 待注销的命令名称列表
    pending_unregister_commands: Vec<String>,
    /// 待注册的面板列表
    pending_panels: Vec<Box<dyn crate::panel::EditorPanel>>,
    /// 待注销的面板名称列表
    pending_unregister_panels: Vec<String>,
    /// 待取消订阅的事件 ID 列表
    pending_unsubscribe_events: Vec<crate::event::SubscriptionId>,
}

impl<'a> DefaultExtensionApi<'a> {
    /// 创建默认扩展 API 实例
    pub fn new(
        commands: &'a mut crate::command::CommandManager,
        events: &'a mut crate::event::EventBus,
        services: &'a mut crate::service::ServiceRegistry,
        extension_points: &'a mut ExtensionPointRegistry,
    ) -> Self {
        Self {
            commands,
            events,
            services,
            extension_points,
            pending_commands: Vec::new(),
            pending_unregister_commands: Vec::new(),
            pending_panels: Vec::new(),
            pending_unregister_panels: Vec::new(),
            pending_unsubscribe_events: Vec::new(),
        }
    }
}

impl ExtensionApi for DefaultExtensionApi<'_> {
    fn version(&self) -> &str {
        "0.1.0"
    }

    fn register_command(&mut self, name: String, command: Box<dyn crate::command::Command>) {
        self.pending_commands.push((name, command));
    }

    fn unregister_command(&mut self, name: &str) {
        self.pending_unregister_commands.push(name.to_string());
    }

    fn register_panel(&mut self, panel: Box<dyn crate::panel::EditorPanel>) {
        self.pending_panels.push(panel);
    }

    fn unregister_panel(&mut self, name: &str) {
        self.pending_unregister_panels.push(name.to_string());
    }

    fn subscribe_event(&mut self, handler: Box<dyn FnMut(&crate::event::EditorEvent)>) -> crate::event::SubscriptionId {
        self.events.subscribe(handler)
    }

    fn unsubscribe_event(&mut self, id: crate::event::SubscriptionId) {
        self.pending_unsubscribe_events.push(id);
    }

    fn register_extension_point(&mut self, name: &str, handler: ExtensionPointHandler) {
        self.extension_points.register(name, handler);
    }

    fn get_service<T: Any + Send + Sync>(&self) -> Option<&T> {
        self.services.get::<T>()
    }
}
