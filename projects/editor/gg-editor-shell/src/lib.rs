#![warn(missing_docs)]

//! GG 编辑器壳程序框架
//!
//! 提供微内核架构的编辑器基础设施，包括服务注册表、命令系统、事件总线、
//! 插件系统、面板管理和 Docking 布局系统。

pub mod command;
pub mod context;
pub mod docking;
pub mod event;
pub mod panel;
pub mod plugin;
pub mod service;
pub mod shell;

pub use command::{Command, CommandManager};
pub use context::{EditorConfig, EditorContext};
pub use docking::{DockRegion, DockRegionConfig, DockSplit, DockingLayout, PanelLayout};
pub use event::{EditorEvent, EventBus, Key, MouseButton, SubscriptionId};
pub use panel::{EditorPanel, PanelLayoutHint, PanelPosition};
pub use plugin::EditorPlugin;
pub use service::{DefaultWindowService, ServiceRegistry, WindowId, WindowService};
pub use shell::EditorShell;
