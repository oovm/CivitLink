//! 基础UI组件实现
//!
//! 提供编辑器UI系统的基础组件，如Layout、Button、Text等
//! 所有组件实现从 gg-ui 重新导出，并扩展编辑器专用组件

pub use gg_ui::widgets_builtin::{Button, Image, Input, Layout, Panel, ScrollView, Stack, Text};

pub mod asset_browser;
pub mod hierarchy;
pub mod inspector;
pub mod menu_bar;
pub mod property_field;
pub mod scene_view;
pub mod split_view;
pub mod status_bar;
pub mod tab_bar;
pub mod tab_container;
pub mod toolbar;
pub mod tree_view;

pub use asset_browser::{AssetBrowser, AssetEntry};
pub use hierarchy::{HierarchyNode, HierarchyView};
pub use inspector::{InspectorPanel, PropertyEntry};
pub use menu_bar::{EditorMenuBar, MenuGroup, MenuItem};
pub use property_field::{PropertyField, PropertyType};
pub use scene_view::SceneView;
pub use split_view::{SplitOrientation, SplitView};
pub use status_bar::{EditorStatusBar, StatusSection};
pub use tab_bar::{EditorTabBar, Tab};
pub use tab_container::{TabContainer, TabEntry};
pub use toolbar::{EditorTool, EditorToolbar};
pub use tree_view::{EditorTreeNode, EditorTreeView};
