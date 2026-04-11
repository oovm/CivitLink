//! GG Editor UI 系统
//!
//! 基于 Valkyrie Widget 的编辑器UI系统，参考 Unity UI Toolkit 设计理念
//! 为编辑器提供高性能、声明式的UI开发体验

#![warn(missing_docs)]

/// 组件系统模块
pub mod components;
/// 渲染系统模块
pub mod renderer;
/// 事件系统模块
pub mod events;
/// 样式系统模块
pub mod styles;
/// 状态管理模块
pub mod state;
/// Flexbox 布局引擎模块
pub mod layout;
/// Uber-Shader 模块
pub mod uber_shader;
/// UsageHints GPU 驱动变换模块
pub mod usage_hints;

use std::ops::{BitAnd, BitAndAssign, BitOr, BitOrAssign, Not};
use std::sync::{Arc, RwLock};
use oak_voc::{TemplateNode, VxDocument};

/// 脏标记位标志，用于标识组件需要更新的类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DirtyFlag(u32);

impl DirtyFlag {
    /// 无脏标记
    pub const NONE: DirtyFlag = DirtyFlag(0);
    /// 布局脏标记，表示需要重新计算布局
    pub const LAYOUT: DirtyFlag = DirtyFlag(1 << 0);
    /// 样式脏标记，表示需要重新应用样式
    pub const STYLE: DirtyFlag = DirtyFlag(1 << 1);
    /// 内容脏标记，表示需要重新渲染内容
    pub const CONTENT: DirtyFlag = DirtyFlag(1 << 2);
    /// 变换脏标记，表示需要重新计算变换矩阵
    pub const TRANSFORM: DirtyFlag = DirtyFlag(1 << 3);
    /// 全部脏标记，表示所有类型都需要更新
    pub const ALL: DirtyFlag = DirtyFlag(Self::LAYOUT.0 | Self::STYLE.0 | Self::CONTENT.0 | Self::TRANSFORM.0);

    /// 判断是否包含指定脏标记
    pub fn contains(self, other: DirtyFlag) -> bool {
        self.0 & other.0 != 0
    }

    /// 判断是否没有任何脏标记
    pub fn is_empty(self) -> bool {
        self.0 == 0
    }

    /// 获取内部位值
    pub fn bits(self) -> u32 {
        self.0
    }
}

impl Default for DirtyFlag {
    fn default() -> Self {
        DirtyFlag::NONE
    }
}

impl BitOr for DirtyFlag {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        DirtyFlag(self.0 | rhs.0)
    }
}

impl BitOrAssign for DirtyFlag {
    fn bitor_assign(&mut self, rhs: Self) {
        self.0 |= rhs.0;
    }
}

impl BitAnd for DirtyFlag {
    type Output = Self;

    fn bitand(self, rhs: Self) -> Self::Output {
        DirtyFlag(self.0 & rhs.0)
    }
}

impl BitAndAssign for DirtyFlag {
    fn bitand_assign(&mut self, rhs: Self) {
        self.0 &= rhs.0;
    }
}

impl Not for DirtyFlag {
    type Output = Self;

    fn not(self) -> Self::Output {
        DirtyFlag(!self.0 & DirtyFlag::ALL.0)
    }
}

/// Flex 布局方向
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FlexDirection {
    /// 水平方向排列
    Row,
    /// 垂直方向排列
    Column,
}

/// 组件生命周期状态
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ComponentLifecycle {
    /// 已创建但未挂载
    Created,
    /// 已挂载到 DOM 树
    Mounted,
    /// 已更新
    Updated,
    /// 已卸载
    Unmounted,
}

/// VX 组件 trait，对齐 *.widget 文件格式的三段式结构
pub trait VxComponent: Send + Sync {
    /// 渲染模板，返回 UI 节点描述
    fn render_template(&self) -> vx_parser::TemplateNode;

    /// 脚本初始化，设置响应式状态和事件处理
    fn script_setup(&mut self);

    /// 获取样式定义
    fn get_style(&self) -> Option<&str> {
        None
    }

    /// 获取组件 ID
    fn get_id(&self) -> &str;

    /// 处理 GUI 事件
    fn handle_event(&mut self, event: &events::GuiEvent, ctx: &mut events::EventContext);

    /// 获取子组件列表
    fn children(&self) -> Vec<Arc<RwLock<dyn VxComponent>>> {
        Vec::new()
    }

