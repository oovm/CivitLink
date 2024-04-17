#![feature(new_range_api)]
#![warn(missing_docs)]

//! GG 引擎 UI 组件库
//! 提供声明式 UI 组件、布局系统和事件处理

/// Canvas 游戏UI系统
pub mod canvas;
/// DPI 缩放
pub mod dpi;
/// UI 事件系统
pub mod event;
/// 字体和字形基础设施
pub mod font;
/// 基于 ab_glyph 的文本整形器
pub mod font_shaper;
/// GUI 事件系统（组件级）
pub mod gui_event;
/// GUI 运行时
pub mod gui_runtime;
/// 布局引擎
pub mod layout;
/// UI 节点树
pub mod node;
/// 性能优化
pub mod perf;
/// 响应式系统
pub mod reactive;
/// UI 渲染
pub mod render;
/// UI 样式定义
pub mod style;
/// 模板节点转换
pub mod template;
/// 主题系统
pub mod theme;
/// 控件 trait 和核心类型
pub mod widget;
/// 内置控件
pub mod widgets;
/// 基础控件实现
pub mod widgets_builtin;

pub use dpi::{DpiAware, DpiScale};
pub use event::{EventSystem, UiEvent};
pub use font::{FontAtlas, FontFace, GlyphInfo, TextShaper};
pub use gui_event::{EventContext, EventPhase, GuiEvent, Key, KeyModifiers, MouseButton};
pub use gui_runtime::{ComponentWrapper, GuiFactory, GuiRenderer, GuiRuntime};
pub use layout::{LayoutEngine, LayoutResult};
pub use node::{AccessibilityNode, AccessibilityTree, AccessibilityTreeNode, AriaRole, UiNode, UiNodeData, UiNodeId, UiTree};
pub use reactive::{CloneAny, ComputedSignal, PropertyValue, Signal, SignalId, batch, effect, remove_effect};
pub use render::UiRenderer;
pub use style::*;
pub use template::{parse_color, parse_inline_style, template_node_to_ui_tree, template_node_to_ui_tree_mapped};
pub use theme::{ShadowToken, Theme, ThemeId, ThemeRegistry, ThemeTokens};
pub use widget::{DirtyFlag, DynamicWidget, UsageHint, UsageHints, Widget, WidgetLifecycle};
pub use widgets::{
    Button, ButtonState, ComboBox, ContextMenu, Dialog, DialogButton, Dropdown, GridView, MenuItem, Panel, ProgressBar,
    Spinner, TabBar, TabItem, TextBox, Tooltip, TooltipPosition, TreeNode, TreeView,
};
pub use widgets_builtin::{
    Button as BuiltinButton, Image as BuiltinImage, Input as BuiltinInput, Layout as BuiltinLayout, Panel as BuiltinPanel,
    ScrollView as BuiltinScrollView, Stack as BuiltinStack, Text as BuiltinText,
};
