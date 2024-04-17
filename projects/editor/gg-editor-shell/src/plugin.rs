//! 编辑器插件系统

use crate::context::EditorContext;

/// 编辑器插件 trait
///
/// 插件可以在编辑器启动时初始化、在关闭时清理，
/// 通过 `EditorContext` 访问服务注册表、命令管理器和事件总线。
pub trait EditorPlugin {
    /// 插件名称
    fn name(&self) -> &str;

    /// 初始化插件
    fn initialize(&mut self, context: &mut EditorContext);

    /// 关闭插件
    fn shutdown(&mut self, context: &mut EditorContext);
}
