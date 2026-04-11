//! 命令系统
//!
//! 提供可撤销/重做的命令 trait 和命令管理器。

use std::collections::HashMap;

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
    /// 撤销栈
    pub(crate) undo_stack: Vec<Box<dyn Command>>,
    /// 重做栈
    pub(crate) redo_stack: Vec<Box<dyn Command>>,
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

/// 修饰键状态
///
/// 跟踪键盘修饰键（Ctrl、Shift、Alt）的按下状态。
#[derive(Debug, Clone, Copy, Default)]
pub struct ModifierState {
    /// Ctrl 键是否按下
    pub ctrl: bool,
    /// Shift 键是否按下
    pub shift: bool,
    /// Alt 键是否按下
    pub alt: bool,
}

/// 快捷键定义
///
/// 由一个主键和可选的修饰键组合构成，用于触发命令。
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ShortcutKey {
    /// 主键
    pub key: crate::event::Key,
    /// 需要 Ctrl 修饰键
    pub ctrl: bool,
    /// 需要 Shift 修饰键
    pub shift: bool,
    /// 需要 Alt 修饰键
    pub alt: bool,
}

impl ShortcutKey {
    /// 创建无修饰键的快捷键
    pub fn new(key: crate::event::Key) -> Self {
        Self { key, ctrl: false, shift: false, alt: false }
    }

    /// 添加 Ctrl 修饰键
    pub fn with_ctrl(mut self) -> Self {
        self.ctrl = true;
        self
    }

    /// 添加 Shift 修饰键
    pub fn with_shift(mut self) -> Self {
        self.shift = true;
        self
    }

    /// 添加 Alt 修饰键
    pub fn with_alt(mut self) -> Self {
        self.alt = true;
        self
    }

    /// 检查快捷键是否与当前修饰键状态和按键匹配
    pub fn matches(&self, key: &crate::event::Key, modifiers: &ModifierState) -> bool {
        &self.key == key && self.ctrl == modifiers.ctrl && self.shift == modifiers.shift && self.alt == modifiers.alt
    }
}

/// 快捷键注册表
///
/// 管理快捷键到命令名称的映射，支持注册、查询和匹配。
pub struct ShortcutRegistry {
    /// 快捷键到命令名称的映射
    shortcuts: HashMap<ShortcutKey, String>,
}

impl ShortcutRegistry {
    /// 创建空的快捷键注册表
    pub fn new() -> Self {
        Self { shortcuts: HashMap::new() }
    }

    /// 注册快捷键
    ///
    /// 将快捷键映射到命令名称，若快捷键已存在则替换。
    pub fn register(&mut self, shortcut: ShortcutKey, command_name: String) {
        self.shortcuts.insert(shortcut, command_name);
    }

    /// 查找匹配的命令名称
    ///
    /// 根据按键和修饰键状态查找对应的命令名称。
    pub fn find_command(&self, key: &crate::event::Key, modifiers: &ModifierState) -> Option<&str> {
        self.shortcuts.iter().find(|(shortcut, _)| shortcut.matches(key, modifiers)).map(|(_, name)| name.as_str())
    }
}

impl Default for ShortcutRegistry {
    fn default() -> Self {
        Self::new()
    }
}
