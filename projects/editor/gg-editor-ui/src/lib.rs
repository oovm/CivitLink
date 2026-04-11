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

use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};
use oak_voc::{TemplateNode, VxDocument};

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

/// GUI 运行时
pub struct GuiRuntime {
    /// 渲染器
    renderer: Arc<RwLock<dyn GuiRenderer>>,
    /// 根组件
    root_component: Option<Arc<RwLock<dyn VxComponent>>>,
    /// 是否正在运行
    is_running: bool,
    /// 目标帧率
    target_fps: Option<u32>,
}

impl GuiRuntime {
    /// 创建新的 GUI 运行时
    pub fn new<T: GuiRenderer + 'static>(renderer: T) -> Self {
        Self { 
            renderer: Arc::new(RwLock::new(renderer)), 
            root_component: None, 
            is_running: false, 
            target_fps: None 
        }
    }

    /// 设置根组件
    pub fn set_root_component(&mut self, component: Arc<RwLock<dyn VxComponent>>) {
        self.root_component = Some(component);
    }

    /// 设置目标帧率
    pub fn set_target_fps(&mut self, fps: u32) {
        self.target_fps = Some(fps);
    }

    /// 运行 GUI 循环
    pub fn run(&mut self) {
        self.is_running = true;
        while self.is_running {
            let frame_start = Instant::now();
            self.update();
            if let Some(fps) = self.target_fps {
                let frame_duration = Duration::from_secs_f64(1.0 / fps as f64);
                let elapsed = frame_start.elapsed();
                if elapsed < frame_duration {
                    std::thread::sleep(frame_duration - elapsed);
                }
            }
        }
    }

    /// 停止 GUI 循环
    pub fn shutdown(&mut self) {
        self.is_running = false;
    }

    /// 处理单个帧
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
}

/// 平台特定的 GUI 工厂
pub trait GuiFactory {
    /// 创建 GUI 渲染器
    fn create_renderer(&self) -> Arc<dyn GuiRenderer>;

    /// 创建基础组件
    fn create_component(&self, component_type: &str) -> Arc<RwLock<dyn VxComponent>>;
}
