//! 编辑器上下文
//!
//! 提供面板和插件访问编辑器核心子系统的统一入口。

use crate::command::CommandManager;
use crate::event::EventBus;
use crate::service::ServiceRegistry;

/// 编辑器上下文
///
/// 聚合了对服务注册表、命令管理器和事件总线的可变引用，
/// 作为面板渲染和插件操作的统一上下文参数传递。
pub struct EditorContext<'a> {
    services: &'a mut ServiceRegistry,
    commands: &'a mut CommandManager,
    events: &'a mut EventBus,
}

impl<'a> EditorContext<'a> {
    /// 创建新的编辑器上下文
    pub fn new(
        services: &'a mut ServiceRegistry,
        commands: &'a mut CommandManager,
        events: &'a mut EventBus,
    ) -> Self {
        Self {
            services,
            commands,
            events,
        }
    }

    /// 获取服务注册表引用
    pub fn services(&self) -> &ServiceRegistry {
        self.services
    }

    /// 获取服务注册表可变引用
    pub fn services_mut(&mut self) -> &mut ServiceRegistry {
        self.services
    }

    /// 获取命令管理器引用
    pub fn commands(&self) -> &CommandManager {
        self.commands
    }

    /// 获取命令管理器可变引用
    pub fn commands_mut(&mut self) -> &mut CommandManager {
        self.commands
    }

    /// 获取事件总线引用
    pub fn events(&self) -> &EventBus {
        self.events
    }

    /// 获取事件总线可变引用
    pub fn events_mut(&mut self) -> &mut EventBus {
        self.events
    }
}
