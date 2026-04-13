//! GG Engine UI 运行时
//!
//! 提供 Widget 组件体系、响应式状态管理、生命周期钩子和基础组件实现，
//! 对齐 *.widget 文件格式的 template/script/style 三段式结构。
//!
//! 核心类型均从 gg-ui 重导出，本 crate 仅保留运行时适配层和插件集成。

#![warn(missing_docs)]

/// GUI 渲染器适配器模块
pub mod gui_renderer_adapter;
/// UI 插件模块
pub mod plugin;

pub use gg_ui::{
    AccessibilityNode, AccessibilityTree, AccessibilityTreeNode, AriaRole, CloneAny, ComputedSignal, DirtyFlag, DynamicWidget,
    EventContext, EventPhase, FlexDirection, GuiEvent, GuiFactory, GuiRenderer, GuiRuntime, Key, KeyModifiers, MouseButton,
    PropertyValue, Signal, SignalId, UsageHint, UsageHints, Widget, WidgetLifecycle, batch, effect, remove_effect,
};

pub use oak_voc::{ast::VxParseError, parse_vx};
