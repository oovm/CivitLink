//! 编辑器上下文
//!
//! 提供面板和插件访问编辑器核心子系统的统一入口。

use crate::{command::CommandManager, event::EventBus, service::ServiceRegistry};
use gg_world::GameWorld;

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
/// 聚合了对服务注册表、命令管理器、事件总线和游戏世界的可变引用，
/// 作为面板渲染和插件操作的统一上下文参数传递。
pub struct EditorContext<'a> {
    services: &'a mut ServiceRegistry,
    commands: &'a mut CommandManager,
    events: &'a mut EventBus,
    world: &'a mut GameWorld,
}

impl<'a> EditorContext<'a> {
    /// 创建新的编辑器上下文
    pub fn new(
        services: &'a mut ServiceRegistry,
        commands: &'a mut CommandManager,
        events: &'a mut EventBus,
        world: &'a mut GameWorld,
    ) -> Self {
        Self { services, commands, events, world }
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

    /// 获取游戏世界引用
    pub fn world(&self) -> &GameWorld {
        self.world
    }

    /// 获取游戏世界可变引用
    pub fn world_mut(&mut self) -> &mut GameWorld {
        self.world
    }

    /// 执行命令并压入撤销栈
    ///
    /// 便捷方法，避免外部调用者同时持有 `CommandManager` 和 `EditorContext` 的可变引用
    /// 导致的借用冲突。内部依次清空重做栈、执行命令、将命令压入撤销栈。
    pub fn execute_command(&mut self, command: Box<dyn crate::command::Command>) {
        self.commands.redo_stack.clear();
        let mut cmd = command;
        if cmd.execute(self).is_ok() {
            self.commands.undo_stack.push(cmd);
        }
    }

    /// 撤销最近一次命令
    ///
    /// 便捷方法，从撤销栈弹出最近执行的命令，调用其 `undo` 方法，
    /// 然后压入重做栈。避免借用冲突。
    pub fn undo_command(&mut self) -> gg_core::GResult<()> {
        let mut command = self.commands.undo_stack.pop().ok_or_else(|| gg_core::GError {
            kind: gg_core::GErrorKind::Other,
            message: "没有可撤销的命令".to_string(),
        })?;
        command.undo(self)?;
        self.commands.redo_stack.push(command);
        Ok(())
    }

    /// 重做最近一次撤销的命令
    ///
    /// 便捷方法，从重做栈弹出最近撤销的命令，调用其 `execute` 方法，
    /// 然后压入撤销栈。避免借用冲突。
    pub fn redo_command(&mut self) -> gg_core::GResult<()> {
        let mut command = self.commands.redo_stack.pop().ok_or_else(|| gg_core::GError {
            kind: gg_core::GErrorKind::Other,
            message: "没有可重做的命令".to_string(),
        })?;
        command.execute(self)?;
        self.commands.undo_stack.push(command);
        Ok(())
    }
}
