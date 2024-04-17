#![warn(missing_docs)]

//! GG 编辑器壳程序框架
//!
//! 提供微内核架构的编辑器基础设施，包括服务注册表、命令系统、事件总线、
//! 插件系统、面板管理和 Docking 布局系统。

pub mod command;
pub mod context;
pub mod docking;
pub mod dynamic_loader;
pub mod event;
pub mod extension;
pub mod panel;
pub mod plugin;
pub mod service;
pub mod shell;

pub use command::{Command, CommandManager, ModifierState, ShortcutKey, ShortcutRegistry};
pub use context::{EditorConfig, EditorContext};
pub use docking::{
    DockRegion, DockRegionConfig, DockRegionSnapshot, DockSplit, DockSplitSnapshot, DockingLayout, LayoutSnapshot, PanelLayout,
};
#[cfg(not(target_arch = "wasm32"))]
pub use dynamic_loader::DynamicPluginLoader;
pub use dynamic_loader::{EDITOR_PLUGIN_ABI_VERSION, PluginEntryFn, PluginManifest};
pub use event::{DragData, DragState, DragVisualFeedback, EditorEvent, EventBus, Key, MouseButton, SubscriptionId};
pub use extension::{DefaultExtensionApi, ExtensionApi, ExtensionPointHandler, ExtensionPointRegistry};
pub use panel::{EditorPanel, PanelLayoutHint, PanelPosition};
pub use plugin::{EditorPlugin, PluginDescriptor, PluginEntry, PluginManager, PluginState};
pub use service::{
    DefaultWindowService, PendingWindowCreate, ServiceDescriptor, ServiceLifecycle, ServiceRegistry, WindowId, WindowService,
    WinitWindowInfo, WinitWindowService,
};
pub use shell::EditorShell;