    /// 组件挂载时调用
    fn on_mount(&mut self) {}

    /// 组件更新时调用
    fn on_update(&mut self) {}

    /// 组件卸载时调用
    fn on_cleanup(&mut self) {}

    /// 判断组件是否有脏标记
    fn is_dirty(&self) -> bool {
        false
    }

    /// 清除所有脏标记
    fn clear_dirty(&mut self) {}

    /// 获取当前脏标记
    fn get_dirty_flags(&self) -> DirtyFlag {
        DirtyFlag::NONE
    }

    /// 标记指定脏标记，默认为空操作
    fn mark_dirty(&mut self, _flag: DirtyFlag) {}

    /// 获取组件的 UsageHints，指示哪些属性变化由 GPU 处理
    fn usage_hints(&self) -> usage_hints::UsageHints {
        usage_hints::UsageHints::NONE
    }
}

/// 动态 VX 组件，由 VxDocument 转换而来
pub struct DynamicVxComponent {
    /// 组件 ID
    id: String,
    /// 模板节点
    template: Option<vx_parser::TemplateNode>,
    /// 格式化的样式字符串
    style_string: Option<String>,
    /// 脚本源码
    script_source: Option<String>,
    /// 生命周期状态
    lifecycle: ComponentLifecycle,
    /// 脏标记位标志
    dirty_flags: DirtyFlag,
    /// UsageHints 标志，指示哪些属性变化由 GPU 处理
    hints: usage_hints::UsageHints,
}

impl DynamicVxComponent {
    /// 创建新的动态 VX 组件
    pub fn new(id: &str) -> Self {
        Self {
            id: id.to_string(),
            template: None,
            style_string: None,
            script_source: None,
            lifecycle: ComponentLifecycle::Created,
            dirty_flags: DirtyFlag::NONE,
            hints: usage_hints::UsageHints::NONE,
        }
    }

    /// 从文档数据创建动态 VX 组件
    pub fn from_document(
        id: &str,
        template: Option<vx_parser::TemplateNode>,
        style_string: Option<String>,
        script_source: Option<String>,
    ) -> Self {
        Self {
            id: id.to_string(),
            template,
            style_string,
            script_source,
            lifecycle: ComponentLifecycle::Created,
            dirty_flags: DirtyFlag::NONE,
            hints: usage_hints::UsageHints::NONE,
        }
    }

    /// 获取脚本源码引用
    pub fn script_source(&self) -> Option<&str> {
        self.script_source.as_deref()
    }

    /// 获取生命周期状态
    pub fn lifecycle(&self) -> ComponentLifecycle {
        self.lifecycle
    }

    /// 标记指定脏标记
    pub fn mark_dirty(&mut self, flag: DirtyFlag) {
        self.dirty_flags |= flag;
    }
}

impl VxComponent for DynamicVxComponent {
    fn render_template(&self) -> vx_parser::TemplateNode {
        match &self.template {
            Some(node) => node.clone(),
            None => vx_parser::TemplateNode::Text(String::new()),
        }
    }

    fn script_setup(&mut self) {
        let _ = &self.script_source;
    }

    fn get_style(&self) -> Option<&str> {
        self.style_string.as_ref().and_then(|s| if s.is_empty() { None } else { Some(s.as_str()) })
    }

    fn get_id(&self) -> &str {
        &self.id
    }

    fn handle_event(&mut self, _event: &events::GuiEvent, _ctx: &mut events::EventContext) {}

    fn on_mount(&mut self) {
        self.lifecycle = ComponentLifecycle::Mounted;
    }

    fn on_update(&mut self) {
        self.lifecycle = ComponentLifecycle::Updated;
    }

    fn on_cleanup(&mut self) {
        self.lifecycle = ComponentLifecycle::Unmounted;
    }

    fn is_dirty(&self) -> bool {
        !self.dirty_flags.is_empty()
    }

    fn clear_dirty(&mut self) {
        self.dirty_flags = DirtyFlag::NONE;
    }

    fn get_dirty_flags(&self) -> DirtyFlag {
        self.dirty_flags
    }

    fn mark_dirty(&mut self, flag: DirtyFlag) {
        self.dirty_flags |= flag;
    }

    fn usage_hints(&self) -> usage_hints::UsageHints {
        self.hints
    }
}

