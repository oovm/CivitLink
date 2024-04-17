//! 命令系统
//!
//! 提供可撤销/重做的命令 trait 和命令管理器。

use crate::context::EditorContext;
use gg_core::{GError, GErrorKind, GResult};

/// 命令 trait
///
/// 所有可撤销的操作都应实现此 trait，由 `CommandManager` 管理执行、撤销和重做。
pub trait Command {
    /// 执行命令
    fn execute(&mut self, context: &mut EditorContext) -> GResult<()>;

    /// 撤销命令
    fn undo(&mut self, context: &mut EditorContext) -> GResult<()>;

    /// 命令描述
    fn description(&self) -> &str;
}

/// 命令管理器
///
/// 维护撤销栈和重做栈，支持命令的执行、撤销和重做操作。
pub struct CommandManager {
    undo_stack: Vec<Box<dyn Command>>,
    redo_stack: Vec<Box<dyn Command>>,
}

impl CommandManager {
    /// 创建空的命令管理器
    pub fn new() -> Self {
        Self { undo_stack: Vec::new(), redo_stack: Vec::new() }
    }

    /// 执行命令
    ///
    /// 调用命令的 `execute` 方法，将其压入撤销栈，并清空重做栈。
    pub fn execute(&mut self, mut command: Box<dyn Command>, context: &mut EditorContext) {
        self.redo_stack.clear();
        if command.execute(context).is_ok() {
            self.undo_stack.push(command);
        }
    }

    /// 撤销最近一次命令
    ///
    /// 从撤销栈弹出最近执行的命令，调用其 `undo` 方法，然后压入重做栈。
    pub fn undo(&mut self, context: &mut EditorContext) -> GResult<()> {
        let mut command = self
            .undo_stack
            .pop()
            .ok_or_else(|| GError { kind: GErrorKind::Other, message: "没有可撤销的命令".to_string() })?;
        command.undo(context)?;
        self.redo_stack.push(command);
        Ok(())
    }

    /// 重做最近一次撤销的命令
    ///
    /// 从重做栈弹出最近撤销的命令，调用其 `execute` 方法，然后压入撤销栈。
    pub fn redo(&mut self, context: &mut EditorContext) -> GResult<()> {
        let mut command = self
            .redo_stack
            .pop()
            .ok_or_else(|| GError { kind: GErrorKind::Other, message: "没有可重做的命令".to_string() })?;
        command.execute(context)?;
        self.undo_stack.push(command);
        Ok(())
    }

    /// 是否可撤销
    pub fn can_undo(&self) -> bool {
        !self.undo_stack.is_empty()
    }

    /// 是否可重做
    pub fn can_redo(&self) -> bool {
        !self.redo_stack.is_empty()
    }
}

impl Default for CommandManager {
    fn default() -> Self {
        Self::new()
    }
}
