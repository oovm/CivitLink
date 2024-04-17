//! 编辑器上下文
//!
//! 提供面板和插件访问编辑器核心子系统的统一入口。

use crate::{command::CommandManager, event::EventBus, service::ServiceRegistry};

/// 编辑器配置
///
/// 存储编辑器的全局配置项，包括主题、字体大小、自动保存等。
pub struct EditorConfig {
    /// 主题名称
    pub theme: String,
    /// 字体大小
    pub font_size: u32,
    /// 是否自动保存
    pub auto_save: bool,
    /// 是否显示行号
    pub show_line_numbers: bool,
    /// 制表符大小
    pub tab_size: u32,
}

impl Default for EditorConfig {
    fn default() -> Self {
        Self { theme: "dark".to_string(), font_size: 14, auto_save: false, show_line_numbers: true, tab_size: 4 }
    }
}

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
    pub fn new(services: &'a mut ServiceRegistry, commands: &'a mut CommandManager, events: &'a mut EventBus) -> Self {
        Self { services, commands, events }
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