/// GUI 渲染器特质
pub trait GuiRenderer: Send + Sync {
    /// 渲染组件树
    fn render(&mut self, component: Arc<dyn VxComponent>);

    /// 处理事件，接收根组件用于三阶段事件分发
    fn process_events(&mut self, root: Option<&Arc<RwLock<dyn VxComponent>>>);

    /// 更新渲染器
    fn update(&mut self);

    /// 设置视口大小
    fn set_viewport_size(&mut self, width: u32, height: u32);
}

/// GUI 运行时，采用事件驱动模型
pub struct GuiRuntime {
    /// 渲染器
    renderer: Arc<RwLock<dyn GuiRenderer>>,
    /// 根组件
    root_component: Option<Arc<RwLock<dyn VxComponent>>>,
}

impl GuiRuntime {
    /// 创建新的 GUI 运行时
    pub fn new<T: GuiRenderer + 'static>(renderer: T) -> Self {
        Self {
            renderer: Arc::new(RwLock::new(renderer)),
            root_component: None,
        }
    }

    /// 设置根组件
    pub fn set_root_component(&mut self, component: Arc<RwLock<dyn VxComponent>>) {
        self.root_component = Some(component);
    }

    /// 接收外部事件，标记相关节点为脏
    pub fn process_event(&mut self, event: &events::GuiEvent) {
        if let Some(component) = &self.root_component {
            let mut ctx = events::EventContext::new(events::EventPhase::AtTarget);
            if let Ok(mut comp) = component.write() {
                comp.handle_event(event, &mut ctx);
            }
            self.propagate_dirty_from_children(component);
        }
    }

    /// 仅在存在脏节点时执行重绘，无脏节点则直接返回
    pub fn commit_render(&mut self) {
        if !self.has_dirty_components() {
            return;
        }

        self.update();

        self.clear_all_dirty();
    }

    /// 手动标记指定节点为脏
    pub fn mark_dirty(&mut self, component_id: &str, flag: DirtyFlag) {
        if let Some(root) = &self.root_component {
            self.mark_dirty_recursive(root, component_id, flag);
        }
    }

    /// 检查是否有脏组件
    pub fn has_dirty_components(&self) -> bool {
        if let Some(component) = &self.root_component {
            if let Ok(comp) = component.read() {
                if comp.is_dirty() {
                    return true;
                }
            }
            self.has_dirty_children_recursive(component)
        } else {
            false
        }
    }

    /// 仅在存在脏组件时执行更新
    pub fn update(&mut self) {
        if let Ok(mut renderer) = self.renderer.write() {
            renderer.process_events(self.root_component.as_ref());
        }

        if let Some(component) = &self.root_component {
            if let Ok(mut comp) = component.write() {
                comp.on_update();
            }
        }

        if let Some(component) = &self.root_component {
            if let Ok(comp) = component.read() {
                let cached_id = comp.get_id().to_string();
                if let Ok(mut renderer) = self.renderer.write() {
                    let arc_comp: Arc<dyn VxComponent> = Arc::new(ComponentWrapper { inner: Arc::clone(component), cached_id });
                    renderer.render(arc_comp);
                }
            }
        }

        if let Ok(mut renderer) = self.renderer.write() {
            renderer.update();
        }
    }

    /// 递归标记脏节点
    fn mark_dirty_recursive(&self, component: &Arc<RwLock<dyn VxComponent>>, target_id: &str, flag: DirtyFlag) -> bool {
        let mut found = false;
        if let Ok(mut comp) = component.write() {
            if comp.get_id() == target_id {
                comp.mark_dirty(flag);
                found = true;
            }
        }

        let children = if let Ok(comp) = component.read() {
            comp.children()
        } else {
            Vec::new()
        };

        for child in &children {
            if self.mark_dirty_recursive(child, target_id, flag) {
                found = true;
            }
        }

        if found {
            if let Ok(mut comp) = component.write() {
                if comp.get_id() != target_id {
                    comp.on_update();
                }
            }
        }

        found
    }

    /// 从子节点向上传播脏标记到父节点
    fn propagate_dirty_from_children(&self, component: &Arc<RwLock<dyn VxComponent>>) {
        let children = if let Ok(comp) = component.read() {
            comp.children()
        } else {
            Vec::new()
        };

        let mut child_has_dirty = false;
        for child in &children {
            self.propagate_dirty_from_children(child);
            if let Ok(comp) = child.read() {
                if comp.is_dirty() {
                    child_has_dirty = true;
                }
            }
        }

        if child_has_dirty {
            if let Ok(mut comp) = component.write() {
                comp.on_update();
            }
        }
    }

    /// 递归检查子组件是否有脏标记
    fn has_dirty_children_recursive(&self, component: &Arc<RwLock<dyn VxComponent>>) -> bool {
        let children = if let Ok(comp) = component.read() {
            comp.children()
        } else {
            Vec::new()
        };

        for child in &children {
            if let Ok(comp) = child.read() {
                if comp.is_dirty() {
                    return true;
                }
            }
            if self.has_dirty_children_recursive(child) {
                return true;
            }
        }

        false
    }

    /// 清除所有组件的脏标记
    fn clear_all_dirty(&self) {
        if let Some(component) = &self.root_component {
            self.clear_dirty_recursive(component);
        }
    }

    /// 递归清除脏标记
    fn clear_dirty_recursive(&self, component: &Arc<RwLock<dyn VxComponent>>) {
        if let Ok(mut comp) = component.write() {
            comp.clear_dirty();
        }

        let children = if let Ok(comp) = component.read() {
            comp.children()
        } else {
            Vec::new()
        };

        for child in &children {
            self.clear_dirty_recursive(child);
        }
    }
}

/// 组件包装器，用于将 RwLock 包装的组件转换为 Arc<dyn VxComponent>
struct ComponentWrapper {
    /// 内部组件
    inner: Arc<RwLock<dyn VxComponent>>,
    /// 缓存的组件 ID
    cached_id: String,
}

impl VxComponent for ComponentWrapper {
    fn render_template(&self) -> vx_parser::TemplateNode {
        if let Ok(comp) = self.inner.read() {
            comp.render_template()
        } else {
            vx_parser::TemplateNode::Text(String::new())
        }
    }

    fn script_setup(&mut self) {
        if let Ok(mut comp) = self.inner.write() {
            comp.script_setup();
        }
    }

    fn get_style(&self) -> Option<&str> {
        None
    }

    fn get_id(&self) -> &str {
        &self.cached_id
    }

    fn handle_event(&mut self, event: &events::GuiEvent, ctx: &mut events::EventContext) {
        if let Ok(mut comp) = self.inner.write() {
            comp.handle_event(event, ctx);
        }
    }

    fn children(&self) -> Vec<Arc<RwLock<dyn VxComponent>>> {
        if let Ok(comp) = self.inner.read() {
            comp.children()
        } else {
            Vec::new()
        }
    }

    fn on_mount(&mut self) {
        if let Ok(mut comp) = self.inner.write() {
            comp.on_mount();
        }
    }

    fn on_update(&mut self) {
        if let Ok(mut comp) = self.inner.write() {
            comp.on_update();
        }
    }

    fn on_cleanup(&mut self) {
        if let Ok(mut comp) = self.inner.write() {
            comp.on_cleanup();
        }
    }

    fn is_dirty(&self) -> bool {
        if let Ok(comp) = self.inner.read() {
            comp.is_dirty()
        } else {
            false
        }
    }

    fn clear_dirty(&mut self) {
        if let Ok(mut comp) = self.inner.write() {
            comp.clear_dirty();
        }
    }

    fn get_dirty_flags(&self) -> DirtyFlag {
        if let Ok(comp) = self.inner.read() {
            comp.get_dirty_flags()
        } else {
            DirtyFlag::NONE
        }
    }

    fn mark_dirty(&mut self, flag: DirtyFlag) {
        if let Ok(mut comp) = self.inner.write() {
            comp.mark_dirty(flag);
        }
    }

    fn usage_hints(&self) -> usage_hints::UsageHints {
        if let Ok(comp) = self.inner.read() {
            comp.usage_hints()
        } else {
            usage_hints::UsageHints::NONE
        }
    }
}

/// 平台特定的 GUI 工厂
pub trait GuiFactory {
    /// 创建 GUI 渲染器
    fn create_renderer(&self) -> Arc<dyn GuiRenderer>;

    /// 创建基础组件
    fn create_component(&self, component_type: &str) -> Arc<RwLock<dyn VxComponent>>;
}
